// WezTerm Multi-Process Development Framework - Real-time Dashboard Backend
// Provides WebSocket server for real-time metrics streaming to WezTerm UI

pub mod server;
pub mod handlers;
pub mod broadcast;
pub mod websocket_server;
pub mod task_board;

pub use websocket_server::WebSocketServer;
pub use task_board::{TaskBoardManager, TaskBoardState};

use crate::metrics::{FrameworkMetrics, SystemMetrics, ProcessMetrics, WorkspaceMetrics};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Dashboard server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// WebSocket server port
    pub port: u16,
    
    /// Enable WebSocket server
    pub enabled: bool,
    
    /// Update interval in milliseconds
    pub update_interval: u64,
    
    /// Maximum connected clients
    pub max_clients: usize,
    
    /// Enable authentication
    pub auth_enabled: bool,
    
    /// Authentication token
    pub auth_token: Option<String>,
    
    /// Enable compression
    pub compression: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            port: 9999,
            enabled: true,
            update_interval: 1000, // 1 second
            max_clients: 10,
            auth_enabled: false,
            auth_token: None,
            compression: true,
        }
    }
}

/// Dashboard state shared across handlers
pub struct DashboardState {
    /// Current framework metrics
    pub framework_metrics: Arc<RwLock<FrameworkMetrics>>,
    
    /// Connected clients
    pub connected_clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
    
    /// Dashboard configuration
    pub config: DashboardConfig,
    
    /// Message broadcast channel
    pub broadcast_tx: tokio::sync::broadcast::Sender<DashboardMessage>,
    
    /// Metrics update channel
    pub metrics_rx: Arc<RwLock<tokio::sync::mpsc::Receiver<MetricsUpdate>>>,
}

/// Client connection information
#[derive(Debug, Clone)]
pub struct ClientInfo {
    /// Client ID
    pub id: String,
    
    /// Connection timestamp
    pub connected_at: u64,
    
    /// Client type (wezterm, web, etc)
    pub client_type: String,
    
    /// Subscribed metrics
    pub subscriptions: Vec<MetricSubscription>,
    
    /// Last activity timestamp
    pub last_activity: u64,
}

/// Metric subscription types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricSubscription {
    /// Subscribe to all metrics
    All,
    
    /// System metrics only
    System,
    
    /// Process metrics for specific workspace
    Process(String),
    
    /// Workspace metrics
    Workspace(String),
    
    /// Alerts only
    Alerts,
    
    /// Performance metrics
    Performance,
}

/// Dashboard message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DashboardMessage {
    /// Metrics update
    MetricsUpdate(MetricsUpdate),
    
    /// Alert notification
    Alert(AlertNotification),
    
    /// System status change
    StatusChange(StatusChange),
    
    /// Client command
    Command(ClientCommand),
    
    /// Heartbeat/ping
    Heartbeat { timestamp: u64 },
    
    /// Error message
    Error { message: String, code: Option<String> },
    
    // Task Management Messages
    
    /// Task board state update
    TaskBoardUpdate {
        board_id: String,
        columns: Vec<TaskColumn>,
        timestamp: u64,
    },
    
    /// Task created/updated/deleted
    TaskUpdate {
        task: serde_json::Value, // Serialized Task
        action: TaskAction,
        timestamp: u64,
    },
    
    /// Task moved between columns
    TaskMoved {
        task_id: String,
        from_column: String,
        to_column: String,
        new_position: usize,
        timestamp: u64,
    },
    
    /// Task progress update
    TaskProgress {
        task_id: String,
        progress: u8,
        timestamp: u64,
    },
    
    /// Task time tracking update
    TaskTimeUpdate {
        task_id: String,
        tracking_data: serde_json::Value, // Serialized tracking session
        timestamp: u64,
    },
    
    /// Task stats/metrics
    TaskStats {
        stats: serde_json::Value, // Serialized task system stats
        timestamp: u64,
    },
}

/// Metrics update payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsUpdate {
    /// Update timestamp
    pub timestamp: u64,
    
    /// System metrics if changed
    pub system: Option<SystemMetrics>,
    
    /// Process metrics updates
    pub processes: Vec<ProcessMetrics>,
    
    /// Workspace metrics updates
    pub workspaces: Vec<WorkspaceMetrics>,
    
    /// Framework summary
    pub framework: Option<FrameworkMetrics>,
    
    /// Update type
    pub update_type: UpdateType,
}

/// Update type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateType {
    /// Full update with all metrics
    Full,
    
    /// Incremental update with changes only
    Incremental,
    
    /// High priority update (alerts, failures)
    Priority,
    
    /// Periodic heartbeat update
    Heartbeat,
}

