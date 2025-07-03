// WezTerm Multi-Process Development Framework - Health Check System
// Provides comprehensive health monitoring for all system components

use super::{HealthCheck, HealthStatus, ComponentHealth};
use crate::room::WorkspaceManager;
use crate::task::TaskManager;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, error, debug};

/// Health check manager
pub struct HealthCheckManager {
    /// Workspace manager reference
    workspace_manager: Arc<WorkspaceManager>,
    
    /// Task manager reference
    task_manager: Arc<TaskManager>,
    
    /// Health check interval
    check_interval: Duration,
    
    /// Component health history
    health_history: Arc<tokio::sync::RwLock<Vec<HealthCheck>>>,
    
    /// Last successful checks by component
    last_success: Arc<tokio::sync::RwLock<HashMap<String, u64>>>,
    
    /// Failure counts by component
    failure_counts: Arc<tokio::sync::RwLock<HashMap<String, u32>>>,
}

/// Health check result for individual components
#[derive(Debug, Clone)]
pub struct ComponentCheckResult {
    pub component_name: String,
    pub status: HealthStatus,
    pub message: String,
    pub response_time_ms: u64,
    pub details: HashMap<String, serde_json::Value>,
}

impl HealthCheckManager {
    /// Create new health check manager
    pub fn new(
        workspace_manager: Arc<WorkspaceManager>,
        task_manager: Arc<TaskManager>,
        check_interval: Duration,
    ) -> Self {
        Self {
            workspace_manager,
            task_manager,
            check_interval,
            health_history: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            last_success: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            failure_counts: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
    
    /// Start health check monitoring
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting health check manager with interval: {:?}", self.check_interval);
        
        loop {
            let check_start = Instant::now();
            
            match self.perform_health_check().await {
                Ok(health_check) => {
                    debug!("Health check completed in {:?}", check_start.elapsed());
                    self.update_health_history(health_check).await;
                }
                Err(e) => {
                    error!("Health check failed: {}", e);
                }
            }
            
            sleep(self.check_interval).await;
        }
    }
    
    /// Perform comprehensive health check
    pub async fn perform_health_check(&self) -> Result<HealthCheck, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let timestamp = current_timestamp();
        
        debug!("Starting comprehensive health check");
        
        let mut components = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;
        
        // Check workspace manager
        let workspace_check = self.check_workspace_manager().await;
        if workspace_check.status != HealthStatus::Healthy {
            overall_status = worst_status(overall_status, workspace_check.status.clone());
        }
        components.insert("workspace_manager".to_string(), self.create_component_health(&workspace_check).await);
        
        // Check task manager
        let task_check = self.check_task_manager().await;
        if task_check.status != HealthStatus::Healthy {
            overall_status = worst_status(overall_status, task_check.status.clone());
        }
        components.insert("task_manager".to_string(), self.create_component_health(&task_check).await);
        
        // Check IPC system
        let ipc_check = self.check_ipc_system().await;
        if ipc_check.status != HealthStatus::Healthy {
            overall_status = worst_status(overall_status, ipc_check.status.clone());
        }
        components.insert("ipc_system".to_string(), self.create_component_health(&ipc_check).await);
        
        // Check WebSocket server
        let websocket_check = self.check_websocket_server().await;
        if websocket_check.status != HealthStatus::Healthy {
            overall_status = worst_status(overall_status, websocket_check.status.clone());
        }
        components.insert("websocket_server".to_string(), self.create_component_health(&websocket_check).await);
        
        // Check file system
        let filesystem_check = self.check_file_system().await;
        if filesystem_check.status != HealthStatus::Healthy {
            overall_status = worst_status(overall_status, filesystem_check.status.clone());
        }
        components.insert("file_system".to_string(), self.create_component_health(&filesystem_check).await);
        
        // Check database/persistence
        let persistence_check = self.check_persistence_layer().await;
        if persistence_check.status != HealthStatus::Healthy {
            overall_status = worst_status(overall_status, persistence_check.status.clone());
        }
        components.insert("persistence".to_string(), self.create_component_health(&persistence_check).await);
        
        let check_duration_ms = start_time.elapsed().as_millis() as u64;
        
        let health_check = HealthCheck {
            timestamp,
            overall_status,
            components,
            check_duration_ms,
        };
        
        info!("Health check completed: {} ({}ms)", 
              health_status_to_string(&health_check.overall_status), check_duration_ms);
        
        Ok(health_check)
    }
    
