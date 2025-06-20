use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::path::Path;
use tracing::{info, error, warn};
use wezterm_multi_dev::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting WezTerm Multi-Process Development Framework");
    
    // Unix Domain Socket path
    let socket_path = "/tmp/wezterm-multi-dev.sock";
    
    // Remove existing socket file if it exists
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path)?;
    }
    
    // Create Unix Domain Socket listener
    let listener = UnixListener::bind(socket_path)?;
    info!("IPC Server listening on {}", socket_path);
    
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                info!("New client connected");
                tokio::spawn(handle_client(stream));
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_client(mut stream: UnixStream) {
    let mut buffer = [0; 1024];
    
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => {
                info!("Client disconnected");
                break;
            }
            Ok(n) => {
                let data = &buffer[..n];
                
                // Try to parse JSON message
                match serde_json::from_slice::<Message>(data) {
                    Ok(message) => {
                        info!("Received message: {:?}", message);
                        
                        // Handle message
                        let response = handle_message(message).await;
                        
                        // Send response
                        if let Ok(response_json) = serde_json::to_vec(&response) {
                            if let Err(e) = stream.write_all(&response_json).await {
                                error!("Failed to send response: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse message: {}", e);
                        
                        // Send error response
                        let error_msg = Message::StatusUpdate {
                            process_id: "system".to_string(),
                            status: format!("Parse error: {}", e),
                        };
                        
                        if let Ok(error_json) = serde_json::to_vec(&error_msg) {
                            let _ = stream.write_all(&error_json).await;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to read from stream: {}", e);
                break;
            }
        }
    }
}

async fn handle_message(message: Message) -> Message {
    match message {
        Message::Ping => {
            info!("Ping received, responding with Pong");
            Message::Pong
        }
        Message::WorkspaceCreate { name, template } => {
            info!("Creating workspace: {} with template: {}", name, template);
            // TODO: Implement workspace creation logic
            Message::StatusUpdate {
                process_id: "workspace_manager".to_string(),
                status: format!("Workspace '{}' created successfully", name),
            }
        }
        Message::ProcessSpawn { workspace, command } => {
            info!("Spawning process in workspace '{}': {}", workspace, command);
            // TODO: Implement process spawning logic
            Message::StatusUpdate {
                process_id: "process_manager".to_string(),
                status: format!("Process spawned in workspace '{}'", workspace),
            }
        }
        Message::TaskQueue { id, priority, command } => {
            info!("Queuing task {}: {} (priority: {})", id, command, priority);
            // TODO: Implement task queuing logic
            Message::StatusUpdate {
                process_id: "task_manager".to_string(),
                status: format!("Task '{}' queued successfully", id),
            }
        }
        other => {
            warn!("Unhandled message type: {:?}", other);
            Message::StatusUpdate {
                process_id: "system".to_string(),
                status: "Unknown message type".to_string(),
            }
        }
    }
}
