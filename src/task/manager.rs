// WezTerm Multi-Process Development Framework - Task Manager
// Central task management system with scheduling, execution, and tracking

use super::types::{Task, TaskId, TaskStatus, TaskCategory, TaskFilter, TaskExecution};
use super::queue::{TaskQueue, QueueConfig};
use super::tracker::{TaskTracker};
use super::{TaskConfig, TaskSystemStats, TaskError, TaskResult, current_timestamp};
use crate::room::WorkspaceManager;
use crate::process::manager::ProcessManager;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{info, warn, debug};

/// Central task management system
pub struct TaskManager {
    /// Task management configuration
    config: TaskConfig,
    
    /// Task storage (all tasks)
    tasks: RwLock<HashMap<TaskId, Task>>,
    
    /// Task queue for pending tasks
    queue: Arc<TaskQueue>,
    
    /// Task tracker for time and progress tracking
    tracker: Arc<TaskTracker>,
    
    /// Currently executing tasks
    executing_tasks: Arc<RwLock<HashMap<TaskId, ExecutingTask>>>,
    
    /// Task templates for quick creation
    templates: RwLock<HashMap<String, TaskTemplate>>,
    
    /// System statistics
    stats: RwLock<TaskSystemStats>,
    
    /// Workspace manager reference
    workspace_manager: Option<Arc<WorkspaceManager>>,
    
    /// Process manager reference
    process_manager: Option<Arc<ProcessManager>>,
    
    /// Event listeners
    event_listeners: RwLock<Vec<Box<dyn Fn(&TaskEvent) + Send + Sync>>>,
}

impl TaskManager {
    /// Create a new task manager
    pub fn new(config: TaskConfig) -> Self {
        let queue_config = QueueConfig {
            max_size: config.max_concurrent_tasks * 10, // Queue can hold 10x concurrent limit
            ..Default::default()
        };
        
        let queue = Arc::new(TaskQueue::new(queue_config));
        let tracker = Arc::new(TaskTracker::new());
        
        Self {
            config,
            tasks: RwLock::new(HashMap::new()),
            queue,
            tracker,
            executing_tasks: Arc::new(RwLock::new(HashMap::new())),
            templates: RwLock::new(HashMap::new()),
            stats: RwLock::new(TaskSystemStats::new()),
            workspace_manager: None,
            process_manager: None,
            event_listeners: RwLock::new(Vec::new()),
        }
    }

    /// Set workspace manager reference
    pub fn with_workspace_manager(mut self, workspace_manager: Arc<WorkspaceManager>) -> Self {
        self.workspace_manager = Some(workspace_manager);
        self
    }

    /// Set process manager reference
    pub fn with_process_manager(mut self, process_manager: Arc<ProcessManager>) -> Self {
        self.process_manager = Some(process_manager);
        self
    }

    /// Start the task manager (background processing)
    pub async fn start(&self) -> TaskResult<tokio::task::JoinHandle<()>> {
        info!("Starting task manager");
        
        let queue = Arc::clone(&self.queue);
        let executing_tasks = Arc::clone(&self.executing_tasks);
        let config = self.config.clone();
        let tracker = Arc::clone(&self.tracker);
        
        let task_handle = tokio::spawn(async move {
            let mut processing_interval = interval(Duration::from_millis(100));
            let mut cleanup_interval = interval(Duration::from_secs(config.cleanup_interval));
            
            loop {
                tokio::select! {
                    _ = processing_interval.tick() => {
                        Self::process_queue_tick(&queue, &executing_tasks, &config, &tracker).await;
                    }
                    _ = cleanup_interval.tick() => {
                        Self::cleanup_completed_tasks(&executing_tasks, &config).await;
                    }
                }
            }
        });

        Ok(task_handle)
    }

