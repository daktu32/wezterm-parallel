// WezTerm Multi-Process Development Framework - Task Board Management
// Provides Kanban-style task board with real-time WebSocket updates

use super::{DashboardMessage, TaskColumn, TaskAction, TaskBoardConfig, BoardVisibility};
use crate::task::{Task, TaskManager, TaskStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::logging::LogContext;
use crate::{log_info, log_warn, log_debug, log_error};

/// Task board manager for Kanban-style interface
pub struct TaskBoardManager {
    /// Board configurations
    boards: RwLock<HashMap<String, TaskBoardConfig>>,
    
    /// Task manager reference
    task_manager: Arc<TaskManager>,
    
    /// WebSocket broadcast channel
    broadcast_tx: tokio::sync::broadcast::Sender<DashboardMessage>,
    
    /// Default board configuration
    default_config: TaskBoardConfig,
}

impl TaskBoardManager {
    /// Create a new task board manager
    pub fn new(
        task_manager: Arc<TaskManager>,
        broadcast_tx: tokio::sync::broadcast::Sender<DashboardMessage>,
    ) -> Self {
        let default_config = Self::create_default_board_config();
        
        Self {
            boards: RwLock::new(HashMap::new()),
            task_manager,
            broadcast_tx,
            default_config,
        }
    }

    /// Initialize default board
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        let default_board = self.default_config.clone();
        
        {
            let mut boards = self.boards.write().await;
            boards.insert(default_board.id.clone(), default_board.clone());
        }

        // Broadcast initial board state
        self.broadcast_board_update(&default_board.id).await?;
        
        let init_context = LogContext::new("dashboard", "task_board_init")
            .with_entity_id(&default_board.id);
        log_info!(init_context, "Task board manager initialized with default board");
        Ok(())
    }

    /// Create default board configuration
    fn create_default_board_config() -> TaskBoardConfig {
        TaskBoardConfig {
            id: "default".to_string(),
            title: "Task Board".to_string(),
            columns: vec![
                TaskColumn {
                    id: "todo".to_string(),
                    title: "To Do".to_string(),
                    tasks: Vec::new(),
                    color: Some("#e3f2fd".to_string()),
                    max_tasks: None,
                    sort_order: 0,
                },
                TaskColumn {
                    id: "in_progress".to_string(),
                    title: "In Progress".to_string(),
                    tasks: Vec::new(),
                    color: Some("#fff3e0".to_string()),
                    max_tasks: Some(5), // Limit work in progress
                    sort_order: 1,
                },
                TaskColumn {
                    id: "review".to_string(),
                    title: "Review".to_string(),
                    tasks: Vec::new(),
                    color: Some("#fce4ec".to_string()),
                    max_tasks: None,
                    sort_order: 2,
                },
                TaskColumn {
                    id: "done".to_string(),
                    title: "Done".to_string(),
                    tasks: Vec::new(),
                    color: Some("#e8f5e8".to_string()),
                    max_tasks: None,
                    sort_order: 3,
                },
            ],
            refresh_interval: 1000, // 1 second
            real_time: true,
            visibility: BoardVisibility::Public,
        }
    }

    /// Get board configuration by ID
    pub async fn get_board(&self, board_id: &str) -> Option<TaskBoardConfig> {
        let boards = self.boards.read().await;
        boards.get(board_id).cloned()
    }

    /// Create a new board
    pub async fn create_board(&self, config: TaskBoardConfig) -> Result<(), String> {
        let board_id = config.id.clone();
        
        {
            let mut boards = self.boards.write().await;
            if boards.contains_key(&board_id) {
                return Err(format!("Board with ID '{}' already exists", board_id));
            }
            boards.insert(board_id.clone(), config);
        }

        // Broadcast new board creation
        if let Err(e) = self.broadcast_board_update(&board_id).await {
            let broadcast_error_context = LogContext::new("dashboard", "board_broadcast_error")
                .with_entity_id(&board_id);
            log_error!(broadcast_error_context, "Failed to broadcast board creation: {}", e);
        }

        let create_context = LogContext::new("dashboard", "board_create_success")
            .with_entity_id(&board_id);
        log_info!(create_context, "Created new task board: {}", board_id);
        Ok(())
    }

    /// Update board configuration
    pub async fn update_board(&self, config: TaskBoardConfig) -> Result<(), String> {
        let board_id = config.id.clone();
        
        {
            let mut boards = self.boards.write().await;
            if !boards.contains_key(&board_id) {
                return Err(format!("Board with ID '{}' not found", board_id));
            }
            boards.insert(board_id.clone(), config);
        }

        // Broadcast board update
        if let Err(e) = self.broadcast_board_update(&board_id).await {
            let update_error_context = LogContext::new("dashboard", "board_update_broadcast_error")
                .with_entity_id(&board_id);
            log_error!(update_error_context, "Failed to broadcast board update: {}", e);
        }

        let update_context = LogContext::new("dashboard", "board_update_success")
            .with_entity_id(&board_id);
        log_debug!(update_context, "Updated task board: {}", board_id);
        Ok(())
    }

    /// Delete a board
    pub async fn delete_board(&self, board_id: &str) -> Result<(), String> {
        if board_id == "default" {
            return Err("Cannot delete default board".to_string());
        }

        {
            let mut boards = self.boards.write().await;
            if boards.remove(board_id).is_none() {
                return Err(format!("Board with ID '{}' not found", board_id));
            }
        }

        let delete_context = LogContext::new("dashboard", "board_delete_success")
            .with_entity_id(board_id);
        log_info!(delete_context, "Deleted task board: {}", board_id);
        Ok(())
    }

    /// Get current board state with tasks
    pub async fn get_board_state(&self, board_id: &str) -> Result<TaskBoardState, String> {
        let board_config = self.get_board(board_id).await
            .ok_or_else(|| format!("Board '{}' not found", board_id))?;

        // Get all tasks and organize by status
        let all_tasks = self.task_manager.list_tasks(None).await;
        let mut columns = board_config.columns.clone();

        // Clear existing task lists and repopulate from current tasks
        for column in &mut columns {
            column.tasks.clear();
            
            // Map column IDs to task statuses
            let status_filter = match column.id.as_str() {
                "todo" => TaskStatus::Todo,
                "in_progress" => TaskStatus::InProgress,
                "review" => TaskStatus::Review,
                "done" => TaskStatus::Completed,
                "blocked" => TaskStatus::Blocked,
                "on_hold" => TaskStatus::OnHold,
                _ => continue, // Skip unknown columns
            };

            // Add tasks that match this column's status
            for task in &all_tasks {
                if task.status == status_filter {
                    column.tasks.push(task.id.clone());
                }
            }

            // Sort tasks by priority and creation date
            column.tasks.sort_by(|a, b| {
                let task_a = all_tasks.iter().find(|t| &t.id == a);
                let task_b = all_tasks.iter().find(|t| &t.id == b);
                
                match (task_a, task_b) {
                    (Some(a), Some(b)) => {
                        b.priority.cmp(&a.priority)
                            .then_with(|| a.created_at.cmp(&b.created_at))
                    }
                    _ => std::cmp::Ordering::Equal,
                }
            });
        }

        Ok(TaskBoardState {
            board_id: board_config.id,
            title: board_config.title,
            columns,
            tasks: all_tasks,
            last_updated: crate::task::current_timestamp(),
        })
    }

    /// Move task between columns
    pub async fn move_task(
        &self,
        board_id: &str,
        task_id: &str,
        to_column: &str,
        position: Option<usize>,
    ) -> Result<(), String> {
        // Get the task and determine new status
        let mut task = self.task_manager.get_task(&task_id.to_string()).await
            .ok_or_else(|| format!("Task '{}' not found", task_id))?;

        let old_status = task.status.clone();
        
        // Map column ID to task status
        let new_status = match to_column {
            "todo" => TaskStatus::Todo,
            "in_progress" => TaskStatus::InProgress,
            "review" => TaskStatus::Review,
            "done" => TaskStatus::Completed,
            "blocked" => TaskStatus::Blocked,
            "on_hold" => TaskStatus::OnHold,
            _ => return Err(format!("Unknown column: {}", to_column)),
        };

        // Update task status
        task.update_status(new_status);

        // Update task in manager
        self.task_manager.update_task(task).await
            .map_err(|e| format!("Failed to update task: {:?}", e))?;

        // Broadcast task move
        let message = DashboardMessage::TaskMoved {
            task_id: task_id.to_string(),
            from_column: self.status_to_column_id(&old_status),
            to_column: to_column.to_string(),
            new_position: position.unwrap_or(0),
            timestamp: crate::task::current_timestamp(),
        };

        if let Err(e) = self.broadcast_tx.send(message) {
            let broadcast_warn_context = LogContext::new("dashboard", "task_move_broadcast_failed")
                .with_entity_id(task_id);
            log_warn!(broadcast_warn_context, "Failed to broadcast task move: {}", e);
        }

        // Also broadcast updated board state
        if let Err(e) = self.broadcast_board_update(board_id).await {
            let board_error_context = LogContext::new("dashboard", "task_move_board_update_failed")
                .with_entity_id(task_id);
            log_error!(board_error_context, "Failed to broadcast board update after task move: {}", e);
        }

        let move_context = LogContext::new("dashboard", "task_move_success")
            .with_entity_id(task_id)
            .with_metadata("from_column", serde_json::json!(self.status_to_column_id(&old_status)))
            .with_metadata("to_column", serde_json::json!(to_column));
        log_info!(move_context, "Moved task {} from {} to {}", task_id, self.status_to_column_id(&old_status), to_column);
        Ok(())
    }

    /// Update task progress and broadcast
    pub async fn update_task_progress(
        &self,
        task_id: &str,
        progress: u8,
    ) -> Result<(), String> {
        // Get and update task
        let mut task = self.task_manager.get_task(&task_id.to_string()).await
            .ok_or_else(|| format!("Task '{}' not found", task_id))?;

        task.update_progress(progress);

        // Update task in manager
        self.task_manager.update_task(task.clone()).await
            .map_err(|e| format!("Failed to update task: {:?}", e))?;

        // Broadcast progress update
        let message = DashboardMessage::TaskProgress {
            task_id: task_id.to_string(),
            progress,
            timestamp: crate::task::current_timestamp(),
        };

        if let Err(e) = self.broadcast_tx.send(message) {
            let progress_warn_context = LogContext::new("dashboard", "task_progress_broadcast_failed")
                .with_entity_id(task_id);
            log_warn!(progress_warn_context, "Failed to broadcast task progress: {}", e);
        }

        // If task is completed, also send task update
        if progress >= 100 {
            self.broadcast_task_update(&task, TaskAction::ProgressUpdated).await;
        }

        let progress_context = LogContext::new("dashboard", "task_progress_update")
            .with_entity_id(task_id)
            .with_metadata("progress_percent", serde_json::json!(progress));
        log_debug!(progress_context, "Updated task {} progress to {}%", task_id, progress);
        Ok(())
    }

    /// Handle task creation from dashboard
    pub async fn create_task_from_dashboard(
        &self,
        task_data: serde_json::Value,
    ) -> Result<String, String> {
        // Deserialize task data
        let task: Task = serde_json::from_value(task_data)
            .map_err(|e| format!("Invalid task data: {}", e))?;

        // Create task through manager
        let task_id = self.task_manager.create_task(task.clone()).await
            .map_err(|e| format!("Failed to create task: {:?}", e))?;

        // Broadcast task creation
        self.broadcast_task_update(&task, TaskAction::Created).await;

        // Update board state
        if let Err(e) = self.broadcast_board_update("default").await {
            let creation_error_context = LogContext::new("dashboard", "task_create_board_update_failed")
                .with_entity_id(&task_id);
            log_error!(creation_error_context, "Failed to broadcast board update after task creation: {}", e);
        }

        let creation_context = LogContext::new("dashboard", "task_create_success")
            .with_entity_id(&task_id);
        log_info!(creation_context, "Created task {} from dashboard", task_id);
        Ok(task_id)
    }

    /// Handle task update from dashboard
    pub async fn update_task_from_dashboard(
        &self,
        task_id: &str,
        task_data: serde_json::Value,
    ) -> Result<(), String> {
        // Deserialize task data
        let task: Task = serde_json::from_value(task_data)
            .map_err(|e| format!("Invalid task data: {}", e))?;

        // Verify task ID matches
        if task.id != task_id {
            return Err("Task ID mismatch".to_string());
        }

        // Update task through manager
        self.task_manager.update_task(task.clone()).await
            .map_err(|e| format!("Failed to update task: {:?}", e))?;

        // Broadcast task update
        self.broadcast_task_update(&task, TaskAction::Updated).await;

        let update_task_context = LogContext::new("dashboard", "task_update_success")
            .with_entity_id(task_id);
        log_debug!(update_task_context, "Updated task {} from dashboard", task_id);
        Ok(())
    }

    /// Handle task deletion from dashboard
    pub async fn delete_task_from_dashboard(&self, task_id: &str) -> Result<(), String> {
        // Delete task through manager
        let task = self.task_manager.delete_task(&task_id.to_string()).await
            .map_err(|e| format!("Failed to delete task: {:?}", e))?;

        // Broadcast task deletion
        self.broadcast_task_update(&task, TaskAction::Deleted).await;

        // Update board state
        if let Err(e) = self.broadcast_board_update("default").await {
            let deletion_error_context = LogContext::new("dashboard", "task_delete_board_update_failed")
                .with_entity_id(task_id);
            log_error!(deletion_error_context, "Failed to broadcast board update after task deletion: {}", e);
        }

        let deletion_context = LogContext::new("dashboard", "task_delete_success")
            .with_entity_id(task_id);
        log_info!(deletion_context, "Deleted task {} from dashboard", task_id);
        Ok(())
    }

    /// Broadcast board state update
    async fn broadcast_board_update(&self, board_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let board_state = self.get_board_state(board_id).await?;
        
        let message = DashboardMessage::TaskBoardUpdate {
            board_id: board_state.board_id.clone(),
            columns: board_state.columns,
            timestamp: board_state.last_updated,
        };

        self.broadcast_tx.send(message)?;
        Ok(())
    }

    /// Broadcast task update
    async fn broadcast_task_update(&self, task: &Task, action: TaskAction) {
        let task_json = match serde_json::to_value(task) {
            Ok(json) => json,
            Err(e) => {
                let serialize_error_context = LogContext::new("dashboard", "task_serialize_error")
                    .with_entity_id(&task.id);
                log_error!(serialize_error_context, "Failed to serialize task: {}", e);
                return;
            }
        };

        let message = DashboardMessage::TaskUpdate {
            task: task_json,
            action,
            timestamp: crate::task::current_timestamp(),
        };

        if let Err(e) = self.broadcast_tx.send(message) {
            let task_broadcast_warn_context = LogContext::new("dashboard", "task_update_broadcast_failed")
                .with_entity_id(&task.id);
            log_warn!(task_broadcast_warn_context, "Failed to broadcast task update: {}", e);
        }
    }

    /// Convert task status to column ID
    fn status_to_column_id(&self, status: &TaskStatus) -> String {
        match status {
            TaskStatus::Todo => "todo".to_string(),
            TaskStatus::InProgress => "in_progress".to_string(),
            TaskStatus::Review => "review".to_string(),
            TaskStatus::Completed => "done".to_string(),
            TaskStatus::Blocked => "blocked".to_string(),
            TaskStatus::OnHold => "on_hold".to_string(),
            TaskStatus::Cancelled => "cancelled".to_string(),
            TaskStatus::Failed => "failed".to_string(),
        }
    }

    /// Get list of all boards
    pub async fn list_boards(&self) -> Vec<TaskBoardConfig> {
        let boards = self.boards.read().await;
        boards.values().cloned().collect()
    }

    /// Start real-time updates for a board
    pub async fn start_real_time_updates(&self, board_id: &str) -> Result<(), String> {
        let board = self.get_board(board_id).await
            .ok_or_else(|| format!("Board '{}' not found", board_id))?;

        if !board.real_time {
            return Err("Real-time updates not enabled for this board".to_string());
        }

        // Send initial board state
        if let Err(e) = self.broadcast_board_update(board_id).await {
            let initial_state_error_context = LogContext::new("dashboard", "realtime_initial_state_failed")
                .with_entity_id(board_id);
            log_error!(initial_state_error_context, "Failed to send initial board state: {}", e);
        }

        let realtime_context = LogContext::new("dashboard", "realtime_updates_start")
            .with_entity_id(board_id);
        log_info!(realtime_context, "Started real-time updates for board: {}", board_id);
        Ok(())
    }
}

