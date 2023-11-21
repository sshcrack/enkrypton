use std::time::{Duration, Instant};

use actix::{
    Actor, ActorContext, AsyncContext,
    StreamHandler,
};
use actix_web::web::Bytes;
use actix_web_actors::ws::{self, Message, ProtocolError};
use anyhow::Result;
use async_channel::{Receiver, Sender, TryRecvError};
use log::{debug, info, error};
use payloads::packets::{C2SPacket, S2CPacket};
use smol::future::block_on;

use crate::general::{IdentityProvider, IdentityVerify, MESSAGING, HEARTBEAT_TIMEOUT};

use super::manager_ext::ManagerExt;

pub type ServerChannels = (Receiver<C2SPacket>, Sender<S2CPacket>);

pub struct WsActor {
    // From Packets being sent from the server to the client
    s_rx: Box<Receiver<S2CPacket>>,
    pub s_tx: Box<Sender<S2CPacket>>,

    // For clients being sent from the client to the server
    pub c_rx: Box<Receiver<C2SPacket>>,
    c_tx: Box<Sender<C2SPacket>>,
    receiver: Option<String>,
    last_heartbeat: Instant
}

impl Actor for WsActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let rx = self.s_rx.clone();

        // Constantly checking for new messages
        ctx.run_interval(Duration::from_millis(100), move |_a, ctx| {
            let p = rx.try_recv();
            if let Err(e) = p {
                if e == TryRecvError::Closed {
                    debug!("Channel has been closed. Stopping...");
                    ctx.stop();
                }

                return;
            }
            let p = p.unwrap();
            debug!("Sending packet to client {:?}", p);

            let res = p.try_into();
            if let Err(e) = res {
                error!("Could not parse packet: {:?}", e);
                return;
            }

            let packet: Bytes = res.unwrap();
            ctx.binary(packet);
        });

        ctx.run_interval(Duration::from_secs(1), |a, ctx| {
            let timed_out = a.last_heartbeat.elapsed() > *HEARTBEAT_TIMEOUT;

            if !timed_out {
                return;
            }

            debug!(
                "Websocket timed out, stopping (onionHost: {:?})",
                a.receiver
            );
            ctx.stop();
            ctx.binary(b"".to_vec());
        });
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        self.c_tx.close();
        self.s_tx.close();
        if let Some(onion_host) = &self.receiver {
            debug!("Removing connection for {}", onion_host);

            block_on(block_on(MESSAGING.read()).remove_connection(onion_host));
        }
    }
}

impl WsActor {
    pub fn new() -> Self {
        let (s_tx, s_rx) = async_channel::unbounded();
        let (c_tx, c_rx) = async_channel::unbounded();

        Self {
            last_heartbeat: Instant::now(),
            receiver: None,
            c_tx: Box::new(c_tx),
            c_rx: Box::new(c_rx),

            s_tx: Box::new(s_tx),
            s_rx: Box::new(s_rx),
        }
    }

    pub async fn inner_handle(
        &mut self,
        packet: C2SPacket,
        ctx: &mut <Self as Actor>::Context,
    ) -> Result<()> {
        match packet {
            C2SPacket::IdentityVerified => {
                if let Some(onion_host) = &self.receiver {
                    let messaging = MESSAGING.read().await;
                    messaging.set_self_verified(&onion_host, &self).await;
                    messaging.check_verify_status(&onion_host).await?;
                } else {
                    error!("Received IdentityVerified packet but no onion host was set");
                }
            }
            C2SPacket::SetIdentity(identity) => {
                info!("[SERVER] Verifying identity for {:?}...", identity);
                identity.verify().await?;

                let messaging = MESSAGING.read().await;
                self.receiver = Some(identity.hostname.clone());
                messaging
                    .set_remote_verified(&identity.hostname, &self)
                    .await;
                messaging.check_verify_status(&identity.hostname).await?;

                let b: Bytes = S2CPacket::IdentityVerified.try_into()?;

                let verify_p: Bytes = S2CPacket::identity(&identity.hostname).await?.try_into()?;
                info!("[SERVER] Identity verified. Sending packet.");

                ctx.binary(verify_p);
                ctx.binary(b);
            }
            C2SPacket::Message(msg) => {
                // Sending the message to main handler
                self.c_tx.send(C2SPacket::Message(msg)).await?;
            }
        }
        Ok(())
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<Message, ProtocolError>> for WsActor {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
                self.last_heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(_)) => {}
            Ok(ws::Message::Binary(bin)) => {
                let res = C2SPacket::try_from(&bin.to_vec());
                if let Err(e) = res {
                    error!("Could not parse packet: {:?}", e);
                    return;
                }

                if self.c_tx.is_closed() {
                    error!("Client receive channel close, stopping actor");
                    ctx.stop();
                    return;
                }

                let packet = res.unwrap();
                let res = self.inner_handle(packet, ctx);
                let res = block_on(res);

                if res.is_err() {
                    error!("Could not handle packet: {:?}", res.unwrap_err());
                    return;
                }
            }
            _ => (),
        }
    }
}
