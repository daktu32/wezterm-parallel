// WezTerm Multi-Process Development Framework - Task Scheduler
// Provides advanced task scheduling, dependency resolution, and execution planning

use super::types::{Task, TaskId, TaskStatus};
use super::{TaskError, TaskResult, current_timestamp};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use tokio::sync::RwLock;
use tracing::{info, debug};

/// Task scheduler with dependency resolution and execution planning
#[derive(Debug)]
pub struct TaskScheduler {
    /// Scheduler configuration
    #[allow(dead_code)]
    config: SchedulerConfig,
    
    /// Scheduled tasks
    scheduled_tasks: RwLock<HashMap<TaskId, ScheduledTask>>,
    
    /// Task dependencies graph
    dependency_graph: RwLock<DependencyGraph>,
    
    /// Execution plan cache
    execution_plans: RwLock<HashMap<String, ExecutionPlan>>,
    
    /// Scheduler statistics
    stats: RwLock<SchedulerStats>,
}

impl TaskScheduler {
    /// Create a new task scheduler
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            config,
            scheduled_tasks: RwLock::new(HashMap::new()),
            dependency_graph: RwLock::new(DependencyGraph::new()),
            execution_plans: RwLock::new(HashMap::new()),
            stats: RwLock::new(SchedulerStats::new()),
        }
    }

    /// Schedule a task for execution
    pub async fn schedule_task(&self, task: Task, schedule: Schedule) -> TaskResult<()> {
        let task_id = task.id.clone();
        
        // Validate schedule
        self.validate_schedule(&schedule).await?;
        
        // Create scheduled task
        let scheduled_task = ScheduledTask {
            task: task.clone(),
            schedule: schedule.clone(),
            next_execution: self.calculate_next_execution(&schedule),
            execution_count: 0,
            last_execution: None,
            is_active: true,
        };

        // Add to scheduled tasks
        {
            let mut scheduled = self.scheduled_tasks.write().await;
            scheduled.insert(task_id.clone(), scheduled_task);
        }

        // Update dependency graph
        {
            let mut graph = self.dependency_graph.write().await;
            graph.add_task(task_id.clone(), task.dependencies.clone());
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_scheduled += 1;
        }

        info!("Scheduled task: {} with schedule: {:?}", task_id, schedule);
        Ok(())
    }

    /// Unschedule a task
    pub async fn unschedule_task(&self, task_id: &TaskId) -> TaskResult<ScheduledTask> {
        let scheduled_task = {
            let mut scheduled = self.scheduled_tasks.write().await;
            scheduled.remove(task_id).ok_or_else(|| TaskError::TaskNotFound(task_id.clone()))?
        };

        // Remove from dependency graph
        {
            let mut graph = self.dependency_graph.write().await;
            graph.remove_task(task_id);
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_unscheduled += 1;
        }

        info!("Unscheduled task: {}", task_id);
        Ok(scheduled_task)
    }

    /// Get tasks ready for execution
    pub async fn get_ready_tasks(&self) -> Vec<Task> {
        let now = current_timestamp();
        let mut ready_tasks = Vec::new();

        let scheduled = self.scheduled_tasks.read().await;
        for (task_id, scheduled_task) in scheduled.iter() {
            if scheduled_task.is_active && scheduled_task.next_execution <= now {
                // Check if dependencies are met
                if self.are_dependencies_satisfied(task_id).await {
                    ready_tasks.push(scheduled_task.task.clone());
                }
            }
        }

        // Sort by priority and execution time
        ready_tasks.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| {
                    let a_scheduled = scheduled.get(&a.id).map(|s| s.next_execution).unwrap_or(0);
                    let b_scheduled = scheduled.get(&b.id).map(|s| s.next_execution).unwrap_or(0);
                    a_scheduled.cmp(&b_scheduled)
                })
        });

        ready_tasks
    }

    /// Mark task as executed
    pub async fn mark_executed(&self, task_id: &TaskId) -> TaskResult<()> {
        let mut scheduled = self.scheduled_tasks.write().await;
        if let Some(scheduled_task) = scheduled.get_mut(task_id) {
            let now = current_timestamp();
            scheduled_task.execution_count += 1;
            scheduled_task.last_execution = Some(now);
            
            // Calculate next execution time
            scheduled_task.next_execution = self.calculate_next_execution(&scheduled_task.schedule);
            
            // Check if should continue scheduling
            match scheduled_task.schedule.repeat {
                RepeatPattern::Once => {
                    scheduled_task.is_active = false;
                }
                RepeatPattern::Count(max_count) => {
                    if scheduled_task.execution_count >= max_count {
                        scheduled_task.is_active = false;
                    }
                }
                _ => {
                    // Continue scheduling for other patterns
                }
            }

            // Update statistics
            {
                let mut stats = self.stats.write().await;
                stats.total_executed += 1;
            }

            debug!("Marked task as executed: {} (count: {})", task_id, scheduled_task.execution_count);
            Ok(())
        } else {
            Err(TaskError::TaskNotFound(task_id.clone()))
        }
    }

    /// Create execution plan for a set of tasks
    pub async fn create_execution_plan(&self, task_ids: &[TaskId]) -> TaskResult<ExecutionPlan> {
        let plan_id = format!("plan_{}", uuid::Uuid::new_v4());
        
        // Get all tasks
        let mut tasks = Vec::new();
        {
            let scheduled = self.scheduled_tasks.read().await;
            for task_id in task_ids {
                if let Some(scheduled_task) = scheduled.get(task_id) {
                    tasks.push(scheduled_task.task.clone());
                } else {
                    return Err(TaskError::TaskNotFound(task_id.clone()));
                }
            }
        }

        // Resolve dependencies and create execution order
        let execution_order = self.resolve_execution_order(&tasks).await?;
        
        // Calculate estimated completion time
        let estimated_duration = self.calculate_plan_duration(&execution_order);
        
        let plan = ExecutionPlan {
            id: plan_id.clone(),
            task_ids: task_ids.to_vec(),
            execution_order,
            estimated_duration,
            created_at: current_timestamp(),
            status: PlanStatus::Created,
        };

        // Cache the plan
        {
            let mut plans = self.execution_plans.write().await;
            plans.insert(plan_id, plan.clone());
        }

        info!("Created execution plan with {} tasks", task_ids.len());
        Ok(plan)
    }

    /// Get execution plan by ID
    pub async fn get_execution_plan(&self, plan_id: &str) -> Option<ExecutionPlan> {
        let plans = self.execution_plans.read().await;
        plans.get(plan_id).cloned()
    }

    /// Update task dependencies
    pub async fn update_dependencies(&self, task_id: &TaskId, dependencies: Vec<TaskId>) -> TaskResult<()> {
        {
            let mut graph = self.dependency_graph.write().await;
            graph.update_dependencies(task_id, dependencies.clone());
        }

        // Update the task in scheduled tasks
        {
            let mut scheduled = self.scheduled_tasks.write().await;
            if let Some(scheduled_task) = scheduled.get_mut(task_id) {
                scheduled_task.task.dependencies = dependencies;
            }
        }

        debug!("Updated dependencies for task: {}", task_id);
        Ok(())
    }

    /// Check if task dependencies are satisfied
    async fn are_dependencies_satisfied(&self, task_id: &TaskId) -> bool {
        let graph = self.dependency_graph.read().await;
        let scheduled = self.scheduled_tasks.read().await;
        
        if let Some(dependencies) = graph.get_dependencies(task_id) {
            for dep_id in dependencies {
                if let Some(dep_task) = scheduled.get(dep_id) {
                    if dep_task.task.status != TaskStatus::Completed {
                        return false;
                    }
                } else {
                    return false; // Dependency not found
                }
            }
        }
        
        true
    }

    /// Resolve execution order based on dependencies
    async fn resolve_execution_order(&self, tasks: &[Task]) -> TaskResult<Vec<TaskId>> {
        let _graph = self.dependency_graph.read().await;
        
        // Create a map of task IDs to their dependencies
        let mut task_deps: HashMap<TaskId, HashSet<TaskId>> = HashMap::new();
        let mut all_tasks: HashSet<TaskId> = HashSet::new();
        
        for task in tasks {
            all_tasks.insert(task.id.clone());
            let deps: HashSet<TaskId> = task.dependencies.iter()
                .filter(|dep| all_tasks.contains(*dep))
                .cloned()
                .collect();
            task_deps.insert(task.id.clone(), deps);
        }

        // Topological sort (Kahn's algorithm)
        let mut result = Vec::new();
        let mut in_degree: HashMap<TaskId, usize> = HashMap::new();
        let mut queue: VecDeque<TaskId> = VecDeque::new();

        // Calculate in-degrees
        for task_id in &all_tasks {
            in_degree.insert(task_id.clone(), 0);
        }
        
        for (_task_id, deps) in &task_deps {
            for dep in deps {
                if let Some(degree) = in_degree.get_mut(dep) {
                    *degree += 1;
                }
            }
        }

        // Find tasks with no dependencies
        for (task_id, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(task_id.clone());
            }
        }

        // Process queue
        while let Some(task_id) = queue.pop_front() {
            result.push(task_id.clone());
            
            // Reduce in-degree for dependent tasks
            if let Some(deps) = task_deps.get(&task_id) {
                for dep in deps {
                    if let Some(degree) = in_degree.get_mut(dep) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dep.clone());
                        }
                    }
                }
            }
        }

        // Check for circular dependencies
        if result.len() != all_tasks.len() {
            return Err(TaskError::DependencyNotMet("Circular dependency detected".to_string()));
        }

        Ok(result)
    }

    /// Calculate estimated duration for execution plan
    fn calculate_plan_duration(&self, execution_order: &[TaskId]) -> u64 {
        // For now, assume sequential execution
        // In a real implementation, this would consider parallelization opportunities
        execution_order.len() as u64 * 300 // 5 minutes per task average
    }

    /// Validate schedule configuration
    async fn validate_schedule(&self, schedule: &Schedule) -> TaskResult<()> {
        match &schedule.repeat {
            RepeatPattern::Interval(duration) => {
                if *duration == 0 {
                    return Err(TaskError::InvalidConfig("Interval cannot be zero".to_string()));
                }
            }
            RepeatPattern::Cron(pattern) => {
                // Basic cron validation (in a real implementation, use a cron parser)
                if pattern.split_whitespace().count() != 5 {
                    return Err(TaskError::InvalidConfig("Invalid cron pattern".to_string()));
                }
            }
            _ => {} // Other patterns are valid
        }
        
        Ok(())
    }

    /// Calculate next execution time based on schedule
    fn calculate_next_execution(&self, schedule: &Schedule) -> u64 {
        let now = current_timestamp();
        
        match &schedule.repeat {
            RepeatPattern::Once => schedule.start_time.unwrap_or(now),
            RepeatPattern::Interval(seconds) => now + seconds,
            RepeatPattern::Daily => {
                // Next day at same time
                now + 86400 // 24 hours
            }
            RepeatPattern::Weekly => {
                // Next week at same time
                now + 604800 // 7 days
            }
            RepeatPattern::Cron(_pattern) => {
                // For simplicity, add 1 hour (real implementation would parse cron)
                now + 3600
            }
            RepeatPattern::Count(_) => schedule.start_time.unwrap_or(now),
        }
    }

    /// Get scheduler statistics
    pub async fn get_stats(&self) -> SchedulerStats {
        let mut stats = self.stats.read().await.clone();
        
        // Update real-time stats
        let scheduled = self.scheduled_tasks.read().await;
        stats.active_schedules = scheduled.values().filter(|s| s.is_active).count() as u32;
        
        stats
    }

    /// List all scheduled tasks
    pub async fn list_scheduled_tasks(&self) -> Vec<ScheduledTask> {
        let scheduled = self.scheduled_tasks.read().await;
        scheduled.values().cloned().collect()
    }

    /// Get scheduled task by ID
    pub async fn get_scheduled_task(&self, task_id: &TaskId) -> Option<ScheduledTask> {
        let scheduled = self.scheduled_tasks.read().await;
        scheduled.get(task_id).cloned()
    }
}

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Maximum number of concurrent scheduled tasks
    pub max_scheduled_tasks: usize,
    
    /// Default execution timeout
    pub default_timeout: u64,
    
    /// Enable dependency resolution
    pub dependency_resolution: bool,
    
    /// Enable execution planning
    pub execution_planning: bool,
    
    /// Schedule check interval in seconds
    pub check_interval: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_scheduled_tasks: 100,
            default_timeout: 3600, // 1 hour
            dependency_resolution: true,
            execution_planning: true,
            check_interval: 60, // 1 minute
        }
    }
}

