// WezTerm Multi-Process Development Framework - Task Type Definitions
// Defines core task types, states, and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Task unique identifier
pub type TaskId = String;

/// Task definition and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier
    pub id: TaskId,

    /// Task title/name
    pub title: String,

    /// Detailed task description
    pub description: Option<String>,

    /// Task status
    pub status: TaskStatus,

    /// Task priority level
    pub priority: TaskPriority,

    /// Task category/type
    pub category: TaskCategory,

    /// Associated workspace
    pub workspace: Option<String>,

    /// Task creation timestamp
    pub created_at: u64,

    /// Task update timestamp
    pub updated_at: u64,

    /// Task due date (optional)
    pub due_date: Option<u64>,

    /// Task start time (when execution began)
    pub started_at: Option<u64>,

    /// Task completion time
    pub completed_at: Option<u64>,

    /// Estimated duration in seconds
    pub estimated_duration: Option<u64>,

    /// Actual duration (calculated when completed)
    pub actual_duration: Option<u64>,

    /// Task tags for organization
    pub tags: Vec<String>,

    /// Task assignee (user/system)
    pub assignee: Option<String>,

    /// Task dependencies (must complete before this task)
    pub dependencies: Vec<TaskId>,

    /// Task metadata (flexible key-value storage)
    pub metadata: HashMap<String, String>,

    /// Task execution configuration
    pub execution: TaskExecution,

    /// Task progress (0-100%)
    pub progress: u8,

    /// Task notes/comments
    pub notes: Vec<TaskNote>,

    /// Task execution history
    pub execution_history: Vec<TaskExecutionRecord>,
}

impl Task {
    /// Create a new task
    pub fn new(title: String, category: TaskCategory) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: crate::task::generate_task_id(),
            title,
            description: None,
            status: TaskStatus::Todo,
            priority: TaskPriority::Medium,
            category,
            workspace: None,
            created_at: now,
            updated_at: now,
            due_date: None,
            started_at: None,
            completed_at: None,
            estimated_duration: None,
            actual_duration: None,
            tags: Vec::new(),
            assignee: None,
            dependencies: Vec::new(),
            metadata: HashMap::new(),
            execution: TaskExecution::default(),
            progress: 0,
            notes: Vec::new(),
            execution_history: Vec::new(),
        }
    }

    /// Update task status and timestamp
    pub fn update_status(&mut self, status: TaskStatus) {
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match &status {
            TaskStatus::InProgress => {
                if self.started_at.is_none() {
                    self.started_at = Some(self.updated_at);
                }
            }
            TaskStatus::Completed => {
                self.completed_at = Some(self.updated_at);
                if let Some(started) = self.started_at {
                    self.actual_duration = Some(self.updated_at - started);
                }
                self.progress = 100;
            }
            TaskStatus::Cancelled | TaskStatus::Failed => {
                self.completed_at = Some(self.updated_at);
                if let Some(started) = self.started_at {
                    self.actual_duration = Some(self.updated_at - started);
                }
            }
            _ => {}
        }

        self.status = status;
    }

    /// Add a note to the task
    pub fn add_note(&mut self, content: String, author: Option<String>) {
        let note = TaskNote {
            id: crate::task::generate_task_id(),
            content,
            author,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        self.notes.push(note);
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Update task progress
    pub fn update_progress(&mut self, progress: u8) {
        self.progress = progress.min(100);
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if progress >= 100 {
            self.update_status(TaskStatus::Completed);
        }
    }

    /// Check if task is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now > due_date && !self.is_completed()
        } else {
            false
        }
    }

    /// Check if task is completed
    pub fn is_completed(&self) -> bool {
        matches!(self.status, TaskStatus::Completed)
    }

    /// Check if task is in progress
    pub fn is_in_progress(&self) -> bool {
        matches!(self.status, TaskStatus::InProgress)
    }

    /// Check if task can be started (dependencies met)
    pub fn can_start(&self, completed_tasks: &[TaskId]) -> bool {
        self.dependencies
            .iter()
            .all(|dep| completed_tasks.contains(dep))
    }

    /// Get task duration (estimated or actual)
    pub fn get_duration(&self) -> Option<Duration> {
        if let Some(actual) = self.actual_duration {
            Some(Duration::from_secs(actual))
        } else {
            self.estimated_duration.map(Duration::from_secs)
        }
    }
}

