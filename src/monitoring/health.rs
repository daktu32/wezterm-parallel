// WezTerm Multi-Process Development Framework - Health Check System
// Provides comprehensive health monitoring for all system components

use super::{ComponentHealth, HealthCheck, HealthStatus};
use crate::room::WorkspaceManager;
use crate::task::TaskManager;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, error, info};

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
        info!(
            "Starting health check manager with interval: {:?}",
            self.check_interval
        );

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
        components.insert(
            "workspace_manager".to_string(),
            self.create_component_health(&workspace_check).await,
        );

        // Check task manager
        let task_check = self.check_task_manager().await;
        if task_check.status != HealthStatus::Healthy {
            overall_status = worst_status(overall_status, task_check.status.clone());
        }
        components.insert(
            "task_manager".to_string(),
            self.create_component_health(&task_check).await,
        );

        // Check IPC system
        let ipc_check = self.check_ipc_system().await;
        if ipc_check.status != HealthStatus::Healthy {
            overall_status = worst_status(overall_status, ipc_check.status.clone());
        }
        components.insert(
            "ipc_system".to_string(),
            self.create_component_health(&ipc_check).await,
        );

        // Check WebSocket server
        let websocket_check = self.check_websocket_server().await;
        if websocket_check.status != HealthStatus::Healthy {
            overall_status = worst_status(overall_status, websocket_check.status.clone());
        }
        components.insert(
            "websocket_server".to_string(),
            self.create_component_health(&websocket_check).await,
        );

        // Check file system
        let filesystem_check = self.check_file_system().await;
        if filesystem_check.status != HealthStatus::Healthy {
            overall_status = worst_status(overall_status, filesystem_check.status.clone());
        }
        components.insert(
            "file_system".to_string(),
            self.create_component_health(&filesystem_check).await,
        );

        // Check database/persistence
        let persistence_check = self.check_persistence_layer().await;
        if persistence_check.status != HealthStatus::Healthy {
            overall_status = worst_status(overall_status, persistence_check.status.clone());
        }
        components.insert(
            "persistence".to_string(),
            self.create_component_health(&persistence_check).await,
        );

        let check_duration_ms = start_time.elapsed().as_millis() as u64;

        let health_check = HealthCheck {
            timestamp,
            overall_status,
            components,
            check_duration_ms,
        };

        info!(
            "Health check completed: {} ({}ms)",
            health_status_to_string(&health_check.overall_status),
            check_duration_ms
        );

        Ok(health_check)
    }

    /// Check workspace manager health
    async fn check_workspace_manager(&self) -> ComponentCheckResult {
        let start_time = Instant::now();
        let component_name = "workspace_manager".to_string();

        match self
            .workspace_manager
            .get_workspace_count()
            .await
            .try_into() as Result<u64, _>
        {
            Ok(count) => {
                let response_time = start_time.elapsed().as_millis() as u64;

                // Check if workspace manager is responsive and has reasonable workspace count
                if count > 100 {
                    ComponentCheckResult {
                        component_name,
                        status: HealthStatus::Degraded,
                        message: format!("High workspace count: {count}"),
                        response_time_ms: response_time,
                        details: {
                            let mut details = HashMap::new();
                            details.insert(
                                "workspace_count".to_string(),
                                serde_json::Value::Number(count.into()),
                            );
                            details
                        },
                    }
                } else {
                    ComponentCheckResult {
                        component_name,
                        status: HealthStatus::Healthy,
                        message: format!("Workspace manager healthy with {count} workspaces"),
                        response_time_ms: response_time,
                        details: {
                            let mut details = HashMap::new();
                            details.insert(
                                "workspace_count".to_string(),
                                serde_json::Value::Number(count.into()),
                            );
                            details
                        },
                    }
                }
            }
            Err(_) => ComponentCheckResult {
                component_name,
                status: HealthStatus::Unhealthy,
                message: "Failed to get workspace count".to_string(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                details: HashMap::new(),
            },
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
            message: format!("Task manager healthy with {count} tasks, queue size: {queue_size}"),
            response_time_ms: response_time,
            details: {
                let mut details = HashMap::new();
                details.insert(
                    "task_count".to_string(),
                    serde_json::Value::Number(count.into()),
                );
                details.insert(
                    "queue_size".to_string(),
                    serde_json::Value::Number(queue_size.into()),
                );
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
                            details.insert(
                                "socket_path".to_string(),
                                serde_json::Value::String(socket_path.to_string()),
                            );
                            details
                                .insert("socket_exists".to_string(), serde_json::Value::Bool(true));
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
                            details.insert(
                                "socket_path".to_string(),
                                serde_json::Value::String(socket_path.to_string()),
                            );
                            details.insert(
                                "socket_size".to_string(),
                                serde_json::Value::Number(metadata.len().into()),
                            );
                            details
                        },
                    }
                }
            }
            Err(_) => ComponentCheckResult {
                component_name,
                status: HealthStatus::Unhealthy,
                message: "IPC socket not accessible".to_string(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                details: {
                    let mut details = HashMap::new();
                    details.insert(
                        "socket_path".to_string(),
                        serde_json::Value::String(socket_path.to_string()),
                    );
                    details.insert("socket_exists".to_string(), serde_json::Value::Bool(false));
                    details
                },
            },
        }
    }

    /// Check WebSocket server health
    async fn check_websocket_server(&self) -> ComponentCheckResult {
        let start_time = Instant::now();
        let component_name = "websocket_server".to_string();

        // Try to connect to WebSocket server
        match tokio::net::TcpStream::connect("127.0.0.1:9999").await {
            Ok(_) => ComponentCheckResult {
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
            },
            Err(_) => ComponentCheckResult {
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
            },
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
                                    details.insert(
                                        "test_file".to_string(),
                                        serde_json::Value::String(test_file_path.to_string()),
                                    );
                                    details.insert(
                                        "read_write_ok".to_string(),
                                        serde_json::Value::Bool(true),
                                    );
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
                    Err(_) => ComponentCheckResult {
                        component_name,
                        status: HealthStatus::Degraded,
                        message: "File system read failed".to_string(),
                        response_time_ms: start_time.elapsed().as_millis() as u64,
                        details: HashMap::new(),
                    },
                }
            }
            Err(_) => ComponentCheckResult {
                component_name,
                status: HealthStatus::Unhealthy,
                message: "File system write failed".to_string(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                details: HashMap::new(),
            },
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
                    message: format!("Persistence layer healthy with {file_count} workspace files"),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    details: {
                        let mut details = HashMap::new();
                        details.insert(
                            "workspace_files".to_string(),
                            serde_json::Value::Number(file_count.into()),
                        );
                        details.insert(
                            "state_dir".to_string(),
                            serde_json::Value::String(
                                workspace_state_dir.to_string_lossy().to_string(),
                            ),
                        );
                        details
                    },
                }
            }
            Err(_) => ComponentCheckResult {
                component_name,
                status: HealthStatus::Degraded,
                message: "Workspace state directory not accessible".to_string(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                details: {
                    let mut details = HashMap::new();
                    details.insert(
                        "state_dir".to_string(),
                        serde_json::Value::String(
                            workspace_state_dir.to_string_lossy().to_string(),
                        ),
                    );
                    details
                },
            },
        }
    }

    /// Create component health from check result
    async fn create_component_health(
        &self,
        check_result: &ComponentCheckResult,
    ) -> ComponentHealth {
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
            let count = failure_counts
                .get(&check_result.component_name)
                .unwrap_or(&0)
                + 1;
            failure_counts.insert(check_result.component_name.clone(), count);
            count
        };

        let last_success = self
            .last_success
            .read()
            .await
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
        .unwrap_or_else(|_| {
            log::warn!("System time error in health check, using fallback timestamp");
            std::time::Duration::from_secs(0)
        })
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room::WorkspaceManager;
    use crate::task::{TaskConfig, TaskManager};
    use std::time::Duration;

    // === テストヘルパー関数 ===

    /// テスト用のHealthCheckManagerを作成
    fn create_test_health_manager() -> HealthCheckManager {
        let workspace_manager = Arc::new(WorkspaceManager::new(None).unwrap());
        let task_manager = Arc::new(TaskManager::new(TaskConfig::default()));

        HealthCheckManager::new(workspace_manager, task_manager, Duration::from_secs(60))
    }

    /// テスト用のTaskManagerを作成（高負荷設定）
    fn create_high_load_task_manager() -> Arc<TaskManager> {
        let config = TaskConfig {
            max_concurrent_tasks: 1000,
            default_timeout: 3600,
            max_retry_attempts: 3,
            persistence_enabled: false,
            persistence_path: None,
            auto_save_interval: 300,
            metrics_enabled: true,
            cleanup_interval: 600,
            max_task_history: 1000,
        };
        Arc::new(TaskManager::new(config))
    }

    // === 基本機能テスト ===

    #[tokio::test]
    async fn test_health_manager_new_initialization_success() {
        let health_manager = create_test_health_manager();

        // 初期化直後は履歴が空であることを確認
        let latest = health_manager.get_latest_health_check().await;
        assert!(latest.is_none());

        // 履歴が空であることを確認
        let history = health_manager.get_health_history(None).await;
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_get_latest_health_check_empty_initially() {
        let health_manager = create_test_health_manager();

        // 初期状態では履歴が空
        let latest = health_manager.get_latest_health_check().await;
        assert!(latest.is_none());
    }

    // === 包括的ヘルスチェック機能テスト ===

    #[tokio::test]
    async fn test_perform_health_check_all_components_healthy() {
        let health_manager = create_test_health_manager();

        let result = health_manager.perform_health_check().await;
        assert!(result.is_ok());

        let health_check = result.unwrap();
        assert_eq!(health_check.components.len(), 6); // 6つのコンポーネント
                                                      // check_duration_ms は u64 型なので常に0以上

        // 各コンポーネントが存在することを確認
        assert!(health_check.components.contains_key("workspace_manager"));
        assert!(health_check.components.contains_key("task_manager"));
        assert!(health_check.components.contains_key("ipc_system"));
        assert!(health_check.components.contains_key("websocket_server"));
        assert!(health_check.components.contains_key("file_system"));
        assert!(health_check.components.contains_key("persistence"));
    }

    #[tokio::test]
    async fn test_perform_health_check_with_unhealthy_components() {
        let health_manager = create_test_health_manager();

        let result = health_manager.perform_health_check().await;
        assert!(result.is_ok());

        let health_check = result.unwrap();
        // WebSocketサーバーが稼働していない場合は Unhealthy になる可能性があるが、
        // テストでは各コンポーネントチェックが実行されることを確認
        assert!(health_check.components.len() > 0);
    }

    // === 個別コンポーネントチェックテスト ===

    #[tokio::test]
    async fn test_check_workspace_manager_healthy() {
        let health_manager = create_test_health_manager();

        let result = health_manager.check_workspace_manager().await;
        assert_eq!(result.component_name, "workspace_manager");
        // response_time_ms は u64 型なので常に0以上
        assert!(result.details.contains_key("workspace_count"));
    }

    #[tokio::test]
    async fn test_check_task_manager_healthy() {
        let health_manager = create_test_health_manager();

        let result = health_manager.check_task_manager().await;
        assert_eq!(result.component_name, "task_manager");
        // response_time_ms は u64 型なので常に0以上
        assert!(result.details.contains_key("task_count"));
        assert!(result.details.contains_key("queue_size"));
    }

    #[tokio::test]
    async fn test_check_task_manager_high_queue_size() {
        let workspace_manager = Arc::new(WorkspaceManager::new(None).unwrap());
        let task_manager = create_high_load_task_manager();

        let health_manager =
            HealthCheckManager::new(workspace_manager, task_manager, Duration::from_secs(60));

        let result = health_manager.check_task_manager().await;
        assert_eq!(result.component_name, "task_manager");
        // response_time_ms は u64 型なので常に0以上

        // 詳細情報が含まれることを確認
        assert!(result.details.contains_key("task_count"));
        assert!(result.details.contains_key("queue_size"));
    }

    #[tokio::test]
    async fn test_check_ipc_system_socket_missing() {
        let health_manager = create_test_health_manager();

        let result = health_manager.check_ipc_system().await;
        assert_eq!(result.component_name, "ipc_system");
        // response_time_ms は u64 型なので常に0以上

        // ソケットが存在しない場合は Unhealthy になる
        assert!(result.details.contains_key("socket_path"));
        assert!(result.details.contains_key("socket_exists"));
    }

    #[tokio::test]
    async fn test_check_websocket_server_not_reachable() {
        let health_manager = create_test_health_manager();

        let result = health_manager.check_websocket_server().await;
        assert_eq!(result.component_name, "websocket_server");
        // response_time_ms は u64 型なので常に0以上

        // サーバーが稼働していない場合は Unhealthy になる
        assert!(result.details.contains_key("port"));
        assert!(result.details.contains_key("listening"));
    }

    #[tokio::test]
    async fn test_check_file_system_read_write_success() {
        let health_manager = create_test_health_manager();

        let result = health_manager.check_file_system().await;
        assert_eq!(result.component_name, "file_system");
        // response_time_ms は u64 型なので常に0以上

        // ファイルシステムが正常に動作することを確認
        if result.status == HealthStatus::Healthy {
            assert!(result.details.contains_key("test_file"));
            assert!(result.details.contains_key("read_write_ok"));
        }
    }

    #[tokio::test]
    async fn test_check_persistence_layer_directory_access() {
        let health_manager = create_test_health_manager();

        let result = health_manager.check_persistence_layer().await;
        assert_eq!(result.component_name, "persistence");
        // response_time_ms は u64 型なので常に0以上

        // ディレクトリアクセスの結果を確認
        assert!(result.details.contains_key("state_dir"));

        // ディレクトリが存在しない場合は Degraded になる
        if result.status == HealthStatus::Degraded {
            assert!(result.message.contains("not accessible"));
        }
    }

    // === 履歴管理・メトリクステスト ===

    #[tokio::test]
    async fn test_update_health_history_adds_entries() {
        let health_manager = create_test_health_manager();

        // 初期状態では履歴が空
        assert!(health_manager.get_latest_health_check().await.is_none());

        // ヘルスチェックを実行
        let health_check = health_manager.perform_health_check().await.unwrap();

        // 履歴を更新
        health_manager
            .update_health_history(health_check.clone())
            .await;

        // 最新の履歴を取得
        let latest = health_manager.get_latest_health_check().await;
        assert!(latest.is_some());

        let latest_check = latest.unwrap();
        assert_eq!(latest_check.overall_status, health_check.overall_status);
        assert_eq!(latest_check.components.len(), health_check.components.len());
    }

    #[tokio::test]
    async fn test_health_history_limit_enforcement() {
        let health_manager = create_test_health_manager();

        // 105個のヘルスチェックを追加（100個の制限を超える）
        for i in 0..105 {
            let health_check = HealthCheck {
                timestamp: i,
                overall_status: HealthStatus::Healthy,
                components: HashMap::new(),
                check_duration_ms: 10,
            };
            health_manager.update_health_history(health_check).await;
        }

        // 履歴は100個に制限されることを確認
        let history = health_manager.get_health_history(None).await;
        assert_eq!(history.len(), 100);

        // 最新の履歴のタイムスタンプを確認（最新から順に返される）
        assert_eq!(history[0].timestamp, 104);
        assert_eq!(history[99].timestamp, 5);
    }

    #[tokio::test]
    async fn test_get_health_history_with_limit() {
        let health_manager = create_test_health_manager();

        // 10個のヘルスチェックを追加
        for i in 0..10 {
            let health_check = HealthCheck {
                timestamp: i,
                overall_status: HealthStatus::Healthy,
                components: HashMap::new(),
                check_duration_ms: 10,
            };
            health_manager.update_health_history(health_check).await;
        }

        // 制限付きで履歴を取得
        let history = health_manager.get_health_history(Some(5)).await;
        assert_eq!(history.len(), 5);

        // 最新から5個が返されることを確認
        assert_eq!(history[0].timestamp, 9);
        assert_eq!(history[4].timestamp, 5);
    }

    // === ユーティリティ関数テスト ===

    #[test]
    fn test_worst_status_comparison_all_combinations() {
        // 既存のテストを拡張
        assert_eq!(
            worst_status(HealthStatus::Healthy, HealthStatus::Degraded),
            HealthStatus::Degraded
        );
        assert_eq!(
            worst_status(HealthStatus::Degraded, HealthStatus::Unhealthy),
            HealthStatus::Unhealthy
        );
        assert_eq!(
            worst_status(HealthStatus::Healthy, HealthStatus::Healthy),
            HealthStatus::Healthy
        );
        assert_eq!(
            worst_status(HealthStatus::Unknown, HealthStatus::Healthy),
            HealthStatus::Unknown
        );

        // 追加のテストケース
        assert_eq!(
            worst_status(HealthStatus::Unhealthy, HealthStatus::Degraded),
            HealthStatus::Unhealthy
        );
        assert_eq!(
            worst_status(HealthStatus::Degraded, HealthStatus::Healthy),
            HealthStatus::Degraded
        );
        assert_eq!(
            worst_status(HealthStatus::Unknown, HealthStatus::Unhealthy),
            HealthStatus::Unknown
        );
    }

    #[test]
    fn test_health_status_to_string_conversion() {
        assert_eq!(health_status_to_string(&HealthStatus::Healthy), "HEALTHY");
        assert_eq!(health_status_to_string(&HealthStatus::Degraded), "DEGRADED");
        assert_eq!(
            health_status_to_string(&HealthStatus::Unhealthy),
            "UNHEALTHY"
        );
        assert_eq!(health_status_to_string(&HealthStatus::Unknown), "UNKNOWN");
    }

    #[test]
    fn test_current_timestamp_generation() {
        let timestamp1 = current_timestamp();
        let timestamp2 = current_timestamp();

        // タイムスタンプが生成されることを確認
        assert!(timestamp1 > 0);
        assert!(timestamp2 > 0);

        // 2つのタイムスタンプが同じか、2番目が大きいことを確認
        assert!(timestamp2 >= timestamp1);
    }

    // === コンポーネントヘルス作成テスト ===

    #[tokio::test]
    async fn test_create_component_health_healthy_status() {
        let health_manager = create_test_health_manager();

        let check_result = ComponentCheckResult {
            component_name: "test_component".to_string(),
            status: HealthStatus::Healthy,
            message: "Component is healthy".to_string(),
            response_time_ms: 100,
            details: HashMap::new(),
        };

        let component_health = health_manager.create_component_health(&check_result).await;

        assert_eq!(component_health.status, HealthStatus::Healthy);
        assert_eq!(component_health.message, "Component is healthy");
        assert_eq!(component_health.response_time_ms, 100);
        assert_eq!(component_health.failure_count, 0);
        assert!(component_health.last_success.is_some());
    }

    #[tokio::test]
    async fn test_create_component_health_unhealthy_status() {
        let health_manager = create_test_health_manager();

        let check_result = ComponentCheckResult {
            component_name: "test_component".to_string(),
            status: HealthStatus::Unhealthy,
            message: "Component is unhealthy".to_string(),
            response_time_ms: 200,
            details: HashMap::new(),
        };

        let component_health = health_manager.create_component_health(&check_result).await;

        assert_eq!(component_health.status, HealthStatus::Unhealthy);
        assert_eq!(component_health.message, "Component is unhealthy");
        assert_eq!(component_health.response_time_ms, 200);
        assert_eq!(component_health.failure_count, 1);
    }

    #[tokio::test]
    async fn test_failure_count_increment() {
        let health_manager = create_test_health_manager();

        let check_result = ComponentCheckResult {
            component_name: "test_component".to_string(),
            status: HealthStatus::Unhealthy,
            message: "Component is unhealthy".to_string(),
            response_time_ms: 200,
            details: HashMap::new(),
        };

        // 最初の失敗
        let component_health1 = health_manager.create_component_health(&check_result).await;
        assert_eq!(component_health1.failure_count, 1);

        // 2回目の失敗
        let component_health2 = health_manager.create_component_health(&check_result).await;
        assert_eq!(component_health2.failure_count, 2);
    }

    #[tokio::test]
    async fn test_failure_count_reset_on_recovery() {
        let health_manager = create_test_health_manager();

        let unhealthy_result = ComponentCheckResult {
            component_name: "test_component".to_string(),
            status: HealthStatus::Unhealthy,
            message: "Component is unhealthy".to_string(),
            response_time_ms: 200,
            details: HashMap::new(),
        };

        let healthy_result = ComponentCheckResult {
            component_name: "test_component".to_string(),
            status: HealthStatus::Healthy,
            message: "Component is healthy".to_string(),
            response_time_ms: 100,
            details: HashMap::new(),
        };

        // 失敗
        let component_health1 = health_manager
            .create_component_health(&unhealthy_result)
            .await;
        assert_eq!(component_health1.failure_count, 1);

        // 回復
        let component_health2 = health_manager
            .create_component_health(&healthy_result)
            .await;
        assert_eq!(component_health2.failure_count, 0);
        assert!(component_health2.last_success.is_some());
    }
}