/// Current task board state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBoardState {
    /// Board ID
    pub board_id: String,
    
    /// Board title
    pub title: String,
    
    /// Board columns with tasks
    pub columns: Vec<TaskColumn>,
    
    /// All tasks (for quick lookup)
    pub tasks: Vec<Task>,
    
    /// Last update timestamp
    pub last_updated: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{TaskConfig, TaskCategory};

    fn create_test_task_manager() -> Arc<TaskManager> {
        let config = TaskConfig {
            max_concurrent_tasks: 5,
            default_timeout: 300,
            max_retry_attempts: 3,
            persistence_enabled: false,
            persistence_path: None,
            auto_save_interval: 60,
            metrics_enabled: true,
            cleanup_interval: 300,
            max_task_history: 100,
        };
        Arc::new(TaskManager::new(config))
    }

    #[tokio::test]
    async fn test_task_board_manager_creation() {
        let task_manager = create_test_task_manager();
        let (broadcast_tx, mut _rx) = tokio::sync::broadcast::channel(100);
        
        let board_manager = TaskBoardManager::new(task_manager, broadcast_tx);
        
        // Consume the broadcast message to prevent SendError
        tokio::spawn(async move {
            while let Ok(_) = _rx.recv().await {
                // Consume messages
            }
        });
        
        board_manager.initialize().await.unwrap();
        
        let default_board = board_manager.get_board("default").await;
        assert!(default_board.is_some());
        
        let board = default_board.unwrap();
        assert_eq!(board.id, "default");
        assert_eq!(board.columns.len(), 4);
    }

    #[tokio::test]
    async fn test_board_state_with_tasks() {
        let task_manager = create_test_task_manager();
        let (broadcast_tx, mut _rx) = tokio::sync::broadcast::channel(100);
        
        let board_manager = TaskBoardManager::new(task_manager.clone(), broadcast_tx);
        
        // Consume the broadcast message to prevent SendError
        tokio::spawn(async move {
            while let Ok(_) = _rx.recv().await {
                // Consume messages
            }
        });
        
        board_manager.initialize().await.unwrap();
        
        // Create test tasks
        let task1 = crate::task::Task::new("Task 1".to_string(), TaskCategory::Development);
        let task2 = crate::task::Task::new("Task 2".to_string(), TaskCategory::Development);
        
        let _task1_id = task_manager.create_task(task1).await.unwrap();
        let task2_id = task_manager.create_task(task2).await.unwrap();
        
        // Update task2 status after creation
        let mut task2_stored = task_manager.get_task(&task2_id).await.unwrap();
        task2_stored.update_status(TaskStatus::InProgress);
        task_manager.update_task(task2_stored).await.unwrap();
        
        // Get board state
        let board_state = board_manager.get_board_state("default").await.unwrap();
        
        assert_eq!(board_state.tasks.len(), 2);
        
        // Check column distribution
        let todo_column = board_state.columns.iter().find(|c| c.id == "todo").unwrap();
        let in_progress_column = board_state.columns.iter().find(|c| c.id == "in_progress").unwrap();
        
        assert_eq!(todo_column.tasks.len(), 1);
        assert_eq!(in_progress_column.tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_move_task_between_columns() {
        let task_manager = create_test_task_manager();
        let (broadcast_tx, mut _rx) = tokio::sync::broadcast::channel(100);
        
        let board_manager = TaskBoardManager::new(task_manager.clone(), broadcast_tx);
        
        // Consume the broadcast message to prevent SendError
        tokio::spawn(async move {
            while let Ok(_) = _rx.recv().await {
                // Consume messages
            }
        });
        
        board_manager.initialize().await.unwrap();
        
        // Create a test task
        let task = crate::task::Task::new("Test Task".to_string(), TaskCategory::Development);
        let task_id = task_manager.create_task(task).await.unwrap();
        
        // Move task to in_progress
        let result = board_manager.move_task("default", &task_id, "in_progress", None).await;
        assert!(result.is_ok());
        
        // Verify task status changed
        let updated_task = task_manager.get_task(&task_id).await.unwrap();
        assert_eq!(updated_task.status, TaskStatus::InProgress);
    }

    #[tokio::test]
    async fn test_update_task_progress() {
        let task_manager = create_test_task_manager();
        let (broadcast_tx, mut _rx) = tokio::sync::broadcast::channel(100);
        
        let board_manager = TaskBoardManager::new(task_manager.clone(), broadcast_tx);
        
        // Consume the broadcast message to prevent SendError
        tokio::spawn(async move {
            while let Ok(_) = _rx.recv().await {
                // Consume messages
            }
        });
        
        board_manager.initialize().await.unwrap();
        
        // Create a test task
        let task = crate::task::Task::new("Test Task".to_string(), TaskCategory::Development);
        let task_id = task_manager.create_task(task).await.unwrap();
        
        // Update progress
        let result = board_manager.update_task_progress(&task_id, 75).await;
        assert!(result.is_ok());
        
        // Verify progress updated
        let updated_task = task_manager.get_task(&task_id).await.unwrap();
        assert_eq!(updated_task.progress, 75);
    }
}