    /// Check workspace manager health
    async fn check_workspace_manager(&self) -> ComponentCheckResult {
        let start_time = Instant::now();
        let component_name = "workspace_manager".to_string();
        
        match self.workspace_manager.get_workspace_count().await.try_into() as Result<u64, _> {
            Ok(count) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                
                // Check if workspace manager is responsive and has reasonable workspace count
                if count > 100 {
                    ComponentCheckResult {
                        component_name,
                        status: HealthStatus::Degraded,
                        message: format!("High workspace count: {}", count),
                        response_time_ms: response_time,
                        details: {
                            let mut details = HashMap::new();
                            details.insert("workspace_count".to_string(), serde_json::Value::Number(count.into()));
                            details
                        },
                    }
                } else {
                    ComponentCheckResult {
                        component_name,
                        status: HealthStatus::Healthy,
                        message: format!("Workspace manager healthy with {} workspaces", count),
                        response_time_ms: response_time,
                        details: {
                            let mut details = HashMap::new();
                            details.insert("workspace_count".to_string(), serde_json::Value::Number(count.into()));
                            details
                        },
                    }
                }
            }
            Err(_) => {
                ComponentCheckResult {
                    component_name,
                    status: HealthStatus::Unhealthy,
                    message: "Failed to get workspace count".to_string(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    details: HashMap::new(),
                }
            }
        }
    }
    
    /// Check task manager health
    async fn check_task_manager(&self) -> ComponentCheckResult {
        let start_time = Instant::now();
        let component_name = "task_manager".to_string();
        
        // Check if task manager is responsive
        let count = self.task_manager.get_task_count().await;
        let response_time = start_time.elapsed().as_millis() as u64;
        
        // Check task queue health
        let queue = self.task_manager.get_queue();
        let queue_size = queue.get_queue_size().await;
        
        let status = if queue_size > 1000 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };
        
