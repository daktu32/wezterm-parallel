use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::path::Path;
use std::sync::Arc;
use std::env;
use tracing::{info, error, warn};
use wezterm_parallel::{Message, workspace::WorkspaceManager};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for version flag
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("wezterm-parallel {}", VERSION);
        return Ok(());
    }
    
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        println!("WezTerm Multi-Process Development Framework v{}", VERSION);
        println!("Usage: wezterm-parallel [OPTIONS]");
        println!();
        println!("Options:");
        println!("  -h, --help     Show this help message");
        println!("  -v, --version  Show version information");
        println!();
        println!("The framework provides multi-process development environment");
        println!("with real-time dashboard and workspace management for WezTerm.");
        return Ok(());
    }
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting WezTerm Multi-Process Development Framework v{}", VERSION);
    
    // Initialize workspace manager
    let workspace_manager = Arc::new(WorkspaceManager::new(None)?);
    info!("Workspace manager initialized with {} workspaces", 
          workspace_manager.get_workspace_count().await);
    
    // Unix Domain Socket path
    let socket_path = "/tmp/wezterm-parallel.sock";
    
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
                let manager = Arc::clone(&workspace_manager);
                tokio::spawn(handle_client(stream, manager));
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_client(mut stream: UnixStream, workspace_manager: Arc<WorkspaceManager>) {
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
                        let response = handle_message(message, &workspace_manager).await;
                        
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

async fn handle_message(message: Message, workspace_manager: &WorkspaceManager) -> Message {
    match message {
        Message::Ping => {
            info!("Ping received, responding with Pong");
            Message::Pong
        }
        Message::WorkspaceCreate { name, template } => {
            info!("Creating workspace: {} with template: {}", name, template);
            
            match workspace_manager.create_workspace(&name, &template).await {
                Ok(()) => {
                    info!("Successfully created workspace '{}'", name);
                    Message::StatusUpdate {
                        process_id: "workspace_manager".to_string(),
                        status: format!("Workspace '{}' created successfully with template '{}'", name, template),
                    }
                }
                Err(e) => {
                    error!("Failed to create workspace '{}': {}", name, e);
                    Message::StatusUpdate {
                        process_id: "workspace_manager".to_string(),
                        status: format!("Failed to create workspace '{}': {}", name, e),
                    }
                }
            }
        }
        Message::ProcessSpawn { workspace, command } => {
            info!("Spawning process in workspace '{}': {}", workspace, command);
            
            // Check if workspace exists
            if workspace_manager.get_workspace_info(&workspace).await.is_some() {
                // TODO: Implement actual process spawning logic
                Message::StatusUpdate {
                    process_id: "process_manager".to_string(),
                    status: format!("Process '{}' spawned in workspace '{}'", command, workspace),
                }
            } else {
                error!("Workspace '{}' not found for process spawning", workspace);
                Message::StatusUpdate {
                    process_id: "process_manager".to_string(),
                    status: format!("Failed to spawn process: workspace '{}' not found", workspace),
                }
            }
        }
        Message::TaskQueue { id, priority, command } => {
            info!("Queuing task {}: {} (priority: {})", id, command, priority);
            
            // Get active workspace for task assignment
            if let Some((workspace_name, _)) = workspace_manager.get_active_workspace().await {
                // TODO: Implement task queuing logic
                Message::StatusUpdate {
                    process_id: "task_manager".to_string(),
                    status: format!("Task '{}' queued successfully in workspace '{}'", id, workspace_name),
                }
            } else {
                warn!("No active workspace found for task queuing");
                Message::StatusUpdate {
                    process_id: "task_manager".to_string(),
                    status: format!("Task '{}' queued in default workspace", id),
                }
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
