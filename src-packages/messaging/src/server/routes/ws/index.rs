use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::server::ws_manager::WsActor;

/// The default route to the websocket
pub async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(WsActor::new(), &req, stream);
    println!("{:?}", resp);
    resp
}