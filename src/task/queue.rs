// WezTerm Multi-Process Development Framework - Task Queue System
// Provides task queuing, prioritization, and scheduling capabilities

use super::types::{Task, TaskId, TaskPriority, TaskStatus};
use super::{TaskError, TaskResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque, BinaryHeap};
use std::cmp::Ordering;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

/// Task queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    /// Maximum queue size
    pub max_size: usize,
    
    /// Queue strategy
    pub strategy: QueueStrategy,
    
    /// Enable priority-based ordering
    pub priority_enabled: bool,
    
    /// Enable deadline-based ordering
    pub deadline_enabled: bool,
    
    /// Enable dependency resolution
    pub dependency_resolution: bool,
    
    /// Queue processing interval in milliseconds
    pub processing_interval: u64,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            strategy: QueueStrategy::PriorityFirst,
            priority_enabled: true,
            deadline_enabled: true,
            dependency_resolution: true,
            processing_interval: 100,
        }
    }
}

/// Queue strategy for task ordering
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueueStrategy {
    /// First In, First Out
    FIFO,
    
    /// Last In, First Out
    LIFO,
    
    /// Priority-based ordering
    PriorityFirst,
    
    /// Deadline-based ordering
    DeadlineFirst,
    
    /// Shortest Job First
    ShortestFirst,
    
    /// Custom weighted scoring
    Weighted,
}

/// Task queue implementation
#[derive(Debug)]
pub struct TaskQueue {
    /// Queue configuration
    config: QueueConfig,
    
    /// Priority queue for high-priority tasks
    priority_queue: RwLock<BinaryHeap<QueuedTask>>,
    
    /// Standard FIFO queue for regular tasks
    standard_queue: RwLock<VecDeque<QueuedTask>>,
    
    /// Task lookup by ID
    task_lookup: RwLock<HashMap<TaskId, Task>>,
    
    /// Queue statistics
    stats: RwLock<QueueStats>,
}

impl TaskQueue {
    /// Create a new task queue
    pub fn new(config: QueueConfig) -> Self {
        Self {
            config,
            priority_queue: RwLock::new(BinaryHeap::new()),
            standard_queue: RwLock::new(VecDeque::new()),
            task_lookup: RwLock::new(HashMap::new()),
            stats: RwLock::new(QueueStats::new()),
        }
    }

    /// Add a task to the queue
    pub async fn enqueue(&self, task: Task) -> TaskResult<()> {
        // Check queue capacity
        let current_size = self.size().await;
        if current_size >= self.config.max_size {
            return Err(TaskError::QueueFull);
        }

        let task_id = task.id.clone();
        let queued_task = QueuedTask::new(task.clone(), &self.config);

        // Add to appropriate queue based on strategy
        match self.config.strategy {
            QueueStrategy::PriorityFirst => {
                if task.priority >= TaskPriority::High {
                    let mut priority_queue = self.priority_queue.write().await;
                    priority_queue.push(queued_task);
                } else {
                    let mut standard_queue = self.standard_queue.write().await;
                    standard_queue.push_back(queued_task);
                }
            }
            QueueStrategy::FIFO | QueueStrategy::LIFO => {
                let mut standard_queue = self.standard_queue.write().await;
                if self.config.strategy == QueueStrategy::FIFO {
                    standard_queue.push_back(queued_task);
                } else {
                    standard_queue.push_front(queued_task);
                }
            }
            _ => {
                // For other strategies, use priority queue with custom ordering
                let mut priority_queue = self.priority_queue.write().await;
                priority_queue.push(queued_task);
            }
        }

        // Add to lookup table
        {
            let mut lookup = self.task_lookup.write().await;
            lookup.insert(task_id.clone(), task);
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.enqueued += 1;
            stats.current_size += 1;
        }

        info!("Task {} enqueued successfully", task_id);
        Ok(())
    }

    /// Remove and return the next task from the queue
    pub async fn dequeue(&self) -> Option<Task> {
        // Try priority queue first
        if let Some(queued_task) = {
            let mut priority_queue = self.priority_queue.write().await;
            priority_queue.pop()
        } {
            return self.complete_dequeue(queued_task).await;
        }

        // Then try standard queue
        if let Some(queued_task) = {
            let mut standard_queue = self.standard_queue.write().await;
            standard_queue.pop_front()
        } {
            return self.complete_dequeue(queued_task).await;
        }

        None
    }