/// Task scheduling strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulingStrategy {
    /// Execute as soon as possible
    Immediate,
    
    /// Execute at specific time
    Scheduled,
    
    /// Execute when dependencies are met
    Dependent,
    
    /// Execute based on priority
    Priority,
    
    /// Execute in optimal order
    Optimized,
}

/// Scheduled task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub task: Task,
    pub schedule: Schedule,
    pub next_execution: u64,
    pub execution_count: u32,
    pub last_execution: Option<u64>,
    pub is_active: bool,
}

/// Task schedule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    /// Start time (optional, defaults to now)
    pub start_time: Option<u64>,
    
    /// End time (optional, infinite if not set)
    pub end_time: Option<u64>,
    
    /// Repeat pattern
    pub repeat: RepeatPattern,
    
    /// Time zone (optional)
    pub timezone: Option<String>,
}

/// Repeat pattern for scheduled tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepeatPattern {
    /// Execute once only
    Once,
    
    /// Repeat at fixed interval (seconds)
    Interval(u64),
    
    /// Repeat daily at same time
    Daily,
    
    /// Repeat weekly at same time
    Weekly,
    
    /// Repeat based on cron expression
    Cron(String),
    
    /// Repeat specific number of times
    Count(u32),
}

/// Execution plan for multiple tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: String,
    pub task_ids: Vec<TaskId>,
    pub execution_order: Vec<TaskId>,
    pub estimated_duration: u64,
    pub created_at: u64,
    pub status: PlanStatus,
}

