use actix::{Actor, StreamHandler};
use actix_web_actors::ws::{self, ProtocolError, Message};

pub struct MessagingServer {

}


impl Actor for MessagingServer {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        
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