    /// Process one tick of the queue
    async fn process_queue_tick(
        queue: &Arc<TaskQueue>,
        executing_tasks: &Arc<RwLock<HashMap<TaskId, ExecutingTask>>>,
        config: &TaskConfig,
        tracker: &Arc<TaskTracker>,
    ) {
        // Check if we can start more tasks
        let current_executing = {
            let executing = executing_tasks.read().await;
            executing.len()
        };

        if current_executing >= config.max_concurrent_tasks {
            return; // At capacity
        }

        // Try to dequeue a ready task
        if let Some(mut task) = queue.dequeue().await {
            task.update_status(TaskStatus::InProgress);
            
            let executing_task = ExecutingTask {
                task_id: task.id.clone(),
                started_at: current_timestamp(),
                timeout_at: task.execution.timeout.map(|t| current_timestamp() + t),
            };

            // Start tracking
            tracker.start_task(&task.id).await;

            // Add to executing tasks
            {
                let mut executing = executing_tasks.write().await;
                executing.insert(task.id.clone(), executing_task);
            }

            // Spawn execution task
            let task_id = task.id.clone();
            let executing_tasks_ref = Arc::clone(executing_tasks);
            let tracker_ref = Arc::clone(tracker);
            
            tokio::spawn(async move {
                let result = Self::execute_task(task).await;
                
                // Remove from executing
                {
                    let mut executing = executing_tasks_ref.write().await;
                    executing.remove(&task_id);
                }
                
                // Stop tracking
                tracker_ref.stop_task(&task_id).await;
                
                debug!("Task {} execution completed: {:?}", task_id, result);
            });
        }
    }

    /// Execute a single task
    async fn execute_task(mut task: Task) -> TaskResult<()> {
        debug!("Executing task: {}", task.id);

        // Simulate task execution based on task type
        match task.execution.mode {
            super::types::ExecutionMode::Manual => {
                // Manual tasks are marked as in progress and wait for user completion
                // In a real implementation, this would integrate with the UI
                sleep(Duration::from_secs(1)).await;
                task.update_status(TaskStatus::Review);
            }
            super::types::ExecutionMode::Automatic => {
                // Automatic tasks execute their command
                if let Some(command) = &task.execution.command {
                    match Self::execute_command(command, &task).await {
                        Ok(_) => task.update_status(TaskStatus::Completed),
                        Err(e) => {
                            task.update_status(TaskStatus::Failed);
                            return Err(e);
                        }
                    }
                } else {
                    // No command specified, just mark as completed
                    task.update_status(TaskStatus::Completed);
                }
            }
            _ => {
                // Other execution modes
                sleep(Duration::from_millis(500)).await;
                task.update_status(TaskStatus::Completed);
            }
        }

        Ok(())
    }

    /// Execute a command for a task
    async fn execute_command(command: &str, task: &Task) -> TaskResult<String> {
        debug!("Executing command for task {}: {}", task.id, command);
        
        // In a real implementation, this would execute the actual command
        // For now, we simulate success
        sleep(Duration::from_millis(100)).await;
        
        Ok("Command executed successfully".to_string())
    }

    /// Clean up completed tasks
    async fn cleanup_completed_tasks(
        executing_tasks: &Arc<RwLock<HashMap<TaskId, ExecutingTask>>>,
        _config: &TaskConfig,
    ) {
        let now = current_timestamp();
        let mut to_timeout = Vec::new();

        // Check for timeouts
        {
            let executing = executing_tasks.read().await;
            for (task_id, executing_task) in executing.iter() {
                if let Some(timeout_at) = executing_task.timeout_at {
                    if now >= timeout_at {
                        to_timeout.push(task_id.clone());
                    }
                }
            }
        }

        // Handle timeouts
        if !to_timeout.is_empty() {
            warn!("Timing out {} tasks", to_timeout.len());
            let mut executing = executing_tasks.write().await;
            for task_id in to_timeout {
                executing.remove(&task_id);
            }
        }
    }