/// Task status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task created but not started
    Todo,

    /// Task is actively being worked on
    InProgress,

    /// Task is blocked by dependencies or issues
    Blocked,

    /// Task is paused/on hold
    OnHold,

    /// Task needs review
    Review,

    /// Task is completed successfully
    Completed,

    /// Task was cancelled
    Cancelled,

    /// Task failed to complete
    Failed,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "To Do"),
            TaskStatus::InProgress => write!(f, "In Progress"),
            TaskStatus::Blocked => write!(f, "Blocked"),
            TaskStatus::OnHold => write!(f, "On Hold"),
            TaskStatus::Review => write!(f, "Review"),
            TaskStatus::Completed => write!(f, "Completed"),
            TaskStatus::Cancelled => write!(f, "Cancelled"),
            TaskStatus::Failed => write!(f, "Failed"),
        }
    }
}

/// Task priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    /// Lowest priority
    Low = 1,

    /// Normal priority
    Medium = 2,

    /// High priority
    High = 3,

    /// Critical priority
    Critical = 4,

    /// Emergency priority
    Urgent = 5,
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "Low"),
            TaskPriority::Medium => write!(f, "Medium"),
            TaskPriority::High => write!(f, "High"),
            TaskPriority::Critical => write!(f, "Critical"),
            TaskPriority::Urgent => write!(f, "Urgent"),
        }
    }
}

/// Task category/type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskCategory {
    /// Development/coding task
    Development,

    /// Bug fix task
    BugFix,

    /// Feature implementation
    Feature,

    /// Testing task
    Testing,

    /// Documentation task
    Documentation,

    /// Code review task
    Review,

    /// Deployment task
    Deployment,

    /// Maintenance task
    Maintenance,

    /// Research task
    Research,

    /// Meeting/discussion
    Meeting,

    /// Planning task
    Planning,

    /// Custom category
    Custom(String),
}

impl std::fmt::Display for TaskCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TaskCategory::Development => write!(f, "Development"),
            TaskCategory::BugFix => write!(f, "Bug Fix"),
            TaskCategory::Feature => write!(f, "Feature"),
            TaskCategory::Testing => write!(f, "Testing"),
            TaskCategory::Documentation => write!(f, "Documentation"),
            TaskCategory::Review => write!(f, "Review"),
            TaskCategory::Deployment => write!(f, "Deployment"),
            TaskCategory::Maintenance => write!(f, "Maintenance"),
            TaskCategory::Research => write!(f, "Research"),
            TaskCategory::Meeting => write!(f, "Meeting"),
            TaskCategory::Planning => write!(f, "Planning"),
            TaskCategory::Custom(name) => write!(f, "{name}"),
        }
    }
}

/// Task execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    /// Command to execute (if automated)
    pub command: Option<String>,

    /// Working directory for execution
    pub working_directory: Option<String>,

    /// Environment variables
    pub environment: HashMap<String, String>,

    /// Execution timeout in seconds
    pub timeout: Option<u64>,

    /// Retry configuration
    pub retry_config: RetryConfig,

    /// Auto-execute when dependencies are met
    pub auto_execute: bool,

    /// Execution mode
    pub mode: ExecutionMode,
}

impl Default for TaskExecution {
    fn default() -> Self {
        Self {
            command: None,
            working_directory: None,
            environment: HashMap::new(),
            timeout: None,
            retry_config: RetryConfig::default(),
            auto_execute: false,
            mode: ExecutionMode::Manual,
        }
    }
}

/// Task retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,

    /// Delay between retries in seconds
    pub delay: u64,

    /// Exponential backoff enabled
    pub exponential_backoff: bool,

    /// Maximum delay for exponential backoff
    pub max_delay: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            delay: 1,
            exponential_backoff: true,
            max_delay: 60,
        }
    }
}

/// Task execution mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Manual execution (user-initiated)
    Manual,

    /// Automatic execution when conditions are met
    Automatic,

    /// Scheduled execution at specific time
    Scheduled,

    /// Triggered execution by events
    Triggered,
}

/// Task note/comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNote {
    /// Note unique identifier
    pub id: String,

    /// Note content
    pub content: String,

    /// Note author
    pub author: Option<String>,

    /// Note creation timestamp
    pub created_at: u64,
}

