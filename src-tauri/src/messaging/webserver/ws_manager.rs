use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::web::Bytes;
use actix_web_actors::ws::{self, Message, ProtocolError};
use anyhow::Result;
use async_channel::{Receiver, Sender};
use lazy_static::lazy_static;
use log::{debug, error};
use smol::future::block_on;

use crate::messaging::{
    packages::{C2SPacket, S2CPacket},
    MESSAGING,
};

lazy_static! {
    pub static ref TIMEOUT: Duration = Duration::from_secs(10);
}

pub struct WsActor {
    // From Packets being sent from the server to the client
    s_rx: Receiver<S2CPacket>,
    s_tx: Sender<S2CPacket>,

    // For clients being sent from the client to the server
    c_rx: Receiver<C2SPacket>,
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
            let timed_out = a.last_heartbeat.elapsed() > *TIMEOUT;

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

    fn stopped(&mut self, ctx: &mut Self::Context) {
        if let Some(onion_host) = &self.receiver {
            debug!("Removing link for {}", onion_host);

            block_on(MESSAGING.write()).remove_link(onion_host);
        }
    }
}

impl WsActor {
    fn new() -> Self {
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

    async fn inner_handle(&mut self, bin: Bytes, ctx: &mut <Self as Actor>::Context) -> Result<()> {
        let bin = bin.to_vec();

        let packet = C2SPacket::try_from(&bin)?;
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
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => {
                let res = block_on(self.inner_handle(bin, ctx));
                if let Err(e) = res {
                    error!("Could not handle packet: {:?}", e);
                }
            }
            _ => (),
        }
    }
}