    /// Create a new task
    pub async fn create_task(&self, mut task: Task) -> TaskResult<TaskId> {
        let task_id = task.id.clone();
        
        // Validate task
        self.validate_task(&task).await?;
        
        // Set initial status
        task.update_status(TaskStatus::Todo);
        
        // Store task
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id.clone(), task.clone());
        }
        
        // Add to queue if not blocked by dependencies
        if task.dependencies.is_empty() || self.are_dependencies_met(&task).await {
            self.queue.enqueue(task).await?;
        }
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_tasks += 1;
            stats.update();
        }
        
        // Notify listeners
        self.notify_listeners(TaskEvent::TaskCreated(task_id.clone())).await;
        
        info!("Task created: {}", task_id);
        Ok(task_id)
    }

    /// Create task from template
    pub async fn create_task_from_template(
        &self,
        template_name: &str,
        title: String,
        workspace: Option<String>,
    ) -> TaskResult<TaskId> {
        let template = {
            let templates = self.templates.read().await;
            templates.get(template_name)
                .ok_or_else(|| TaskError::InvalidConfig(format!("Template '{}' not found", template_name)))?
                .clone()
        };

        let mut task = Task::new(title, template.category);
        task.description = template.description;
        task.priority = template.priority;
        task.estimated_duration = template.estimated_duration;
        task.workspace = workspace;
        task.execution = template.execution;
        task.tags = template.tags;

        self.create_task(task).await
    }

    /// Update a task
    pub async fn update_task(&self, mut task: Task) -> TaskResult<()> {
        let task_id = task.id.clone();
        
        // Validate task
        self.validate_task(&task).await?;
        
        task.updated_at = current_timestamp();
        
        // Update in storage
        {
            let mut tasks = self.tasks.write().await;
            if tasks.contains_key(&task_id) {
                tasks.insert(task_id.clone(), task.clone());
            } else {
                return Err(TaskError::TaskNotFound(task_id));
            }
        }
        
        // Update in queue if present
        let _ = self.queue.update_task(task).await;
        
        // Notify listeners
        self.notify_listeners(TaskEvent::TaskUpdated(task_id.clone())).await;
        
        debug!("Task updated: {}", task_id);
        Ok(())
    }

    /// Delete a task
    pub async fn delete_task(&self, task_id: &TaskId) -> TaskResult<Task> {
        // Remove from storage
        let task = {
            let mut tasks = self.tasks.write().await;
            tasks.remove(task_id).ok_or_else(|| TaskError::TaskNotFound(task_id.clone()))?
        };

        // Remove from queue
        let _ = self.queue.remove(task_id).await;

        // Remove from executing tasks
        {
            let mut executing = self.executing_tasks.write().await;
            executing.remove(task_id);
        }

        // Stop tracking
        self.tracker.stop_task(task_id).await;

        // Notify listeners
        self.notify_listeners(TaskEvent::TaskDeleted(task_id.clone())).await;

        info!("Task deleted: {}", task_id);
        Ok(task)
    }

    /// Get a task by ID
    pub async fn get_task(&self, task_id: &TaskId) -> Option<Task> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// Get total task count
    pub async fn get_task_count(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks.len()
    }

    /// List tasks with optional filter
    pub async fn list_tasks(&self, filter: Option<TaskFilter>) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        let mut result: Vec<Task> = tasks.values().cloned().collect();

        // Apply filters
        if let Some(filter) = filter {
            result = self.apply_filter(result, filter);
        }

        result
    }

    /// Get task statistics
    pub async fn get_stats(&self) -> TaskSystemStats {
        let mut stats = self.stats.read().await.clone();
        
        // Update real-time stats
        stats.active_tasks = {
            let executing = self.executing_tasks.read().await;
            executing.len()
        };
        
        stats.queued_tasks = self.queue.size().await;
        stats.update();
        
        stats
    }

    /// Validate task data
    async fn validate_task(&self, task: &Task) -> TaskResult<()> {
        if task.title.trim().is_empty() {
            return Err(TaskError::InvalidConfig("Task title cannot be empty".to_string()));
        }

        // Validate dependencies exist
        for dep_id in &task.dependencies {
            if self.get_task(dep_id).await.is_none() {
                return Err(TaskError::DependencyNotMet(dep_id.clone()));
            }
        }

        Ok(())
    }

    /// Check if task dependencies are met
    async fn are_dependencies_met(&self, task: &Task) -> bool {
        for dep_id in &task.dependencies {
            if let Some(dep_task) = self.get_task(dep_id).await {
                if dep_task.status != TaskStatus::Completed {
                    return false;
                }
            } else {
                return false; // Dependency doesn't exist
            }
        }
        true
    }

    /// Apply filter to task list
    fn apply_filter(&self, mut tasks: Vec<Task>, filter: TaskFilter) -> Vec<Task> {
        if let Some(status) = filter.status {
            tasks.retain(|task| task.status == status);
        }

        if let Some(priority) = filter.priority {
            tasks.retain(|task| task.priority == priority);
        }

        if let Some(category) = filter.category {
            tasks.retain(|task| task.category == category);
        }

        if let Some(workspace) = filter.workspace {
            tasks.retain(|task| task.workspace.as_ref() == Some(&workspace));
        }

        if let Some(assignee) = filter.assignee {
            tasks.retain(|task| task.assignee.as_ref() == Some(&assignee));
        }

        if !filter.tags.is_empty() {
            tasks.retain(|task| {
                filter.tags.iter().all(|tag| task.tags.contains(tag))
            });
        }

        if filter.overdue_only {
            tasks.retain(|task| task.is_overdue());
        }

        if let Some(search_text) = filter.search_text {
            let search_lower = search_text.to_lowercase();
            tasks.retain(|task| {
                task.title.to_lowercase().contains(&search_lower) ||
                task.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&search_lower))
            });
        }

        tasks
    }

    /// Add task event listener
    pub async fn add_event_listener(&self, listener: Box<dyn Fn(&TaskEvent) + Send + Sync>) {
        let mut listeners = self.event_listeners.write().await;
        listeners.push(listener);
    }

    /// Notify all event listeners
    async fn notify_listeners(&self, event: TaskEvent) {
        let listeners = self.event_listeners.read().await;
        for listener in listeners.iter() {
            listener(&event);
        }
    }

    /// Register task template
    pub async fn register_template(&self, name: String, template: TaskTemplate) {
        let mut templates = self.templates.write().await;
        templates.insert(name, template);
    }

    /// Get task queue reference
    pub fn get_queue(&self) -> Arc<TaskQueue> {
        Arc::clone(&self.queue)
    }

    /// Get task tracker reference
    pub fn get_tracker(&self) -> Arc<TaskTracker> {
        Arc::clone(&self.tracker)
    }

    /// Generate productivity report for time range
    pub async fn generate_productivity_report(&self, since_timestamp: Option<u64>) -> super::tracker::ProductivityReport {
        self.tracker.generate_enhanced_productivity_report(since_timestamp).await
    }

    /// Get productivity insights for a specific task
    pub async fn get_task_insights(&self, task_id: &TaskId) -> Option<super::tracker::TaskInsights> {
        self.tracker.get_task_insights(task_id).await
    }

    /// Start time tracking for a task
    pub async fn start_task_tracking(&self, task_id: &TaskId) {
        self.tracker.start_task(task_id).await;
    }

    /// Stop time tracking for a task
    pub async fn stop_task_tracking(&self, task_id: &TaskId) {
        self.tracker.stop_task(task_id).await;
    }

    /// Pause time tracking for a task
    pub async fn pause_task_tracking(&self, task_id: &TaskId) -> bool {
        self.tracker.pause_task(task_id).await
    }

    /// Resume time tracking for a task
    pub async fn resume_task_tracking(&self, task_id: &TaskId) -> bool {
        self.tracker.resume_task(task_id).await
    }
}

