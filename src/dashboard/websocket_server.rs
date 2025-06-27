// WezTerm Multi-Process Development Framework - Enhanced WebSocket Server
// Provides real-time metrics streaming to WezTerm Lua clients

use super::{DashboardState, DashboardConfig, DashboardMessage, ClientInfo, MetricSubscription, MetricsUpdate};
use super::task_board::{TaskBoardManager, TaskBoardState};
use crate::metrics::FrameworkMetrics;
use crate::task::TaskManager;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use serde_json;

pub struct WebSocketServer {
    state: Arc<DashboardState>,
    config: DashboardConfig,
    task_board_manager: Option<Arc<TaskBoardManager>>,
}

impl WebSocketServer {
    pub fn new(config: DashboardConfig) -> (Self, tokio::sync::mpsc::Sender<MetricsUpdate>) {
        let (state, metrics_tx) = DashboardState::new(config.clone());
        
        let server = Self {
            state: Arc::new(state),
            config,
            task_board_manager: None,
        };
        
        (server, metrics_tx)
    }

    /// Set task manager and enable task board functionality
    pub fn with_task_manager(mut self, task_manager: Arc<TaskManager>) -> Self {
        let task_board_manager = TaskBoardManager::new(
            task_manager,
            self.state.broadcast_tx.clone(),
        );
        self.task_board_manager = Some(Arc::new(task_board_manager));
        self
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.enabled {
            info!("WebSocket dashboard server is disabled");
            return Ok(());
        }

        // Initialize task board if available
        if let Some(ref task_board_manager) = self.task_board_manager {
            task_board_manager.initialize().await?;
            info!("Task board manager initialized");
        }
        
        let addr = format!("127.0.0.1:{}", self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("Dashboard WebSocket server listening on {}", addr);
        
        // Start background tasks
        let metrics_task = self.start_metrics_broadcaster().await;
        let heartbeat_task = self.start_heartbeat_task().await;
        
        // Accept connections
        while let Ok((stream, client_addr)) = listener.accept().await {
            if self.state.client_count().await >= self.config.max_clients {
                warn!("Maximum client limit reached, rejecting connection from {}", client_addr);
                continue;
            }
            
            info!("New WebSocket connection from {}", client_addr);
            
            let state = Arc::clone(&self.state);
            let config = self.config.clone();
            let task_board_manager = self.task_board_manager.clone();
            
            tokio::spawn(async move {
                if let Err(e) = handle_client_connection(stream, state, config, task_board_manager).await {
                    error!("Client connection error: {}", e);
                }
            });
        }
        
        // Clean up tasks
        metrics_task.abort();
        heartbeat_task.abort();
        
        Ok(())
    }
    
    pub fn get_state(&self) -> Arc<DashboardState> {
        Arc::clone(&self.state)
    }
    
    /// Start metrics broadcaster task
    async fn start_metrics_broadcaster(&self) -> tokio::task::JoinHandle<()> {
        let state = Arc::clone(&self.state);
        let update_interval = self.config.update_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(update_interval));
            
            loop {
                interval.tick().await;
                
                // Check for metrics updates
                let mut metrics_rx = state.metrics_rx.write().await;
                while let Ok(update) = metrics_rx.try_recv() {
                    let message = DashboardMessage::MetricsUpdate(update);
                    state.broadcast(message);
                }
            }
        })
    }
    
    /// Start heartbeat task
    async fn start_heartbeat_task(&self) -> tokio::task::JoinHandle<()> {
        let state = Arc::clone(&self.state);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let heartbeat = DashboardMessage::Heartbeat {
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                };
                
                state.broadcast(heartbeat);
            }
        })
    }
    
    /// Update framework metrics
    pub async fn update_metrics(&self, metrics: FrameworkMetrics) {
        self.state.update_metrics(metrics).await;
    }
    
    /// Send alert to dashboard
    pub async fn send_alert(&self, alert: super::AlertNotification) {
        let message = DashboardMessage::Alert(alert);
        self.state.broadcast(message);
    }
    
    /// Get dashboard statistics
    pub async fn get_stats(&self) -> super::DashboardStats {
        self.state.get_stats().await
    }
}