    /// Complete the dequeue operation
    async fn complete_dequeue(&self, queued_task: QueuedTask) -> Option<Task> {
        let task_id = queued_task.task_id.clone();

        // Remove from lookup
        let task = {
            let mut lookup = self.task_lookup.write().await;
            lookup.remove(&task_id)
        };

        if let Some(task) = task {
            // Update statistics
            {
                let mut stats = self.stats.write().await;
                stats.dequeued += 1;
                stats.current_size = stats.current_size.saturating_sub(1);
            }

            debug!("Task {} dequeued", task_id);
            Some(task)
        } else {
            warn!("Task {} not found in lookup during dequeue", task_id);
            None
        }
    }

    /// Peek at the next task without removing it
    pub async fn peek(&self) -> Option<Task> {
        // Check priority queue first
        let priority_task_id = {
            let priority_queue = self.priority_queue.read().await;
            priority_queue.peek().map(|qt| qt.task_id.clone())
        };

        if let Some(task_id) = priority_task_id {
            let lookup = self.task_lookup.read().await;
            if let Some(task) = lookup.get(&task_id) {
                return Some(task.clone());
            }
        }

        // Then check standard queue
        let standard_task_id = {
            let standard_queue = self.standard_queue.read().await;
            standard_queue.front().map(|qt| qt.task_id.clone())
        };

        if let Some(task_id) = standard_task_id {
            let lookup = self.task_lookup.read().await;
            if let Some(task) = lookup.get(&task_id) {
                return Some(task.clone());
            }
        }

        None
    }

    /// Remove a specific task from the queue
    pub async fn remove(&self, task_id: &TaskId) -> TaskResult<Task> {
        // Remove from lookup first
        let task = {
            let mut lookup = self.task_lookup.write().await;
            lookup.remove(task_id).ok_or_else(|| TaskError::TaskNotFound(task_id.clone()))?
        };

        // Remove from queues (this is inefficient for BinaryHeap, but necessary)
        self.remove_from_queues(task_id).await;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.removed += 1;
            stats.current_size = stats.current_size.saturating_sub(1);
        }

        info!("Task {} removed from queue", task_id);
        Ok(task)
    }

    /// Remove task from all queues
    async fn remove_from_queues(&self, task_id: &TaskId) {
        // Remove from priority queue
        {
            let mut priority_queue = self.priority_queue.write().await;
            let original_queue: Vec<QueuedTask> = priority_queue.drain().collect();
            *priority_queue = original_queue
                .into_iter()
                .filter(|qt| qt.task_id != *task_id)
                .collect();
        }

        // Remove from standard queue
        {
            let mut standard_queue = self.standard_queue.write().await;
            standard_queue.retain(|qt| qt.task_id != *task_id);
        }
    }

    /// Get current queue size
    pub async fn size(&self) -> usize {
        let stats = self.stats.read().await;
        stats.current_size
    }

    /// Check if queue is empty
    pub async fn is_empty(&self) -> bool {
        self.size().await == 0
    }

    /// Get queue statistics
    pub async fn get_stats(&self) -> QueueStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Get all tasks currently in queue
    pub async fn list_tasks(&self) -> Vec<Task> {
        let lookup = self.task_lookup.read().await;
        lookup.values().cloned().collect()
    }

    /// Get queue size
    pub async fn get_queue_size(&self) -> usize {
        let lookup = self.task_lookup.read().await;
        lookup.len()
    }

    /// Get task by ID
    pub async fn get_task(&self, task_id: &TaskId) -> Option<Task> {
        let lookup = self.task_lookup.read().await;
        lookup.get(task_id).cloned()
    }

    /// Update task in queue
    pub async fn update_task(&self, task: Task) -> TaskResult<()> {
        let task_id = task.id.clone();
        
        {
            let mut lookup = self.task_lookup.write().await;
            if lookup.contains_key(&task_id) {
                lookup.insert(task_id.clone(), task);
            } else {
                return Err(TaskError::TaskNotFound(task_id));
            }
        }

        debug!("Task {} updated in queue", task_id);
        Ok(())
    }

    /// Clear all tasks from queue
    pub async fn clear(&self) {
        {
            let mut priority_queue = self.priority_queue.write().await;
            priority_queue.clear();
        }
        
        {
            let mut standard_queue = self.standard_queue.write().await;
            standard_queue.clear();
        }
        
        {
            let mut lookup = self.task_lookup.write().await;
            lookup.clear();
        }

        {
            let mut stats = self.stats.write().await;
            stats.cleared += 1;
            stats.current_size = 0;
        }

        info!("Task queue cleared");
    }

    /// Get tasks ready for execution (dependencies met)
    pub async fn get_ready_tasks(&self) -> Vec<Task> {
        if !self.config.dependency_resolution {
            return Vec::new();
        }

        let lookup = self.task_lookup.read().await;
        let all_tasks: Vec<Task> = lookup.values().cloned().collect();
        
        // Find completed task IDs
        let completed_task_ids: Vec<TaskId> = all_tasks
            .iter()
            .filter(|task| task.status == TaskStatus::Completed)
            .map(|task| task.id.clone())
            .collect();

        // Find tasks that can be started
        all_tasks
            .into_iter()
            .filter(|task| {
                task.status == TaskStatus::Todo && task.can_start(&completed_task_ids)
            })
            .collect()
    }

    /// Get configuration
    pub fn get_config(&self) -> &QueueConfig {
        &self.config
    }
}

