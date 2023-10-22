use actix_web::{Responder, HttpResponse, get};

use crate::tor::consts::DEFAULT_HTTP_RETURN;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body(DEFAULT_HTTP_RETURN.to_string())
}