async fn handle_client_connection(
    stream: tokio::net::TcpStream,
    state: Arc<DashboardState>,
    _config: DashboardConfig,
    task_board_manager: Option<Arc<TaskBoardManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    let client_id = Uuid::new_v4().to_string();
    
    // Register client
    let client_info = ClientInfo {
        id: client_id.clone(),
        connected_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        client_type: "wezterm".to_string(),
        subscriptions: vec![MetricSubscription::All], // Default subscription
        last_activity: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    state.register_client(client_info).await;
    info!("Client {} registered", client_id);
    
    // Create channels for outgoing messages
    let (outgoing_tx, mut outgoing_rx) = tokio::sync::mpsc::channel::<Message>(100);
    
    // Create broadcast receiver for this client
    let mut broadcast_rx = state.broadcast_tx.subscribe();
    
    // Spawn task to handle outgoing messages
    let client_id_out = client_id.clone();
    let state_out = Arc::clone(&state);
    let outgoing_sender = outgoing_tx.clone();
    let broadcast_task = tokio::spawn(async move {
        while let Ok(message) = broadcast_rx.recv().await {
            // Check if client should receive this message
            let should_send = match &message {
                DashboardMessage::MetricsUpdate(update) => {
                    state_out.should_send_update(&client_id_out, update).await
                },
                _ => true, // Send non-metrics messages to all clients
            };
            
            if should_send {
                let ws_message = super::WebSocketMessage {
                    id: None,
                    payload: message,
                };
                
                if let Ok(json) = serde_json::to_string(&ws_message) {
                    if let Err(_) = outgoing_sender.send(Message::Text(json)).await {
                        break; // Channel closed
                    }
                } else {
                    error!("Failed to serialize message for client {}", client_id_out);
                }
            }
        }
    });
    
    // Spawn task to send outgoing messages
    let sender_task = tokio::spawn(async move {
        while let Some(message) = outgoing_rx.recv().await {
            if let Err(e) = ws_sender.send(message).await {
                error!("Failed to send message: {}", e);
                break;
            }
        }
    });
    
    // Handle incoming messages
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!("Received message from client {}: {}", client_id, text);
                
                // Parse and handle client command
                if let Ok(ws_msg) = serde_json::from_str::<super::WebSocketMessage>(&text) {
                    if let Err(e) = handle_client_message(&client_id, ws_msg, &state, &outgoing_tx, &task_board_manager).await {
                        error!("Error handling client message: {}", e);
                    }
                } else {
                    warn!("Invalid message format from client {}", client_id);
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client {} sent close message", client_id);
                break;
            }
            Ok(Message::Ping(data)) => {
                if let Err(_) = outgoing_tx.send(Message::Pong(data)).await {
                    error!("Failed to send pong to client {}", client_id);
                    break;
                }
            }
            Ok(_) => {
                // Ignore other message types
            }
            Err(e) => {
                error!("WebSocket error for client {}: {}", client_id, e);
                break;
            }
        }
    }
    
    // Clean up
    broadcast_task.abort();
    sender_task.abort();
    state.unregister_client(&client_id).await;
    info!("Client {} disconnected", client_id);
    
    Ok(())
}

async fn handle_client_message(
    client_id: &str,
    ws_msg: super::WebSocketMessage,
    state: &Arc<DashboardState>,
    outgoing_tx: &tokio::sync::mpsc::Sender<Message>,
    task_board_manager: &Option<Arc<TaskBoardManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    match ws_msg.payload {
        DashboardMessage::Command(command) => {
            match command {
                super::ClientCommand::Subscribe { subscriptions } => {
                    // Update client subscriptions
                    let mut clients = state.connected_clients.write().await;
                    if let Some(client) = clients.get_mut(client_id) {
                        client.subscriptions = subscriptions;
                        client.last_activity = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    }
                    
                    // Send success response
                    let _response = super::DashboardResponse {
                        request_id: ws_msg.id,
                        success: true,
                        data: None,
                        error: None,
                    };
                    
                    // For now, just acknowledge the subscription
                    debug!("Client {} updated subscriptions", client_id);
                }
                super::ClientCommand::RequestFullUpdate => {
                    // Send full metrics update
                    let metrics = state.framework_metrics.read().await;
                    let update = MetricsUpdate::full(metrics.clone());
                    
                    let ws_message = super::WebSocketMessage {
                        id: ws_msg.id,
                        payload: DashboardMessage::MetricsUpdate(update),
                    };
                    
                    if let Ok(json) = serde_json::to_string(&ws_message) {
                        outgoing_tx.send(Message::Text(json)).await?;
                    }
                }
                super::ClientCommand::ExecuteAction { action } => {
                    if let Some(task_manager) = task_board_manager {
                        handle_task_action(client_id, action, task_manager, outgoing_tx, &ws_msg.id).await?;
                    } else {
                        error!("Task board manager not available for client {}", client_id);
                    }
                }
                _ => {
                    // Handle other commands as needed
                    debug!("Unhandled command from client {}: {:?}", client_id, command);
                }
            }
        }
        DashboardMessage::Heartbeat { .. } => {
            // Update client activity
            let mut clients = state.connected_clients.write().await;
            if let Some(client) = clients.get_mut(client_id) {
                client.last_activity = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            }
        }
        _ => {
            debug!("Unhandled message type from client {}", client_id);
        }
    }
    
    Ok(())
}