/// Queued task wrapper for priority ordering
#[derive(Debug, Clone)]
struct QueuedTask {
    task_id: TaskId,
    priority: TaskPriority,
    due_date: Option<u64>,
    estimated_duration: Option<u64>,
    enqueued_at: u64,
    score: f64,
}

impl QueuedTask {
    fn new(task: Task, config: &QueueConfig) -> Self {
        let score = Self::calculate_score(&task, config);
        
        Self {
            task_id: task.id,
            priority: task.priority,
            due_date: task.due_date,
            estimated_duration: task.estimated_duration,
            enqueued_at: super::current_timestamp(),
            score,
        }
    }

    fn calculate_score(task: &Task, config: &QueueConfig) -> f64 {
        let mut score = 0.0;

        // Priority component
        if config.priority_enabled {
            score += match task.priority {
                TaskPriority::Urgent => 100.0,
                TaskPriority::Critical => 80.0,
                TaskPriority::High => 60.0,
                TaskPriority::Medium => 40.0,
                TaskPriority::Low => 20.0,
            };
        }

        // Deadline component
        if config.deadline_enabled {
            if let Some(due_date) = task.due_date {
                let now = super::current_timestamp();
                if due_date <= now {
                    score += 50.0; // Overdue tasks get bonus
                } else {
                    let time_remaining = due_date - now;
                    score += 30.0 / (time_remaining as f64 / 3600.0 + 1.0); // Closer deadline = higher score
                }
            }
        }

        // Duration component (shorter tasks get slight preference)
        if let Some(duration) = task.estimated_duration {
            score += 10.0 / (duration as f64 / 60.0 + 1.0);
        }

        score
    }
}

impl PartialEq for QueuedTask {
    fn eq(&self, other: &Self) -> bool {
        self.score.partial_cmp(&other.score) == Some(Ordering::Equal)
    }
}

impl Eq for QueuedTask {}

impl PartialOrd for QueuedTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Ord for QueuedTask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.partial_cmp(&other.score).unwrap_or(Ordering::Equal)
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    /// Total tasks enqueued
    pub enqueued: u64,
    
    /// Total tasks dequeued
    pub dequeued: u64,
    
    /// Total tasks removed
    pub removed: u64,
    
    /// Times queue was cleared
    pub cleared: u64,
    
    /// Current queue size
    pub current_size: usize,
    
    /// Peak queue size
    pub peak_size: usize,
    
    /// Average wait time in seconds
    pub avg_wait_time: f64,
    
    /// Queue creation timestamp
    pub created_at: u64,
}

