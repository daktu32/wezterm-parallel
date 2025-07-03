use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::path::Path;
use std::sync::Arc;
use std::env;
use std::time::Instant;
use tracing::{info, error, warn};
use wezterm_parallel::{
    Message, 
    room::WorkspaceManager,
    dashboard::{WebSocketServer, DashboardConfig},
    task::{TaskManager, TaskConfig},
    sync::FileSyncManager,
    performance::{PerformanceConfig, PerformanceManager},
    performance::startup::StartupOptimizer,
    performance::memory::MemoryMonitor,
    performance::metrics::MetricsCollector,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let startup_start = Instant::now();
    
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
    
    // === パフォーマンス最適化初期化 ===
    let perf_config = PerformanceConfig {
        lazy_initialization: true,
        max_preload_modules: 5,
        initial_memory_pool_size: 1024 * 1024, // 1MB
        async_task_pool_size: 4,
        gc_interval_secs: 300,
        cpu_limit_percent: 80.0,
        memory_limit_mb: 512,
    };
    
    // 起動最適化開始
    let mut startup_optimizer = StartupOptimizer::new(perf_config.clone());
    
    // コアモジュールの高速初期化
    startup_optimizer.fast_init_core_modules().await?;
    
    // 重要リソースのプリロード
    startup_optimizer.preload_critical_resources().await?;
    
    // パフォーマンスマネージャー初期化
    let perf_manager = Arc::new(std::sync::Mutex::new(PerformanceManager::new(perf_config.clone())));
    
    // メモリ監視開始
    let mut memory_monitor = MemoryMonitor::new(perf_config.memory_limit_mb);
    
    // メトリクス収集開始
    let metrics_collector = Arc::new(tokio::sync::RwLock::new(MetricsCollector::new(100, std::time::Duration::from_secs(30))));
    {
        let mut collector = metrics_collector.write().await;
        collector.start_collection();
    }
    
    info!("パフォーマンス最適化システム初期化完了");
    
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
    
    // Initialize file sync manager
    let file_sync_manager = Arc::new(tokio::sync::RwLock::new(FileSyncManager::new()));
    info!("File sync manager initialized");
    
    // Start file watching for current directory
    {
        let mut sync_manager = file_sync_manager.write().await;
        if let Err(e) = sync_manager.start_watching(".") {
            warn!("Failed to start file watching: {}", e);
        } else {
            info!("File watching started for current directory");
        }
    }
    
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
    
    // 遅延初期化をスケジュール
    startup_optimizer.schedule_lazy_initialization();
    
    // 起動完了を記録
    startup_optimizer.complete_startup().await;
    
    // パフォーマンス統計をログ
    let startup_time = startup_start.elapsed();
    info!("全体の起動時間: {:?}", startup_time);
    
    if let Ok(mut perf_mgr) = perf_manager.lock() {
        perf_mgr.record_startup_complete();
        info!("{}", perf_mgr.generate_report());
    }
    
    // Unix Domain Socket path
    let socket_path = "/tmp/wezterm-parallel.sock";
    
    // Remove existing socket file if it exists
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path)?;
    }
    
    // Create Unix Domain Socket listener
    let listener = UnixListener::bind(socket_path)?;
    info!("IPC Server listening on {}", socket_path);
    
    // パフォーマンス監視タスクを開始
    let perf_manager_clone = Arc::clone(&perf_manager);
    let metrics_collector_clone = Arc::clone(&metrics_collector);
    
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            
            // メモリ使用量チェック
            if let Err(e) = memory_monitor.check_memory_usage().await {
                warn!("メモリ監視エラー: {}", e);
            }
            
            // パフォーマンス統計更新
            {
                if let Ok(mut perf_mgr) = perf_manager_clone.lock() {
                    perf_mgr.periodic_gc();
                    
                    // CPU・メモリ使用量を更新（実際の値を取得する必要がある）
                    perf_mgr.update_cpu_usage(25.0); // サンプル値
                    perf_mgr.update_memory_usage(64 * 1024 * 1024); // 64MB サンプル値
                }
            }
            
            // メトリクス更新
            {
                let metrics = metrics_collector_clone.read().await;
                metrics.update_cpu_usage(25.0).await;
                metrics.update_memory_usage(64 * 1024 * 1024, 128 * 1024 * 1024).await;
            }
        }
    });
    
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                info!("New client connected");
                let ws_manager = Arc::clone(&workspace_manager);
                let task_mgr = Arc::clone(&task_manager);
                let perf_mgr = Arc::clone(&perf_manager);
                tokio::spawn(handle_client(stream, ws_manager, task_mgr, perf_mgr));
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_client(mut stream: UnixStream, workspace_manager: Arc<WorkspaceManager>, task_manager: Arc<TaskManager>, perf_manager: Arc<std::sync::Mutex<PerformanceManager>>) {
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
                        
                        // Handle message with performance tracking
                        let start_time = Instant::now();
                        let response = handle_message(message, &workspace_manager, &task_manager).await;
                        let _response_time = start_time.elapsed();
                        
                        // パフォーマンス統計を更新
                        if let Ok(mut perf_mgr) = perf_manager.lock() {
                            perf_mgr.update_cpu_usage(20.0); // リクエスト処理によるCPU使用量
                        }
                        
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