/// Handle task management actions
async fn handle_task_action(
    client_id: &str,
    action: super::DashboardAction,
    task_board_manager: &Arc<TaskBoardManager>,
    outgoing_tx: &tokio::sync::mpsc::Sender<Message>,
    request_id: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = match action {
        super::DashboardAction::CreateTask { task_data } => {
            match task_board_manager.create_task_from_dashboard(task_data).await {
                Ok(task_id) => {
                    info!("Created task {} for client {}", task_id, client_id);
                    Ok(serde_json::to_value(task_id)?)
                }
                Err(e) => {
                    error!("Failed to create task for client {}: {}", client_id, e);
                    Err(e)
                }
            }
        }
        super::DashboardAction::UpdateTask { task_id, task_data } => {
            match task_board_manager.update_task_from_dashboard(&task_id, task_data).await {
                Ok(_) => {
                    info!("Updated task {} for client {}", task_id, client_id);
                    Ok(serde_json::Value::Bool(true))
                }
                Err(e) => {
                    error!("Failed to update task {} for client {}: {}", task_id, client_id, e);
                    Err(e)
                }
            }
        }
        super::DashboardAction::DeleteTask { task_id } => {
            match task_board_manager.delete_task_from_dashboard(&task_id).await {
                Ok(_) => {
                    info!("Deleted task {} for client {}", task_id, client_id);
                    Ok(serde_json::Value::Bool(true))
                }
                Err(e) => {
                    error!("Failed to delete task {} for client {}: {}", task_id, client_id, e);
                    Err(e)
                }
            }
        }
        super::DashboardAction::MoveTask { task_id, to_column, position } => {
            match task_board_manager.move_task("default", &task_id, &to_column, position).await {
                Ok(_) => {
                    info!("Moved task {} to {} for client {}", task_id, to_column, client_id);
                    Ok(serde_json::Value::Bool(true))
                }
                Err(e) => {
                    error!("Failed to move task {} for client {}: {}", task_id, client_id, e);
                    Err(e)
                }
            }
        }
        super::DashboardAction::UpdateTaskProgress { task_id, progress } => {
            match task_board_manager.update_task_progress(&task_id, progress).await {
                Ok(_) => {
                    info!("Updated task {} progress to {}% for client {}", task_id, progress, client_id);
                    Ok(serde_json::Value::Bool(true))
                }
                Err(e) => {
                    error!("Failed to update task {} progress for client {}: {}", task_id, client_id, e);
                    Err(e)
                }
            }
        }
        _ => {
            debug!("Unhandled task action for client {}: {:?}", client_id, action);
            Ok(serde_json::Value::Null)
        }
    };

    // Send response back to client
    let (success, data, error) = match &result {
        Ok(value) => (true, Some(value.clone()), None),
        Err(e) => (false, None, Some(e.to_string())),
    };

    let response = super::DashboardResponse {
        request_id: request_id.clone(),
        success,
        data,
        error,
    };

    if let Ok(json) = serde_json::to_string(&response) {
        outgoing_tx.send(Message::Text(json)).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_websocket_server_creation() {
        let config = DashboardConfig {
            port: 9996,
            enabled: true,
            ..Default::default()
        };
        
        let (server, _metrics_tx) = WebSocketServer::new(config);
        let stats = server.get_stats().await;
        
        assert_eq!(stats.connected_clients, 0);
        assert_eq!(stats.total_workspaces, 0);
    }

    #[tokio::test]
    async fn test_metrics_update() {
        let config = DashboardConfig {
            port: 9995,
            enabled: true,
            ..Default::default()
        };
        
        let (server, metrics_tx) = WebSocketServer::new(config);
        
        // Send a metrics update
        let update = MetricsUpdate {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            system: Some(crate::metrics::SystemMetrics::new()),
            processes: Vec::new(),
            workspaces: Vec::new(),
            framework: Some(FrameworkMetrics::new()),
            update_type: super::super::UpdateType::Full,
        };
        
        let result = metrics_tx.send(update).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_disabled_server() {
        let config = DashboardConfig {
            enabled: false,
            ..Default::default()
        };
        
        let (server, _metrics_tx) = WebSocketServer::new(config);
        
        // Should return immediately without error
        let result = timeout(Duration::from_millis(100), server.start()).await;
        assert!(result.is_ok());
    }
}