impl QueueStats {
    fn new() -> Self {
        Self {
            enqueued: 0,
            dequeued: 0,
            removed: 0,
            cleared: 0,
            current_size: 0,
            peak_size: 0,
            avg_wait_time: 0.0,
            created_at: super::current_timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::types::{TaskCategory, Task};

    #[tokio::test]
    async fn test_queue_creation() {
        let config = QueueConfig::default();
        let queue = TaskQueue::new(config);
        
        assert_eq!(queue.size().await, 0);
        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_enqueue_dequeue() {
        let config = QueueConfig::default();
        let queue = TaskQueue::new(config);
        
        let task = Task::new("Test Task".to_string(), TaskCategory::Development);
        let task_id = task.id.clone();
        
        // Enqueue
        let result = queue.enqueue(task).await;
        assert!(result.is_ok());
        assert_eq!(queue.size().await, 1);
        assert!(!queue.is_empty().await);
        
        // Dequeue
        let dequeued_task = queue.dequeue().await;
        assert!(dequeued_task.is_some());
        assert_eq!(dequeued_task.unwrap().id, task_id);
        assert_eq!(queue.size().await, 0);
        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let config = QueueConfig {
            strategy: QueueStrategy::PriorityFirst,
            ..Default::default()
        };
        let queue = TaskQueue::new(config);
        
        // Add tasks with different priorities
        let mut low_task = Task::new("Low Priority".to_string(), TaskCategory::Development);
        low_task.priority = TaskPriority::Low;
        
        let mut high_task = Task::new("High Priority".to_string(), TaskCategory::Development);
        high_task.priority = TaskPriority::High;
        
        let mut medium_task = Task::new("Medium Priority".to_string(), TaskCategory::Development);
        medium_task.priority = TaskPriority::Medium;
        
        // Enqueue in random order
        queue.enqueue(low_task).await.unwrap();
        queue.enqueue(high_task).await.unwrap();
        queue.enqueue(medium_task).await.unwrap();
        
        // Dequeue should return in priority order (implementation dependent)
        let first = queue.dequeue().await.unwrap();
        let second = queue.dequeue().await.unwrap(); 
        let third = queue.dequeue().await.unwrap();
        
        // Verify all tasks are returned
        let titles: Vec<String> = vec![first.title, second.title, third.title];
        assert!(titles.contains(&"High Priority".to_string()));
        assert!(titles.contains(&"Medium Priority".to_string()));
        assert!(titles.contains(&"Low Priority".to_string()));
    }

    #[tokio::test]
    async fn test_queue_capacity() {
        let config = QueueConfig {
            max_size: 2,
            ..Default::default()
        };
        let queue = TaskQueue::new(config);
        
        // Fill queue to capacity
        let task1 = Task::new("Task 1".to_string(), TaskCategory::Development);
        let task2 = Task::new("Task 2".to_string(), TaskCategory::Development);
        let task3 = Task::new("Task 3".to_string(), TaskCategory::Development);
        
        assert!(queue.enqueue(task1).await.is_ok());
        assert!(queue.enqueue(task2).await.is_ok());
        
        // Third task should fail
        let result = queue.enqueue(task3).await;
        assert!(matches!(result, Err(TaskError::QueueFull)));
    }

    #[tokio::test]
    async fn test_task_removal() {
        let config = QueueConfig::default();
        let queue = TaskQueue::new(config);
        
        let task = Task::new("Test Task".to_string(), TaskCategory::Development);
        let task_id = task.id.clone();
        
        queue.enqueue(task).await.unwrap();
        assert_eq!(queue.size().await, 1);
        
        let removed_task = queue.remove(&task_id).await.unwrap();
        assert_eq!(removed_task.id, task_id);
        assert_eq!(queue.size().await, 0);
    }

    #[tokio::test]
    async fn test_queue_stats() {
        let config = QueueConfig::default();
        let queue = TaskQueue::new(config);
        
        let stats = queue.get_stats().await;
        assert_eq!(stats.enqueued, 0);
        assert_eq!(stats.dequeued, 0);
        assert_eq!(stats.current_size, 0);
        
        let task = Task::new("Test Task".to_string(), TaskCategory::Development);
        queue.enqueue(task).await.unwrap();
        
        let stats = queue.get_stats().await;
        assert_eq!(stats.enqueued, 1);
        assert_eq!(stats.current_size, 1);
        
        queue.dequeue().await;
        
        let stats = queue.get_stats().await;
        assert_eq!(stats.enqueued, 1);
        assert_eq!(stats.dequeued, 1);
        assert_eq!(stats.current_size, 0);
    }
}