/// Execution plan status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanStatus {
    Created,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Dependency graph for task relationships
#[derive(Debug, Clone)]
struct DependencyGraph {
    /// Task ID -> Set of dependencies
    dependencies: HashMap<TaskId, HashSet<TaskId>>,
    
    /// Task ID -> Set of dependents
    dependents: HashMap<TaskId, HashSet<TaskId>>,
}

impl DependencyGraph {
    fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    fn add_task(&mut self, task_id: TaskId, deps: Vec<TaskId>) {
        // Add dependencies
        let dep_set: HashSet<TaskId> = deps.into_iter().collect();
        self.dependencies.insert(task_id.clone(), dep_set.clone());

        // Update dependents
        for dep in dep_set {
            self.dependents.entry(dep).or_insert_with(HashSet::new).insert(task_id.clone());
        }
    }

    fn remove_task(&mut self, task_id: &TaskId) {
        // Remove from dependencies
        if let Some(deps) = self.dependencies.remove(task_id) {
            // Update dependents
            for dep in deps {
                if let Some(dep_set) = self.dependents.get_mut(&dep) {
                    dep_set.remove(task_id);
                }
            }
        }

        // Remove from dependents
        self.dependents.remove(task_id);
    }

    fn update_dependencies(&mut self, task_id: &TaskId, new_deps: Vec<TaskId>) {
        // Remove old dependencies
        if let Some(old_deps) = self.dependencies.get(task_id) {
            for dep in old_deps {
                if let Some(dep_set) = self.dependents.get_mut(dep) {
                    dep_set.remove(task_id);
                }
            }
        }

        // Add new dependencies
        self.add_task(task_id.clone(), new_deps);
    }