/// Alert notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotification {
    /// Alert ID
    pub id: String,
    
    /// Alert severity
    pub severity: AlertSeverity,
    
    /// Alert category
    pub category: String,
    
    /// Alert message
    pub message: String,
    
    /// Affected component
    pub component: Option<String>,
    
    /// Alert timestamp
    pub timestamp: u64,
    
    /// Additional details
    pub details: Option<serde_json::Value>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Resolved,
}

/// System status change notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusChange {
    /// Component that changed
    pub component: String,
    
    /// Previous status
    pub previous_status: String,
    
    /// New status
    pub new_status: String,
    
    /// Change reason
    pub reason: Option<String>,
    
    /// Change timestamp
    pub timestamp: u64,
}

/// Client command from dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", content = "params")]
pub enum ClientCommand {
    /// Subscribe to metrics
    Subscribe {
        subscriptions: Vec<MetricSubscription>,
    },
    
    /// Unsubscribe from metrics
    Unsubscribe {
        subscriptions: Vec<MetricSubscription>,
    },
    
    /// Request full update
    RequestFullUpdate,
    
    /// Set update interval
    SetUpdateInterval {
        interval_ms: u64,
    },
    
    /// Execute action
    ExecuteAction {
        action: DashboardAction,
    },
    
    /// Query historical data
    QueryHistory {
        metric_type: String,
        start_time: u64,
        end_time: u64,
        limit: Option<usize>,
    },
}

/// Dashboard actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "params")]
pub enum DashboardAction {
    /// Kill a process
    KillProcess { process_id: String },
    
    /// Restart a process
    RestartProcess { process_id: String },
    
    /// Create workspace
    CreateWorkspace { name: String, template: String },
    
    /// Delete workspace
    DeleteWorkspace { name: String },
    
    /// Clear alerts
    ClearAlerts { category: Option<String> },
    
    /// Reset metrics
    ResetMetrics { metric_type: Option<String> },
    
    /// Trigger garbage collection
    TriggerGC,
    
    /// Export metrics
    ExportMetrics { format: String, path: String },
    
    // Task Management Actions
    
    /// Create new task
    CreateTask { task_data: serde_json::Value },
    
    /// Update existing task
    UpdateTask { task_id: String, task_data: serde_json::Value },
    
    /// Delete task
    DeleteTask { task_id: String },
    
    /// Move task to different column/status
    MoveTask { task_id: String, to_column: String, position: Option<usize> },
    
    /// Start task tracking
    StartTaskTracking { task_id: String },
    
    /// Stop task tracking
    StopTaskTracking { task_id: String },
    
    /// Update task progress
    UpdateTaskProgress { task_id: String, progress: u8 },
}

/// Dashboard WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    /// Message ID for request/response matching
    pub id: Option<String>,
    
    /// Message payload
    pub payload: DashboardMessage,
}

/// Dashboard response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardResponse {
    /// Request ID this responds to
    pub request_id: Option<String>,
    
    /// Success status
    pub success: bool,
    
    /// Response data
    pub data: Option<serde_json::Value>,
    
    /// Error message if failed
    pub error: Option<String>,
}

impl DashboardState {
    /// Create new dashboard state
    pub fn new(config: DashboardConfig) -> (Self, tokio::sync::mpsc::Sender<MetricsUpdate>) {
        let (broadcast_tx, _) = tokio::sync::broadcast::channel(100);
        let (metrics_tx, metrics_rx) = tokio::sync::mpsc::channel(100);
        
        let state = Self {
            framework_metrics: Arc::new(RwLock::new(FrameworkMetrics::new())),
            connected_clients: Arc::new(RwLock::new(HashMap::new())),
            config,
            broadcast_tx,
            metrics_rx: Arc::new(RwLock::new(metrics_rx)),
        };
        
        (state, metrics_tx)
    }
    
    /// Register a new client
    pub async fn register_client(&self, client_info: ClientInfo) {
        let mut clients = self.connected_clients.write().await;
        clients.insert(client_info.id.clone(), client_info);
    }
    
    /// Unregister a client
    pub async fn unregister_client(&self, client_id: &str) {
        let mut clients = self.connected_clients.write().await;
        clients.remove(client_id);
    }
    
    /// Get connected client count
    pub async fn client_count(&self) -> usize {
        self.connected_clients.read().await.len()
    }
    
