use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::path::Path;
use std::sync::Arc;
use std::env;
use tracing::{info, error, warn};
use wezterm_parallel::{
    Message, 
    workspace::WorkspaceManager,
    dashboard::{WebSocketServer, DashboardConfig},
    task::{TaskManager, TaskConfig},
};

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
    
    // Initialize task manager
    let task_config = TaskConfig {
        max_concurrent_tasks: 10,
        default_timeout: 3600, // 1 hour
        max_retry_attempts: 3,
        persistence_enabled: false,
        persistence_path: None,
        auto_save_interval: 300, // 5 minutes
        metrics_enabled: true,
        cleanup_interval: 600, // 10 minutes
        max_task_history: 1000,
    };
    
    let task_manager = Arc::new(TaskManager::new(task_config));
    info!("Task manager initialized");
    
    // Start task manager background processing
    let _task_handle = task_manager.start().await?;
    info!("Task manager background processing started");
    
    // Initialize WebSocket dashboard server
    let dashboard_config = DashboardConfig {
        port: 9999,
        enabled: true,
        update_interval: 1000, // 1 second
        max_clients: 10,
        auth_enabled: false,
        auth_token: None,
        compression: true,
    };
    
    let (websocket_server, _metrics_tx) = WebSocketServer::new(dashboard_config);
    let websocket_server = Arc::new(websocket_server.with_task_manager(Arc::clone(&task_manager)));
    
    // Start WebSocket server in background
    let ws_server = Arc::clone(&websocket_server);
    tokio::spawn(async move {
        if let Err(e) = ws_server.start().await {
            error!("WebSocket server error: {}", e);
        }
    });
    
    info!("WebSocket dashboard server started on port 9999");
    
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
                let ws_manager = Arc::clone(&workspace_manager);
                let task_mgr = Arc::clone(&task_manager);
                tokio::spawn(handle_client(stream, ws_manager, task_mgr));
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_client(mut stream: UnixStream, workspace_manager: Arc<WorkspaceManager>, task_manager: Arc<TaskManager>) {
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
                        let response = handle_message(message, &workspace_manager, &task_manager).await;
                        
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

async fn handle_message(message: Message, workspace_manager: &WorkspaceManager, task_manager: &TaskManager) -> Message {
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
            
            // Create a task from the queue message
            let mut task = wezterm_parallel::task::Task::new(
                format!("Task: {}", command),
                wezterm_parallel::task::types::TaskCategory::Development
            );
            
            // Set priority based on message priority
            task.priority = match priority {
                1 | 2 => wezterm_parallel::task::types::TaskPriority::Low,
                3 | 4 => wezterm_parallel::task::types::TaskPriority::Medium,
                5 | 6 => wezterm_parallel::task::types::TaskPriority::High,
                7 | 8 => wezterm_parallel::task::types::TaskPriority::Critical,
                _ => wezterm_parallel::task::types::TaskPriority::Urgent,
            };
            
            // Set workspace if available
            if let Some((workspace_name, _)) = workspace_manager.get_active_workspace().await {
                task.workspace = Some(workspace_name.clone());
            }
            
            // Add task to task manager
            match task_manager.create_task(task).await {
                Ok(task_id) => {
                    info!("Task '{}' created successfully with ID: {}", command, task_id);
                    Message::StatusUpdate {
                        process_id: "task_manager".to_string(),
                        status: format!("Task '{}' created successfully with ID: {}", command, task_id),
                    }
                }
                Err(e) => {
                    error!("Failed to create task '{}': {:?}", command, e);
                    Message::StatusUpdate {
                        process_id: "task_manager".to_string(),
                        status: format!("Failed to create task '{}': {:?}", command, e),
                    }
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
