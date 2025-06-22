use super::broadcast::BroadcastManager;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use log::{info, warn, error};

pub async fn handle_websocket<S>(
    ws_stream: WebSocketStream<S>,
    broadcast_manager: Arc<BroadcastManager>,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (ws_sender, mut ws_receiver) = ws_stream.split();
    let mut receiver = broadcast_manager.subscribe().await;

    // Handle incoming messages from WebSocket
    let ws_sender_clone = Arc::new(tokio::sync::Mutex::new(ws_sender));
    let ws_sender_for_broadcast = Arc::clone(&ws_sender_clone);

    // Task to handle broadcasting to this WebSocket
    let broadcast_task = tokio::spawn(async move {
        while let Ok(message) = receiver.recv().await {
            let mut sender = ws_sender_for_broadcast.lock().await;
            if let Err(e) = sender.send(Message::Text(message)).await {
                error!("Failed to send message to WebSocket: {}", e);
                break;
            }
        }
    });

    // Task to handle incoming WebSocket messages
    let receive_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    info!("Received WebSocket message: {}", text);
                    // Handle incoming commands here
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    warn!("WebSocket receive error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = broadcast_task => {
            info!("Broadcast task completed");
        }
        _ = receive_task => {
            info!("Receive task completed");
        }
    }

    Ok(())
}

pub async fn handle_static_files() -> Result<(), Box<dyn std::error::Error>> {
    // Static file serving implementation
    Ok(())
}