    fn get_dependencies(&self, task_id: &TaskId) -> Option<&HashSet<TaskId>> {
        self.dependencies.get(task_id)
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub total_scheduled: u32,
    pub total_unscheduled: u32,
    pub total_executed: u32,
    pub active_schedules: u32,
    pub created_at: u64,
}

impl SchedulerStats {
    fn new() -> Self {
        Self {
            total_scheduled: 0,
            total_unscheduled: 0,
            total_executed: 0,
            active_schedules: 0,
            created_at: current_timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::types::{TaskCategory, Task};

    fn create_test_config() -> SchedulerConfig {
        SchedulerConfig {
            max_scheduled_tasks: 10,
            default_timeout: 300,
            dependency_resolution: true,
            execution_planning: true,
            check_interval: 1,
        }
    }

    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = create_test_config();
        let scheduler = TaskScheduler::new(config);
        
        let stats = scheduler.get_stats().await;
        assert_eq!(stats.total_scheduled, 0);
        assert_eq!(stats.active_schedules, 0);
    }

    #[tokio::test]
    async fn test_schedule_task() {
        let config = create_test_config();
        let scheduler = TaskScheduler::new(config);
        
        let task = Task::new("Test Task".to_string(), TaskCategory::Development);
        let schedule = Schedule {
            start_time: None,
            end_time: None,
            repeat: RepeatPattern::Once,
            timezone: None,
        };
        
        let result = scheduler.schedule_task(task.clone(), schedule).await;
        assert!(result.is_ok());
        
        let scheduled_task = scheduler.get_scheduled_task(&task.id).await;
        assert!(scheduled_task.is_some());
        
        let stats = scheduler.get_stats().await;
        assert_eq!(stats.total_scheduled, 1);
        assert_eq!(stats.active_schedules, 1);
    }

    #[tokio::test]
    async fn test_dependency_resolution() {
        let config = create_test_config();
        let scheduler = TaskScheduler::new(config);
        
        // Create tasks with dependencies
        let task1 = Task::new("Task 1".to_string(), TaskCategory::Development);
        let mut task2 = Task::new("Task 2".to_string(), TaskCategory::Development);
        task2.dependencies = vec![task1.id.clone()];
        
        let schedule = Schedule {
            start_time: None,
            end_time: None,
            repeat: RepeatPattern::Once,
            timezone: None,
        };
        
        scheduler.schedule_task(task1, schedule.clone()).await.unwrap();
        scheduler.schedule_task(task2.clone(), schedule).await.unwrap();
        
        // Task 2 should not be ready until task 1 is completed
        let ready_tasks = scheduler.get_ready_tasks().await;
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].title, "Task 1");
    }

