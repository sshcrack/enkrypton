use std::thread;

use actix_web::{App, HttpServer, web};
use actix_web_actors::ws;
use anyhow::Result;
use log::{error, info};
use tauri::async_runtime::block_on;

use crate::tor::config::CONFIG;

use super::routes::{hello, ws_index};

pub fn start_webserver() {
    thread::spawn(|| {
        let res = block_on(server_mainloop());

        if res.is_err() {
            error!("{}", res.unwrap_err());
        } else {
            info!("Webserver stopped.")
        }
    });
}

async fn server_mainloop() -> Result<()> {
    HttpServer::new(|| {
        return App::new()
        .service(hello)
        .route("/ws/", web::get().to(ws_index))
    })
        .bind(("127.0.0.1", CONFIG.service_port))?
        .run()
        .await?;

    return Ok(());
}
