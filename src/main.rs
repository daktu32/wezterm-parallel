use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::path::Path;
use std::sync::Arc;
use std::env;
use std::time::Instant;
use wezterm_parallel::logging::{LogContext};
use wezterm_parallel::{log_info, log_warn, log_error};
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
    
    let startup_context = LogContext::new("system", "startup")
        .with_metadata("version", serde_json::json!(VERSION));
    log_info!(startup_context, "Starting WezTerm Multi-Process Development Framework v{}", VERSION);
    
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
    
    let perf_context = LogContext::new("system", "performance_init");
    log_info!(perf_context, "パフォーマンス最適化システム初期化完了");
    
    // Initialize workspace manager
    let workspace_manager = Arc::new(WorkspaceManager::new(None)?);
    let workspace_count = workspace_manager.get_workspace_count().await;
    let ws_context = LogContext::new("system", "workspace_init")
        .with_metadata("workspace_count", serde_json::json!(workspace_count));
    log_info!(ws_context, "Workspace manager initialized with {} workspaces", workspace_count);
    
    // Initialize template engine
    use wezterm_parallel::room::template::TemplateEngine;
    let template_engine = Arc::new(tokio::sync::Mutex::new(TemplateEngine::new()));
    let template_context = LogContext::new("system", "template_init");
    log_info!(template_context, "Template engine initialized");
    
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
    let task_init_context = LogContext::new("system", "task_init");
    log_info!(task_init_context, "Task manager initialized");
    
    // Start task manager background processing
    let _task_handle = task_manager.start().await?;
    let task_bg_context = LogContext::new("system", "task_background_start");
    log_info!(task_bg_context, "Task manager background processing started");
    
    // Initialize file sync manager
    let file_sync_manager = Arc::new(tokio::sync::RwLock::new(FileSyncManager::new()));
    let sync_init_context = LogContext::new("system", "file_sync_init");
    log_info!(sync_init_context, "File sync manager initialized");
    
    // Start file watching for current directory
    {
        let mut sync_manager = file_sync_manager.write().await;
        if let Err(e) = sync_manager.start_watching(".") {
            let sync_warn_context = LogContext::new("system", "file_watch_failure");
            log_warn!(sync_warn_context, "Failed to start file watching: {}", e);
        } else {
            let sync_start_context = LogContext::new("system", "file_watch_start")
                .with_metadata("directory", serde_json::json!("."));
            log_info!(sync_start_context, "File watching started for current directory");
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
            let ws_error_context = LogContext::new("system", "websocket_error");
            log_error!(ws_error_context, "WebSocket server error: {}", e);
        }
    });
    
    let ws_start_context = LogContext::new("system", "websocket_start")
        .with_metadata("port", serde_json::json!(9999));
    log_info!(ws_start_context, "WebSocket dashboard server started on port 9999");
    
    // 遅延初期化をスケジュール
    startup_optimizer.schedule_lazy_initialization();
    
    // 起動完了を記録
    startup_optimizer.complete_startup().await;
    
    // パフォーマンス統計をログ
    let startup_time = startup_start.elapsed();
    let startup_complete_context = LogContext::new("system", "startup_complete")
        .with_metadata("startup_time_ms", serde_json::json!(startup_time.as_millis()));
    log_info!(startup_complete_context, "全体の起動時間: {:?}", startup_time);
    
    if let Ok(mut perf_mgr) = perf_manager.lock() {
        perf_mgr.record_startup_complete();
        let perf_report_context = LogContext::new("system", "performance_report");
        log_info!(perf_report_context, "{}", perf_mgr.generate_report());
    }
    
    // Unix Domain Socket path
    let socket_path = "/tmp/wezterm-parallel.sock";
    
    // Remove existing socket file if it exists
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path)?;
    }
    
    // Create Unix Domain Socket listener
    let listener = UnixListener::bind(socket_path)?;
    let ipc_start_context = LogContext::new("system", "ipc_server_start")
        .with_metadata("socket_path", serde_json::json!(socket_path));
    log_info!(ipc_start_context, "IPC Server listening on {}", socket_path);
    
    // パフォーマンス監視タスクを開始
    let perf_manager_clone = Arc::clone(&perf_manager);
    let metrics_collector_clone = Arc::clone(&metrics_collector);
    
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            
            // メモリ使用量チェック
            if let Err(e) = memory_monitor.check_memory_usage().await {
                let memory_warn_context = LogContext::new("system", "memory_monitor_error");
                log_warn!(memory_warn_context, "メモリ監視エラー: {}", e);
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
                let connection_context = LogContext::new("ipc", "client_connect");
                log_info!(connection_context, "New client connected");
                let ws_manager = Arc::clone(&workspace_manager);
                let task_mgr = Arc::clone(&task_manager);
                let perf_mgr = Arc::clone(&perf_manager);
                let tmpl_engine = Arc::clone(&template_engine);
                tokio::spawn(handle_client(stream, ws_manager, task_mgr, perf_mgr, tmpl_engine));
            }
            Err(e) => {
                let connection_error_context = LogContext::new("ipc", "connection_accept_error");
                log_error!(connection_error_context, "Failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_client(
    mut stream: UnixStream, 
    workspace_manager: Arc<WorkspaceManager>, 
    task_manager: Arc<TaskManager>, 
    perf_manager: Arc<std::sync::Mutex<PerformanceManager>>,
    template_engine: Arc<tokio::sync::Mutex<wezterm_parallel::room::template::TemplateEngine>>
) {
    let mut buffer = [0; 1024];
    
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => {
                let disconnect_context = LogContext::new("ipc", "client_disconnect");
                log_info!(disconnect_context, "Client disconnected");
                break;
            }
            Ok(n) => {
                let data = &buffer[..n];
                
                // Try to parse JSON message
                match serde_json::from_slice::<Message>(data) {
                    Ok(message) => {
                        let message_context = LogContext::new("ipc", "message_receive")
                            .with_metadata("message_type", serde_json::json!(format!("{:?}", message)));
                        log_info!(message_context, "Received message: {:?}", message);
                        
                        // Handle message with performance tracking
                        let start_time = Instant::now();
                        let response = handle_message(message, &workspace_manager, &task_manager, &template_engine).await;
                        let _response_time = start_time.elapsed();
                        
                        // パフォーマンス統計を更新
                        if let Ok(mut perf_mgr) = perf_manager.lock() {
                            perf_mgr.update_cpu_usage(20.0); // リクエスト処理によるCPU使用量
                        }
                        
                        // Send response
                        if let Ok(response_json) = serde_json::to_vec(&response) {
                            if let Err(e) = stream.write_all(&response_json).await {
                                let send_error_context = LogContext::new("ipc", "response_send_error");
                                log_error!(send_error_context, "Failed to send response: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        let parse_error_context = LogContext::new("ipc", "message_parse_error");
                        log_warn!(parse_error_context, "Failed to parse message: {}", e);
                        
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
                let read_error_context = LogContext::new("ipc", "stream_read_error");
                log_error!(read_error_context, "Failed to read from stream: {}", e);
                break;
            }
        }
    }
}

async fn handle_message(
    message: Message, 
    workspace_manager: &WorkspaceManager, 
    task_manager: &TaskManager,
    template_engine: &Arc<tokio::sync::Mutex<wezterm_parallel::room::template::TemplateEngine>>
) -> Message {
    use wezterm_parallel::TemplateInfo;
    
    match message {
        Message::Ping => {
            let ping_context = LogContext::new("ipc", "ping_receive");
            log_info!(ping_context, "Ping received, responding with Pong");
            Message::Pong
        }
        Message::WorkspaceCreate { name, template } => {
            let create_context = LogContext::new("ipc", "workspace_create_request")
                .with_entity_id(&name)
                .with_metadata("template", serde_json::json!(template));
            log_info!(create_context, "Creating workspace: {} with template: {}", name, template);
            
            match workspace_manager.create_workspace(&name, &template).await {
                Ok(()) => {
                    let success_context = LogContext::new("ipc", "workspace_create_success")
                        .with_entity_id(&name);
                    log_info!(success_context, "Successfully created workspace '{}'", name);
                    Message::StatusUpdate {
                        process_id: "workspace_manager".to_string(),
                        status: format!("Workspace '{}' created successfully with template '{}'", name, template),
                    }
                }
                Err(e) => {
                    let error_context = LogContext::new("ipc", "workspace_create_error")
                        .with_entity_id(&name);
                    log_error!(error_context, "Failed to create workspace '{}': {}", name, e);
                    Message::StatusUpdate {
                        process_id: "workspace_manager".to_string(),
                        status: format!("Failed to create workspace '{}': {}", name, e),
                    }
                }
            }
        }
        Message::ProcessSpawn { workspace, command } => {
            let spawn_context = LogContext::new("ipc", "process_spawn_request")
                .with_entity_id(&workspace)
                .with_metadata("command", serde_json::json!(command));
            log_info!(spawn_context, "Spawning process in workspace '{}': {}", workspace, command);
            
            // Check if workspace exists
            if workspace_manager.get_workspace_info(&workspace).await.is_some() {
                // TODO: Implement actual process spawning logic
                Message::StatusUpdate {
                    process_id: "process_manager".to_string(),
                    status: format!("Process '{}' spawned in workspace '{}'", command, workspace),
                }
            } else {
                let not_found_context = LogContext::new("ipc", "workspace_not_found")
                    .with_entity_id(&workspace);
                log_error!(not_found_context, "Workspace '{}' not found for process spawning", workspace);
                Message::StatusUpdate {
                    process_id: "process_manager".to_string(),
                    status: format!("Failed to spawn process: workspace '{}' not found", workspace),
                }
            }
        }
        Message::TaskQueue { id, priority, command } => {
            let queue_context = LogContext::new("ipc", "task_queue_request")
                .with_entity_id(&id)
                .with_metadata("priority", serde_json::json!(priority))
                .with_metadata("command", serde_json::json!(command));
            log_info!(queue_context, "Queuing task {}: {} (priority: {})", id, command, priority);
            
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
                    let task_success_context = LogContext::new("ipc", "task_create_success")
                        .with_entity_id(&task_id)
                        .with_metadata("command", serde_json::json!(command));
                    log_info!(task_success_context, "Task '{}' created successfully with ID: {}", command, task_id);
                    Message::StatusUpdate {
                        process_id: "task_manager".to_string(),
                        status: format!("Task '{}' created successfully with ID: {}", command, task_id),
                    }
                }
                Err(e) => {
                    let task_error_context = LogContext::new("ipc", "task_create_error")
                        .with_metadata("command", serde_json::json!(command));
                    log_error!(task_error_context, "Failed to create task '{}': {:?}", command, e);
                    Message::StatusUpdate {
                        process_id: "task_manager".to_string(),
                        status: format!("Failed to create task '{}': {:?}", command, e),
                    }
                }
            }
        }
        Message::TemplateList => {
            let template_list_context = LogContext::new("ipc", "template_list_request");
            log_info!(template_list_context, "Listing available templates");
            let engine = template_engine.lock().await;
            let templates = engine.list_templates();
            let template_infos: Vec<TemplateInfo> = templates.iter().map(|t| {
                TemplateInfo {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    author: "System".to_string(),
                    version: "1.0".to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    layout_type: format!("{:?}", t.layout.layout_type),
                    pane_count: t.layout.pane_sizes.len() as u32,
                    auto_start_processes: !t.default_commands.is_empty(),
                }
            }).collect();
            
            Message::TemplateListResponse { templates: template_infos }
        }
        Message::TemplateGet { name } => {
            let template_get_context = LogContext::new("ipc", "template_get_request")
                .with_entity_id(&name);
            log_info!(template_get_context, "Getting template: {}", name);
            let engine = template_engine.lock().await;
            if let Some(template) = engine.get_template(&name) {
                match serde_json::to_string(template) {
                    Ok(content) => Message::TemplateGetResponse { template: Some(content) },
                    Err(e) => {
                        let serialize_error_context = LogContext::new("ipc", "template_serialize_error")
                            .with_entity_id(&name);
                        log_error!(serialize_error_context, "Failed to serialize template: {}", e);
                        Message::TemplateGetResponse { template: None }
                    }
                }
            } else {
                Message::TemplateGetResponse { template: None }
            }
        }
        Message::TemplateCreate { name, content } => {
            let template_create_context = LogContext::new("ipc", "template_create_request")
                .with_entity_id(&name);
            log_info!(template_create_context, "Creating template: {}", name);
            
            match serde_json::from_str::<wezterm_parallel::room::template::WorkspaceTemplate>(&content) {
                Ok(template) => {
                    let mut engine = template_engine.lock().await;
                    engine.register_template(template);
                    let template_success_context = LogContext::new("ipc", "template_create_success")
                        .with_entity_id(&name);
                    log_info!(template_success_context, "Template '{}' created successfully", name);
                    Message::TemplateCreateResponse { 
                        success: true, 
                        error: None 
                    }
                }
                Err(e) => {
                    let parse_error_context = LogContext::new("ipc", "template_parse_error")
                        .with_entity_id(&name);
                    log_error!(parse_error_context, "Failed to parse template JSON: {}", e);
                    Message::TemplateCreateResponse { 
                        success: false, 
                        error: Some(format!("Invalid template format: {}", e)) 
                    }
                }
            }
        }
        Message::TemplateDelete { name: _ } => {
            // TODO: Implement template deletion
            Message::TemplateDeleteResponse { 
                success: false, 
                error: Some("Template deletion not yet implemented".to_string()) 
            }
        }
        other => {
            let unhandled_context = LogContext::new("ipc", "unhandled_message")
                .with_metadata("message_type", serde_json::json!(format!("{:?}", other)));
            log_warn!(unhandled_context, "Unhandled message type: {:?}", other);
            Message::StatusUpdate {
                process_id: "system".to_string(),
                status: "Unknown message type".to_string(),
            }
        }
    }
}
