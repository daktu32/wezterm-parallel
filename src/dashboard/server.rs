use super::handlers::handle_websocket;
use super::broadcast::BroadcastManager;
use crate::config::ServerConfig;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use std::sync::Arc;
use log::{info, error};

pub struct DashboardServer {
    #[allow(dead_code)]
    config: ServerConfig,
    broadcast_manager: Arc<BroadcastManager>,
}

impl DashboardServer {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            broadcast_manager: Arc::new(BroadcastManager::new()),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = "127.0.0.1:8080"; // Default address since we're using Unix sockets for IPC
        let listener = TcpListener::bind(addr).await?;
        info!("Dashboard server listening on {}", addr);

        while let Ok((stream, addr)) = listener.accept().await {
            info!("New connection from {}", addr);
            
            let broadcast_manager = Arc::clone(&self.broadcast_manager);
            
            tokio::spawn(async move {
                match accept_async(stream).await {
                    Ok(ws_stream) => {
                        if let Err(e) = handle_websocket(ws_stream, broadcast_manager).await {
                            error!("WebSocket error: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to accept WebSocket connection: {}", e);
                    }
                }
            });
        }

        Ok(())
    }

    pub fn get_broadcast_manager(&self) -> Arc<BroadcastManager> {
        Arc::clone(&self.broadcast_manager)
    }
}