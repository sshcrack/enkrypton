use std::time::{Duration, Instant};

use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::web::Bytes;
use actix_web_actors::ws::{self, Message, ProtocolError};
use anyhow::Result;
use async_channel::{Receiver, Sender};
use log::{debug, error};
use smol::future::block_on;

use crate::messaging::{
    packages::{C2SPacket, S2CPacket},
    MESSAGING, HEARTBEAT_TIMEOUT,
};


pub type ServerChannels = (Receiver<C2SPacket>, Sender<S2CPacket>);

pub struct WsActor {
    // From Packets being sent from the server to the client
    s_rx: Receiver<S2CPacket>,
    pub s_tx: Sender<S2CPacket>,

    // For clients being sent from the client to the server
    pub c_rx: Receiver<C2SPacket>,
    c_tx: Sender<C2SPacket>,
    receiver: Option<String>,
    last_heartbeat: Instant,
}

impl Actor for WsActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let rx = self.s_rx.clone();
        ctx.add_stream(async_stream::stream! {
            loop {
                let res = rx.recv().await;
                if res.is_err() {
                    break;
                }

                let res = res.unwrap();
                let res = res.try_into();
                if res.is_err() {
                    error!("Could not serialize msg: {:?}", res.unwrap_err());
                    continue;
                }

                yield Ok(res.unwrap());
            };
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

            block_on(MESSAGING.write()).remove_connection(onion_host);
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
            c_tx,
            c_rx,

            s_tx,
            s_rx,
        }
    }

    pub async fn inner_handle(&mut self, packet: C2SPacket, ctx: &mut <Self as Actor>::Context) -> Result<()> {
        match packet {
            C2SPacket::SetIdentity(identity) => {
                identity.verify().await?;

                let mut messaging = MESSAGING.write().await;
                messaging.insert_server(&identity.hostname, &self);

                let b: Bytes = S2CPacket::IdentityVerified.try_into()?;
                ctx.binary(b);
            },
            packet => {
                // We didn't handle this packet so redirecting it
                self.c_tx.send(packet).await?;
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
            Ok(ws::Message::Text(_)) => {},
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