    /// Update framework metrics
    pub async fn update_metrics(&self, metrics: FrameworkMetrics) {
        let mut current = self.framework_metrics.write().await;
        *current = metrics;
    }
    
    /// Check if client should receive update
    pub async fn should_send_update(&self, client_id: &str, update: &MetricsUpdate) -> bool {
        let clients = self.connected_clients.read().await;
        
        if let Some(client) = clients.get(client_id) {
            // Check if client is subscribed to this type of update
            for subscription in &client.subscriptions {
                match subscription {
                    MetricSubscription::All => return true,
                    MetricSubscription::System if update.system.is_some() => return true,
                    MetricSubscription::Process(workspace) => {
                        if update.processes.iter().any(|p| &p.workspace == workspace) {
                            return true;
                        }
                    }
                    MetricSubscription::Workspace(name) => {
                        if update.workspaces.iter().any(|w| &w.workspace_name == name) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
        
        false
    }
    
    /// Get client subscriptions
    pub async fn get_client_subscriptions(&self, client_id: &str) -> Vec<MetricSubscription> {
        let clients = self.connected_clients.read().await;
        
        clients.get(client_id)
            .map(|c| c.subscriptions.clone())
            .unwrap_or_default()
    }
    
    /// Broadcast message to all clients
    pub fn broadcast(&self, message: DashboardMessage) {
        let _ = self.broadcast_tx.send(message);
    }
    
    /// Get dashboard statistics
    pub async fn get_stats(&self) -> DashboardStats {
        let clients = self.connected_clients.read().await;
        let metrics = self.framework_metrics.read().await;
        
        DashboardStats {
            connected_clients: clients.len(),
            total_workspaces: metrics.total_workspaces as usize,
            total_processes: metrics.total_processes as usize,
            uptime: metrics.framework_uptime,
            last_update: metrics.timestamp,
        }
    }
}

/// Dashboard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub connected_clients: usize,
    pub total_workspaces: usize,
    pub total_processes: usize,
    pub uptime: u64,
    pub last_update: u64,
}

// Task Management Types

/// Task board column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskColumn {
    /// Column ID
    pub id: String,
    
    /// Column title
    pub title: String,
    
    /// Tasks in this column
    pub tasks: Vec<String>, // Task IDs in order
    
    /// Column color/theme
    pub color: Option<String>,
    
    /// Maximum tasks allowed in column
    pub max_tasks: Option<usize>,
    
    /// Column sort order
    pub sort_order: usize,
}

/// Task action types for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskAction {
    Created,
    Updated,
    Deleted,
    StatusChanged,
    ProgressUpdated,
    Moved,
}

/// Task board configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBoardConfig {
    /// Board ID
    pub id: String,
    
    /// Board title
    pub title: String,
    
    /// Column definitions
    pub columns: Vec<TaskColumn>,
    
    /// Auto-refresh interval in milliseconds
    pub refresh_interval: u64,
    
    /// Enable real-time updates
    pub real_time: bool,
    
    /// Board visibility settings
    pub visibility: BoardVisibility,
}

/// Board visibility settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoardVisibility {
    /// Public board
    Public,
    
    /// Private to workspace
    Workspace(String),
    
    /// Private to user
    User(String),
}

impl MetricsUpdate {
    /// Create a full metrics update
    pub fn full(framework: FrameworkMetrics) -> Self {
        let mut processes = Vec::new();
        let mut workspaces = Vec::new();
        
        // Extract all process and workspace metrics
        for workspace in framework.workspaces.values() {
            workspaces.push(workspace.clone());
            for process in workspace.processes.values() {
                processes.push(process.clone());
            }
        }
        
        Self {
            timestamp: framework.timestamp,
            system: Some(framework.system.clone()),
            processes,
            workspaces,
            framework: Some(framework),
            update_type: UpdateType::Full,
        }
    }
    
    /// Create an incremental update
    pub fn incremental(
        system: Option<SystemMetrics>,
        processes: Vec<ProcessMetrics>,
        workspaces: Vec<WorkspaceMetrics>,
    ) -> Self {
        Self {
            timestamp: SystemMetrics::current_timestamp(),
            system,
            processes,
            workspaces,
            framework: None,
            update_type: UpdateType::Incremental,
        }
    }
    
    /// Create a priority update
    pub fn priority(
        processes: Vec<ProcessMetrics>,
        _alert: Option<AlertNotification>,
    ) -> Self {
        Self {
            timestamp: SystemMetrics::current_timestamp(),
            system: None,
            processes,
            workspaces: Vec::new(),
            framework: None,
            update_type: UpdateType::Priority,
        }
    }
}