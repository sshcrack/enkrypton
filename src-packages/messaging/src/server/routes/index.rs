use actix_web::{Responder, HttpResponse, get};
use shared::DEFAULT_HTTP_RETURN;

/// Default http return to identity enkrypton
/// This is used from the enkrypton client to identify the server
#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body(DEFAULT_HTTP_RETURN.to_string())
}