// WezTerm Multi-Process Development Framework - Task Management System
// Provides task creation, scheduling, prioritization, and tracking capabilities

pub mod manager;
pub mod queue;
pub mod scheduler;
pub mod tracker;
pub mod types;
pub mod distributor;

pub use manager::TaskManager;
pub use queue::{TaskQueue, QueueConfig};
pub use scheduler::{TaskScheduler, SchedulingStrategy};
pub use tracker::{TaskTracker, TimeTracker};
pub use types::*;
pub use distributor::{TaskDistributor, DistributedTask, TaskDependency, ProcessLoad};

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use uuid::Uuid;

/// Task management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    /// Maximum number of concurrent tasks
    pub max_concurrent_tasks: usize,
    
    /// Default task timeout in seconds
    pub default_timeout: u64,
    
    /// Task retry attempts
    pub max_retry_attempts: u32,
    
    /// Task persistence enabled
    pub persistence_enabled: bool,
    
    /// Task persistence file path
    pub persistence_path: Option<String>,
    
    /// Auto-save interval in seconds
    pub auto_save_interval: u64,
    
    /// Enable task metrics collection
    pub metrics_enabled: bool,
    
    /// Task cleanup interval for completed tasks
    pub cleanup_interval: u64,
    
    /// Maximum task history to keep
    pub max_task_history: usize,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            default_timeout: 300, // 5 minutes
            max_retry_attempts: 3,
            persistence_enabled: true,
            persistence_path: None,
            auto_save_interval: 30, // 30 seconds
            metrics_enabled: true,
            cleanup_interval: 3600, // 1 hour
            max_task_history: 1000,
        }
    }
}

/// Task system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSystemStats {
    /// Total tasks created
    pub total_tasks: u64,
    
    /// Currently active tasks
    pub active_tasks: usize,
    
    /// Tasks in queue
    pub queued_tasks: usize,
    
    /// Completed tasks
    pub completed_tasks: u64,
    
    /// Failed tasks
    pub failed_tasks: u64,
    
    /// Average task completion time (seconds)
    pub avg_completion_time: f64,
    
    /// System uptime
    pub uptime: u64,
    
    /// Last update timestamp
    pub last_update: u64,
}

impl TaskSystemStats {
    pub fn new() -> Self {
        Self {
            total_tasks: 0,
            active_tasks: 0,
            queued_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            avg_completion_time: 0.0,
            uptime: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            last_update: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        }
    }
    
    pub fn update(&mut self) {
        self.last_update = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    }
}

/// Task system error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskError {
    /// Task not found
    TaskNotFound(String),
    
    /// Task queue is full
    QueueFull,
    
    /// Task timeout exceeded
    Timeout(String),
    
    /// Task failed with error
    ExecutionFailed(String),
    
    /// Invalid task configuration
    InvalidConfig(String),
    
    /// Dependency not met
    DependencyNotMet(String),
    
    /// Resource unavailable
    ResourceUnavailable(String),
    
    /// Persistence error
    PersistenceError(String),
    
    /// Serialization error
    SerializationError(String),
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TaskError::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            TaskError::QueueFull => write!(f, "Task queue is full"),
            TaskError::Timeout(id) => write!(f, "Task timeout: {}", id),
            TaskError::ExecutionFailed(msg) => write!(f, "Task execution failed: {}", msg),
            TaskError::InvalidConfig(msg) => write!(f, "Invalid task configuration: {}", msg),
            TaskError::DependencyNotMet(dep) => write!(f, "Dependency not met: {}", dep),
            TaskError::ResourceUnavailable(res) => write!(f, "Resource unavailable: {}", res),
            TaskError::PersistenceError(msg) => write!(f, "Persistence error: {}", msg),
            TaskError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for TaskError {}

/// Task result type
pub type TaskResult<T> = Result<T, TaskError>;

/// Generate unique task ID
pub fn generate_task_id() -> String {
    Uuid::new_v4().to_string()
}

/// Get current timestamp in seconds
pub fn current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

/// Get current timestamp in milliseconds
pub fn current_timestamp_millis() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

/// Format duration for display
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_config_default() {
        let config = TaskConfig::default();
        assert_eq!(config.max_concurrent_tasks, 10);
        assert_eq!(config.default_timeout, 300);
        assert_eq!(config.max_retry_attempts, 3);
        assert!(config.persistence_enabled);
        assert!(config.metrics_enabled);
    }

    #[test]
    fn test_task_system_stats() {
        let mut stats = TaskSystemStats::new();
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.active_tasks, 0);
        
        stats.total_tasks = 5;
        stats.update();
        assert_eq!(stats.total_tasks, 5);
        assert!(stats.last_update > 0);
    }

    #[test]
    fn test_generate_task_id() {
        let id1 = generate_task_id();
        let id2 = generate_task_id();
        
        assert_ne!(id1, id2);
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
    }

    #[test]
    fn test_current_timestamp() {
        let timestamp = current_timestamp();
        assert!(timestamp > 0);
    }

    #[test]
    fn test_format_duration() {
        let duration1 = Duration::from_secs(30);
        assert_eq!(format_duration(duration1), "30s");
        
        let duration2 = Duration::from_secs(90);
        assert_eq!(format_duration(duration2), "1m 30s");
        
        let duration3 = Duration::from_secs(3665);
        assert_eq!(format_duration(duration3), "1h 1m 5s");
    }

    #[test]
    fn test_task_error_display() {
        let error = TaskError::TaskNotFound("test-123".to_string());
        assert_eq!(error.to_string(), "Task not found: test-123");
        
        let error = TaskError::QueueFull;
        assert_eq!(error.to_string(), "Task queue is full");
    }
}