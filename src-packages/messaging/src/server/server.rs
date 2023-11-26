use std::thread;

use actix_web::{web, App, HttpServer};
use anyhow::Result;
use log::{error, info};
use shared::config::CONFIG;
use tauri::async_runtime::block_on;

use super::routes::{hello, ws_index};

/// Starts the local webserver in a new thread and runs the server_mainloop
pub fn start_webserver() {
    thread::Builder::new().name("webserver".to_string()).spawn(move || {
        let res = block_on(server_mainloop());

        if res.is_err() {
            error!("{}", res.unwrap_err());
        } else {
            info!("Webserver stopped.")
        }
    }).unwrap();
}

/// Just initializes a new async webserver and listens just to local connections and to the specified port
async fn server_mainloop() -> Result<()> {
    HttpServer::new(|| {
        return App::new()
            // Return the default message to tell other clients that this server is actually alive
            .service(hello)
            // The websocket endpoint
            .route("/ws/", web::get().to(ws_index));
    })
    // Bind just to localhost and run
    .bind(("127.0.0.1", CONFIG.service_port()))?
    .run()
    .await?;

    return Ok(());
}