/// Currently executing task information
#[derive(Debug, Clone)]
struct ExecutingTask {
    task_id: TaskId,
    started_at: u64,
    timeout_at: Option<u64>,
}

/// Task template for quick task creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTemplate {
    pub name: String,
    pub description: Option<String>,
    pub category: TaskCategory,
    pub priority: super::types::TaskPriority,
    pub estimated_duration: Option<u64>,
    pub execution: TaskExecution,
    pub tags: Vec<String>,
}

/// Task events
#[derive(Debug, Clone)]
pub enum TaskEvent {
    TaskCreated(TaskId),
    TaskUpdated(TaskId),
    TaskDeleted(TaskId),
    TaskStarted(TaskId),
    TaskCompleted(TaskId),
    TaskFailed(TaskId),
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::types::{TaskCategory, TaskPriority};

    fn create_test_config() -> TaskConfig {
        TaskConfig {
            max_concurrent_tasks: 2,
            default_timeout: 10,
            max_retry_attempts: 1,
            persistence_enabled: false,
            persistence_path: None,
            auto_save_interval: 1,
            metrics_enabled: true,
            cleanup_interval: 1,
            max_task_history: 100,
        }
    }

    #[tokio::test]
    async fn test_task_manager_creation() {
        let config = create_test_config();
        let manager = TaskManager::new(config);
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.active_tasks, 0);
    }

    #[tokio::test]
    async fn test_create_task() {
        let config = create_test_config();
        let manager = TaskManager::new(config);
        
        let task = Task::new("Test Task".to_string(), TaskCategory::Development);
        let task_id = manager.create_task(task).await.unwrap();
        
        assert!(!task_id.is_empty());
        
        let retrieved_task = manager.get_task(&task_id).await;
        assert!(retrieved_task.is_some());
        assert_eq!(retrieved_task.unwrap().title, "Test Task");
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_tasks, 1);
    }

    #[tokio::test]
    async fn test_update_task() {
        let config = create_test_config();
        let manager = TaskManager::new(config);
        
        let task = Task::new("Test Task".to_string(), TaskCategory::Development);
        let task_id = manager.create_task(task).await.unwrap();
        
        let mut updated_task = manager.get_task(&task_id).await.unwrap();
        updated_task.title = "Updated Task".to_string();
        updated_task.priority = TaskPriority::High;
        
        let result = manager.update_task(updated_task).await;
        assert!(result.is_ok());
        
        let retrieved_task = manager.get_task(&task_id).await.unwrap();
        assert_eq!(retrieved_task.title, "Updated Task");
        assert_eq!(retrieved_task.priority, TaskPriority::High);
    }

    #[tokio::test]
    async fn test_delete_task() {
        let config = create_test_config();
        let manager = TaskManager::new(config);
        
        let task = Task::new("Test Task".to_string(), TaskCategory::Development);
        let task_id = manager.create_task(task).await.unwrap();
        
        let deleted_task = manager.delete_task(&task_id).await.unwrap();
        assert_eq!(deleted_task.title, "Test Task");
        
        let retrieved_task = manager.get_task(&task_id).await;
        assert!(retrieved_task.is_none());
    }

    #[tokio::test]
    async fn test_list_tasks_with_filter() {
        let config = create_test_config();
        let manager = TaskManager::new(config);
        
        // Create tasks with different priorities
        let mut high_task = Task::new("High Priority Task".to_string(), TaskCategory::Development);
        high_task.priority = TaskPriority::High;
        
        let mut low_task = Task::new("Low Priority Task".to_string(), TaskCategory::Testing);
        low_task.priority = TaskPriority::Low;
        
        manager.create_task(high_task).await.unwrap();
        manager.create_task(low_task).await.unwrap();
        
        // Filter by priority
        let filter = TaskFilter {
            priority: Some(TaskPriority::High),
            ..Default::default()
        };
        
        let filtered_tasks = manager.list_tasks(Some(filter)).await;
        assert_eq!(filtered_tasks.len(), 1);
        assert_eq!(filtered_tasks[0].title, "High Priority Task");
        
        // Filter by category
        let filter = TaskFilter {
            category: Some(TaskCategory::Testing),
            ..Default::default()
        };
        
        let filtered_tasks = manager.list_tasks(Some(filter)).await;
        assert_eq!(filtered_tasks.len(), 1);
        assert_eq!(filtered_tasks[0].title, "Low Priority Task");
    }

    #[tokio::test]
    async fn test_task_template() {
        let config = create_test_config();
        let manager = TaskManager::new(config);
        
        let template = TaskTemplate {
            name: "Bug Fix Template".to_string(),
            description: Some("Standard bug fix template".to_string()),
            category: TaskCategory::BugFix,
            priority: TaskPriority::High,
            estimated_duration: Some(3600), // 1 hour
            execution: TaskExecution::default(),
            tags: vec!["bug".to_string(), "urgent".to_string()],
        };
        
        manager.register_template("bug_fix".to_string(), template).await;
        
        let task_id = manager.create_task_from_template(
            "bug_fix",
            "Fix login issue".to_string(),
            Some("frontend".to_string())
        ).await.unwrap();
        
        let task = manager.get_task(&task_id).await.unwrap();
        assert_eq!(task.title, "Fix login issue");
        assert_eq!(task.category, TaskCategory::BugFix);
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.workspace, Some("frontend".to_string()));
        assert!(task.tags.contains(&"bug".to_string()));
    }
}