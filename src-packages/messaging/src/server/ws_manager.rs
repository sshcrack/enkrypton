use std::time::{Duration, Instant};

use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::web::Bytes;
use actix_web_actors::ws::{self, Message, ProtocolError};
use anyhow::Result;
use async_channel::{Receiver, Sender, TryRecvError};
use log::{debug, error, info, warn};
use payloads::{
    event::AppHandleExt,
    packets::{C2SPacket, S2CPacket},
    payloads::{WsClientStatus, WsClientUpdatePayload, WsMessageStatus},
};
use shared::get_app;
use smol::future::block_on;

use crate::general::{IdentityProvider, IdentityVerify, HEARTBEAT_TIMEOUT, MESSAGING};

use super::manager_ext::ManagerExt;

pub type ServerChannels = (Receiver<C2SPacket>, Sender<S2CPacket>);

pub struct WsActor {
    s_rx: Box<Receiver<S2CPacket>>,
    // Used to send packets from the server to the client
    pub s_tx: Box<Sender<S2CPacket>>,

    // Used to receive packets from the client
    pub c_rx: Box<Receiver<C2SPacket>>,
    c_tx: Box<Sender<C2SPacket>>,

    // The receiver the `WsActor` is handling
    receiver: Option<String>,
    // The time the last heartbeat was sent
    last_heartbeat: Instant,
}

impl Actor for WsActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let rx = self.s_rx.clone();

        // Constantly checking for new messages to send to the client
        ctx.run_interval(Duration::from_millis(100), move |_a, ctx| {
            let p = rx.try_recv();
            if let Err(e) = p {
                if e == TryRecvError::Closed {
                    warn!("[SERVER] Channel has been closed. Stopping...");
                    ctx.stop();
                }

                return;
            }
            let p = p.unwrap();
            debug!("[SERVER] Sending packet to client {:?}", p);

            let res = p.try_into();
            if let Err(e) = res {
                error!("[SERVER] Could not parse packet: {:?}", e);
                return;
            }

            // Actually sending the packet to the client
            let packet: Bytes = res.unwrap();
            ctx.binary(packet);
        });

        // Checking for a heartbeat timeout
        ctx.run_interval(Duration::from_secs(1), |a, ctx| {
            let timed_out = a.last_heartbeat.elapsed() > *HEARTBEAT_TIMEOUT;

            if !timed_out {
                return;
            }

            warn!(
                "[SERVER] Websocket timed out, stopping (onionHost: {:?})",
                a.receiver
            );
            // Disconnecting the websocket and stopping the actor
            ctx.stop();
            ctx.binary(b"".to_vec());
        });
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        // Closing all communication channels
        self.c_tx.close();
        self.s_tx.close();

        if let Some(onion_host) = &self.receiver {
            debug!("[SERVER] Connection was stopped. Removing connection for {}", onion_host);

            // Sending a disconnect update to the frontend
            let _ = block_on(get_app())
                .emit_payload(WsClientUpdatePayload {
                    hostname: onion_host.to_string(),
                    status: WsClientStatus::Disconnected,
                })
                .map_err(|e| warn!("[SERVER] Could not emit ws client update: {:?}", e));

            // And remove the connection from the messaging manager
            block_on(block_on(MESSAGING.read()).remove_connection(onion_host));
        }
    }
}

impl WsActor {
    // Creates a new `WsActor`
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

    // Handles a packet that was received
    pub async fn inner_handle(
        &mut self,
        packet: C2SPacket,
        ctx: &mut <Self as Actor>::Context,
    ) -> Result<()> {
        let mut packet_auth = None;
        match packet {
            // Again, same as for the client, setting that this connection has been verified on our end
            C2SPacket::IdentityVerified => {
                if let Some(onion_host) = &self.receiver {
                    let messaging = MESSAGING.read().await;
                    messaging.set_self_verified(&onion_host, &self).await;
                    messaging.check_verified(&onion_host).await?;
                } else {
                    error!("[SERVER] Received IdentityVerified packet but no onion host was set");
                }
            }
            // Verifying the connection of the client and sending a `IdentityVerified` back. Oh and we send our own identity back
            C2SPacket::SetIdentity(identity) => {
                info!("[SERVER] Verifying identity for {:?}...", identity);
                identity.verify().await?;

                let messaging = MESSAGING.read().await;
                self.receiver = Some(identity.hostname.clone());

                // Setting the remote verified status
                messaging
                    .set_remote_verified(&identity.hostname, &self)
                    .await;
                messaging.check_verified(&identity.hostname).await?;

                let b: Bytes = S2CPacket::IdentityVerified.try_into()?;

                // Creating a new identity packet
                let verify_p: Bytes = S2CPacket::identity(&identity.hostname).await?.try_into()?;
                info!("[SERVER] Identity verified. Sending packet.");

                ctx.binary(verify_p);
                ctx.binary(b);
            }
            other => packet_auth = Some(other),
        }

        // We don't need to process this packet any further as it was already handled
        if packet_auth.is_none() {
            return Ok(());
        }

        let packet_auth = packet_auth.unwrap();
        if self.receiver.is_none() {
            warn!(
                "[SERVER] Ignoring packet {:?}. No Receiver yet.",
                packet_auth
            );
            return Ok(());
        }

        // The receiver of this connection
        let rec = self.receiver.as_ref().unwrap();

        // Return an error here if we are not verified yet.
        MESSAGING.read().await.assert_verified(rec).await?;
        match packet_auth {
            C2SPacket::Message(msg) => {
                // Sending the message to main handler
                self.c_tx.send(C2SPacket::Message(msg)).await?;
            }
            C2SPacket::MessageFailed(date) => {
                // Updating the message status to failed
                debug!("[SERVER] Received Client Packet, setting failed");
                MESSAGING
                    .read()
                    .await
                    .set_msg_status(rec, date, WsMessageStatus::Failed)
                    .await?;
            }
            C2SPacket::MessageReceived(date) => {
                // Update the message status to success
                MESSAGING
                    .read()
                    .await
                    .set_msg_status(rec, date, WsMessageStatus::Success)
                    .await?;
            }
            _ => warn!("[SERVER] Could not process packet {:?}", packet_auth),
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
                // Deserialize the packet and process it further
                let res = C2SPacket::try_from(&bin.to_vec());
                if let Err(e) = res {
                    error!("[SERVER] Could not parse packet: {:?}", e);
                    return;
                }

                if self.c_tx.is_closed() {
                    warn!("[SERVER] Client receive channel close, stopping actor");
                    ctx.stop();
                    return;
                }

                let packet = res.unwrap();
                // Processing more in that function
                let res = self.inner_handle(packet, ctx);
                let res = block_on(res);

                if res.is_err() {
                    error!("[SERVER] Could not handle packet: {:?}", res.unwrap_err());
                    return;
                }
            }
            _ => (),
        }
    }
}
