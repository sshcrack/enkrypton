use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use anyhow::Result;
use futures_util::{
    future, pin_mut,
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{debug, error, warn};
use tauri::async_runtime::block_on;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_socks::tcp::Socks5Stream;
use tokio_tungstenite::{
    client_async_with_config, connect_async, tungstenite::Message, WebSocketStream,
};

use super::SocksProxy;

type WriteStream = SplitSink<WebSocketStream<Socks5Stream<TcpStream>>, Message>;
type ReadStream = SplitStream<WebSocketStream<Socks5Stream<TcpStream>>>;

#[derive(Debug, Clone)]
pub struct MessagingClient {
    write: Arc<Mutex<WriteStream>>,
    read: Arc<Mutex<ReadStream>>,
    listen_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl MessagingClient {
    pub async fn new(onion_addr: &str) -> Result<Self> {
        let proxy = SocksProxy::new()?;
        let sock = proxy.connect(onion_addr).await?;

        let (ws_stream, _) = tokio_tungstenite::client_async(onion_addr, sock).await?;

        let (write, read) = ws_stream.split();
        return Ok(MessagingClient {
            write: Arc::new(Mutex::new(write)),
            read: Arc::new(Mutex::new(read)),
            listen_thread: Arc::new(Mutex::new(None)),
        });
    }

    pub async fn send_msg(&self, msg: &str) -> Result<()> {
        debug!("Locking write mutex...");
        let mut state = self.write.lock().await;
        state.send(Message::Text(msg.to_string())).await?;

        Ok(())
    }

    pub async fn create_listen_thread(&self) -> Result<()> {
        let read = self.listen_thread.try_lock()?;
        if read.is_some() {
            warn!("Tried to listen twice");
            return Ok(());
        }

        drop(read);

        let cloned = self.clone();
        let handle = thread::spawn(move || {
            let res = block_on(cloned.read_thread());
            if let Err(e) = res {
                error!("Could not listen: {}", e);
            }
        });

        debug!("Writing handle");
        let mut write = self.listen_thread.lock().await;
        write.replace(handle);

        Ok(())
    }

    pub async fn read_thread(&self) -> Result<()> {
        debug!("Locking read mutex...");
        let mut state = self.read.try_lock()?;

        state
            .by_ref()
            .for_each(move |msg| async {
                if let Err(e) = msg {
                    warn!("Read err: {}", e)
                }
            })
            .await;

        Ok(())
    }
}