    #[tokio::test]
    async fn test_execution_plan() {
        let config = create_test_config();
        let scheduler = TaskScheduler::new(config);
        
        let task1 = Task::new("Task 1".to_string(), TaskCategory::Development);
        let task2 = Task::new("Task 2".to_string(), TaskCategory::Development);
        
        let schedule = Schedule {
            start_time: None,
            end_time: None,
            repeat: RepeatPattern::Once,
            timezone: None,
        };
        
        scheduler.schedule_task(task1.clone(), schedule.clone()).await.unwrap();
        scheduler.schedule_task(task2.clone(), schedule).await.unwrap();
        
        let task_ids = vec![task1.id, task2.id];
        let plan = scheduler.create_execution_plan(&task_ids).await.unwrap();
        
        assert_eq!(plan.task_ids.len(), 2);
        assert_eq!(plan.execution_order.len(), 2);
        assert_eq!(plan.status, PlanStatus::Created);
    }

    #[tokio::test]
    async fn test_repeat_patterns() {
        let config = create_test_config();
        let scheduler = TaskScheduler::new(config);
        
        let task = Task::new("Recurring Task".to_string(), TaskCategory::Development);
        let schedule = Schedule {
            start_time: None,
            end_time: None,
            repeat: RepeatPattern::Count(3),
            timezone: None,
        };
        
        scheduler.schedule_task(task.clone(), schedule).await.unwrap();
        
        // Mark as executed 3 times
        for _ in 0..3 {
            scheduler.mark_executed(&task.id).await.unwrap();
        }
        
        let scheduled_task = scheduler.get_scheduled_task(&task.id).await.unwrap();
        assert!(!scheduled_task.is_active); // Should be inactive after 3 executions
        assert_eq!(scheduled_task.execution_count, 3);
    }
}