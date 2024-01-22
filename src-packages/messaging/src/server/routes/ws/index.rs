use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::server::ws_manager::WsActor;

/// The index route for the websocket
///
/// # Arguments
///
/// * `req` - The http request that was sent from the client
/// * `stream` - The websocket stream
pub async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(WsActor::new(), &req, stream);
    println!("{:?}", resp);
    resp
}