/// Task execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionRecord {
    /// Execution attempt number
    pub attempt: u32,

    /// Execution start time
    pub started_at: u64,

    /// Execution end time
    pub ended_at: Option<u64>,

    /// Execution result
    pub result: ExecutionResult,

    /// Execution duration in seconds
    pub duration: Option<u64>,

    /// Execution output/logs
    pub output: Option<String>,

    /// Error message (if failed)
    pub error: Option<String>,
}

/// Task execution result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionResult {
    /// Execution completed successfully
    Success,

    /// Execution failed
    Failed,

    /// Execution timed out
    Timeout,

    /// Execution was cancelled
    Cancelled,

    /// Execution is still running
    Running,
}

/// Task filter criteria
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskFilter {
    /// Filter by status
    pub status: Option<TaskStatus>,

    /// Filter by priority
    pub priority: Option<TaskPriority>,

    /// Filter by category
    pub category: Option<TaskCategory>,

    /// Filter by workspace
    pub workspace: Option<String>,

    /// Filter by assignee
    pub assignee: Option<String>,

    /// Filter by tags (all must match)
    pub tags: Vec<String>,

    /// Filter by due date range
    pub due_date_range: Option<(u64, u64)>,

    /// Filter by creation date range
    pub created_date_range: Option<(u64, u64)>,

    /// Include overdue tasks only
    pub overdue_only: bool,

    /// Text search in title/description
    pub search_text: Option<String>,
}

/// Task sorting criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskSort {
    /// Sort by creation date
    CreatedAt(SortOrder),

    /// Sort by updated date
    UpdatedAt(SortOrder),

    /// Sort by due date
    DueDate(SortOrder),

    /// Sort by priority
    Priority(SortOrder),

    /// Sort by title
    Title(SortOrder),

    /// Sort by progress
    Progress(SortOrder),
}

/// Sort order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("Test Task".to_string(), TaskCategory::Development);

        assert_eq!(task.title, "Test Task");
        assert_eq!(task.status, TaskStatus::Todo);
        assert_eq!(task.priority, TaskPriority::Medium);
        assert_eq!(task.category, TaskCategory::Development);
        assert_eq!(task.progress, 0);
        assert!(!task.id.is_empty());
    }

    #[test]
    fn test_task_status_update() {
        let mut task = Task::new("Test Task".to_string(), TaskCategory::Development);

        task.update_status(TaskStatus::InProgress);
        assert_eq!(task.status, TaskStatus::InProgress);
        assert!(task.started_at.is_some());

        task.update_status(TaskStatus::Completed);
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.completed_at.is_some());
        assert!(task.actual_duration.is_some());
        assert_eq!(task.progress, 100);
    }

    #[test]
    fn test_task_progress_update() {
        let mut task = Task::new("Test Task".to_string(), TaskCategory::Development);

        task.update_progress(50);
        assert_eq!(task.progress, 50);
        assert_eq!(task.status, TaskStatus::Todo);

        task.update_progress(100);
        assert_eq!(task.progress, 100);
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[test]
    fn test_task_note_addition() {
        let mut task = Task::new("Test Task".to_string(), TaskCategory::Development);

        task.add_note(
            "This is a test note".to_string(),
            Some("user123".to_string()),
        );
        assert_eq!(task.notes.len(), 1);
        assert_eq!(task.notes[0].content, "This is a test note");
        assert_eq!(task.notes[0].author, Some("user123".to_string()));
    }

    #[test]
    fn test_task_dependency_check() {
        let task = Task::new("Test Task".to_string(), TaskCategory::Development);

        let completed_tasks = vec!["task1".to_string(), "task2".to_string()];
        assert!(task.can_start(&completed_tasks));

        let mut task_with_deps = task;
        task_with_deps.dependencies = vec!["task1".to_string(), "task3".to_string()];
        assert!(!task_with_deps.can_start(&completed_tasks));
    }

    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::Urgent > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Medium);
        assert!(TaskPriority::Medium > TaskPriority::Low);
    }

    #[test]
    fn test_task_status_display() {
        assert_eq!(TaskStatus::Todo.to_string(), "To Do");
        assert_eq!(TaskStatus::InProgress.to_string(), "In Progress");
        assert_eq!(TaskStatus::Completed.to_string(), "Completed");
    }
}