        ComponentCheckResult {
            component_name,
            status,
            message: format!("Task manager healthy with {} tasks, queue size: {}", count, queue_size),
            response_time_ms: response_time,
            details: {
                let mut details = HashMap::new();
                details.insert("task_count".to_string(), serde_json::Value::Number(count.into()));
                details.insert("queue_size".to_string(), serde_json::Value::Number(queue_size.into()));
                details
            },
        }
    }
    
    /// Check IPC system health
    async fn check_ipc_system(&self) -> ComponentCheckResult {
        let start_time = Instant::now();
        let component_name = "ipc_system".to_string();
        
        // Check if IPC socket exists and is accessible
        let socket_path = "/tmp/wezterm-parallel.sock";
        
        match tokio::fs::metadata(socket_path).await {
            Ok(metadata) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                
                if metadata.len() == 0 {
                    ComponentCheckResult {
                        component_name,
                        status: HealthStatus::Healthy,
                        message: "IPC socket is accessible".to_string(),
                        response_time_ms: response_time,
                        details: {
                            let mut details = HashMap::new();
                            details.insert("socket_path".to_string(), serde_json::Value::String(socket_path.to_string()));
                            details.insert("socket_exists".to_string(), serde_json::Value::Bool(true));
                            details
                        },
                    }
                } else {
                    ComponentCheckResult {
                        component_name,
                        status: HealthStatus::Degraded,
                        message: "IPC socket has unexpected size".to_string(),
                        response_time_ms: response_time,
                        details: {
                            let mut details = HashMap::new();
                            details.insert("socket_path".to_string(), serde_json::Value::String(socket_path.to_string()));
                            details.insert("socket_size".to_string(), serde_json::Value::Number(metadata.len().into()));
                            details
                        },
                    }
                }
            }
            Err(_) => {
                ComponentCheckResult {
                    component_name,
                    status: HealthStatus::Unhealthy,
                    message: "IPC socket not accessible".to_string(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    details: {
                        let mut details = HashMap::new();
                        details.insert("socket_path".to_string(), serde_json::Value::String(socket_path.to_string()));
                        details.insert("socket_exists".to_string(), serde_json::Value::Bool(false));
                        details
                    },
                }
            }
        }
    }
    
    /// Check WebSocket server health
    async fn check_websocket_server(&self) -> ComponentCheckResult {
        let start_time = Instant::now();
        let component_name = "websocket_server".to_string();
        
        // Try to connect to WebSocket server
        match tokio::net::TcpStream::connect("127.0.0.1:9999").await {
            Ok(_) => {
                ComponentCheckResult {
                    component_name,
                    status: HealthStatus::Healthy,
                    message: "WebSocket server is listening".to_string(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    details: {
                        let mut details = HashMap::new();
                        details.insert("port".to_string(), serde_json::Value::Number(9999.into()));
                        details.insert("listening".to_string(), serde_json::Value::Bool(true));
                        details
                    },
                }
            }
            Err(_) => {
                ComponentCheckResult {
                    component_name,
                    status: HealthStatus::Unhealthy,
                    message: "WebSocket server is not reachable".to_string(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    details: {
                        let mut details = HashMap::new();
                        details.insert("port".to_string(), serde_json::Value::Number(9999.into()));
                        details.insert("listening".to_string(), serde_json::Value::Bool(false));
                        details
                    },
                }
            }
        }
    }
    
    /// Check file system health
    async fn check_file_system(&self) -> ComponentCheckResult {
        let start_time = Instant::now();
        let component_name = "file_system".to_string();
        
        // Check if we can write to temporary directory
        let test_file_path = "/tmp/wezterm-parallel-health-check";
        
        match tokio::fs::write(test_file_path, "health check").await {
            Ok(_) => {
                // Try to read it back
                match tokio::fs::read_to_string(test_file_path).await {
                    Ok(content) => {
                        // Clean up
                        let _ = tokio::fs::remove_file(test_file_path).await;
                        
                        if content == "health check" {
                            ComponentCheckResult {
                                component_name,
                                status: HealthStatus::Healthy,
                                message: "File system read/write operations successful".to_string(),
                                response_time_ms: start_time.elapsed().as_millis() as u64,
                                details: {
                                    let mut details = HashMap::new();
                                    details.insert("test_file".to_string(), serde_json::Value::String(test_file_path.to_string()));
                                    details.insert("read_write_ok".to_string(), serde_json::Value::Bool(true));
                                    details
                                },
                            }
                        } else {
                            ComponentCheckResult {
                                component_name,
                                status: HealthStatus::Degraded,
                                message: "File system data integrity issue".to_string(),
                                response_time_ms: start_time.elapsed().as_millis() as u64,
                                details: HashMap::new(),
                            }
                        }
                    }
                    Err(_) => {
                        ComponentCheckResult {
                            component_name,
                            status: HealthStatus::Degraded,
                            message: "File system read failed".to_string(),
                            response_time_ms: start_time.elapsed().as_millis() as u64,
                            details: HashMap::new(),
                        }
                    }
                }
            }
            Err(_) => {
                ComponentCheckResult {
                    component_name,
                    status: HealthStatus::Unhealthy,
                    message: "File system write failed".to_string(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    details: HashMap::new(),
                }
            }
        }
    }
    
    /// Check persistence layer health
    async fn check_persistence_layer(&self) -> ComponentCheckResult {
        let start_time = Instant::now();
        let component_name = "persistence".to_string();
        
        // Check if workspace state files are accessible
        let workspace_state_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("workspace_states");
        
        match tokio::fs::read_dir(&workspace_state_dir).await {
            Ok(mut entries) => {
                let mut file_count = 0;
                while let Ok(Some(_)) = entries.next_entry().await {
                    file_count += 1;
                }
                
                ComponentCheckResult {
                    component_name,
                    status: HealthStatus::Healthy,
                    message: format!("Persistence layer healthy with {} workspace files", file_count),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    details: {
                        let mut details = HashMap::new();
                        details.insert("workspace_files".to_string(), serde_json::Value::Number(file_count.into()));
                        details.insert("state_dir".to_string(), serde_json::Value::String(workspace_state_dir.to_string_lossy().to_string()));
                        details
                    },
                }
            }
            Err(_) => {
                ComponentCheckResult {
                    component_name,
                    status: HealthStatus::Degraded,
                    message: "Workspace state directory not accessible".to_string(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    details: {
                        let mut details = HashMap::new();
                        details.insert("state_dir".to_string(), serde_json::Value::String(workspace_state_dir.to_string_lossy().to_string()));
                        details
                    },
                }
            }
        }
    }
    
    /// Create component health from check result
    async fn create_component_health(&self, check_result: &ComponentCheckResult) -> ComponentHealth {
        let current_time = current_timestamp();
        
        // Update failure count
        let mut failure_counts = self.failure_counts.write().await;
        let failure_count = if check_result.status == HealthStatus::Healthy {
            // Reset failure count on success
            failure_counts.insert(check_result.component_name.clone(), 0);
            
            // Update last success time
            let mut last_success = self.last_success.write().await;
            last_success.insert(check_result.component_name.clone(), current_time);
            
            0
        } else {
            // Increment failure count
            let count = failure_counts.get(&check_result.component_name).unwrap_or(&0) + 1;
            failure_counts.insert(check_result.component_name.clone(), count);
            count
        };
        
        let last_success = self.last_success.read().await
            .get(&check_result.component_name)
            .copied();
        
        ComponentHealth {
            status: check_result.status.clone(),
            message: check_result.message.clone(),
            last_success,
            failure_count,
            response_time_ms: check_result.response_time_ms,
        }
    }
    
    /// Update health history
    async fn update_health_history(&self, health_check: HealthCheck) {
        let mut history = self.health_history.write().await;
        history.push(health_check);
        
        // Keep only last 100 health checks
        if history.len() > 100 {
            history.remove(0);
        }
    }
    
    /// Get latest health check
    pub async fn get_latest_health_check(&self) -> Option<HealthCheck> {
        let history = self.health_history.read().await;
        history.last().cloned()
    }
    
    /// Get health history
    pub async fn get_health_history(&self, limit: Option<usize>) -> Vec<HealthCheck> {
        let history = self.health_history.read().await;
        let limit = limit.unwrap_or(history.len());
        history.iter().rev().take(limit).cloned().collect()
    }
}

/// Determine the worst health status between two statuses
fn worst_status(status1: HealthStatus, status2: HealthStatus) -> HealthStatus {
    match (status1, status2) {
        (HealthStatus::Unknown, _) | (_, HealthStatus::Unknown) => HealthStatus::Unknown,
        (HealthStatus::Unhealthy, _) | (_, HealthStatus::Unhealthy) => HealthStatus::Unhealthy,
        (HealthStatus::Degraded, _) | (_, HealthStatus::Degraded) => HealthStatus::Degraded,
        (HealthStatus::Healthy, HealthStatus::Healthy) => HealthStatus::Healthy,
    }
}

/// Convert health status to string
fn health_status_to_string(status: &HealthStatus) -> &'static str {
    match status {
        HealthStatus::Healthy => "HEALTHY",
        HealthStatus::Degraded => "DEGRADED",
        HealthStatus::Unhealthy => "UNHEALTHY",
        HealthStatus::Unknown => "UNKNOWN",
    }
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room::WorkspaceManager;
    use crate::task::{TaskManager, TaskConfig};
    
    #[tokio::test]
    async fn test_health_check_manager_creation() {
        let workspace_manager = Arc::new(WorkspaceManager::new(None).unwrap());
        let task_manager = Arc::new(TaskManager::new(TaskConfig::default()));
        
        let health_manager = HealthCheckManager::new(
            workspace_manager,
            task_manager,
            Duration::from_secs(60),
        );
        
        let latest = health_manager.get_latest_health_check().await;
        assert!(latest.is_none());
    }
    
    #[test]
    fn test_worst_status() {
        assert_eq!(worst_status(HealthStatus::Healthy, HealthStatus::Degraded), HealthStatus::Degraded);
        assert_eq!(worst_status(HealthStatus::Degraded, HealthStatus::Unhealthy), HealthStatus::Unhealthy);
        assert_eq!(worst_status(HealthStatus::Healthy, HealthStatus::Healthy), HealthStatus::Healthy);
        assert_eq!(worst_status(HealthStatus::Unknown, HealthStatus::Healthy), HealthStatus::Unknown);
    }
}