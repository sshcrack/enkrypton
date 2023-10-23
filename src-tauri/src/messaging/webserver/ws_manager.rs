use actix::{Actor, StreamHandler};
use actix_web_actors::ws::{self, Message, ProtocolError};
use log::debug;
use smol::future::block_on;

use crate::messaging::MESSAGING;


pub struct MessagingServer {
    receiver: Option<String>,
}

impl Actor for MessagingServer {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {}

    fn stopped(&mut self, ctx: &mut Self::Context) {
        if let Some(onion_host) = &self.receiver {
            debug!("Removing link for {}", onion_host);

            block_on(MESSAGING.write()).remove_link(onion_host);
        }
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<Message, ProtocolError>> for MessagingServer {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}
