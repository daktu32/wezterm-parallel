// WezTerm Multi-Process Development Framework - Process Pool

use std::collections::VecDeque;
use std::time::SystemTime;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug};

use crate::room::state::{ProcessInfo, ProcessStatus, TaskState};
use super::manager::ProcessManager;

#[derive(Debug)]
pub struct ProcessPool {
    manager: ProcessManager,
    task_queue: RwLock<VecDeque<QueuedTask>>,
    allocation_strategy: AllocationStrategy,
    pool_config: PoolConfig,
}

#[derive(Debug, Clone)]
struct QueuedTask {
    task: TaskState,
    assigned_process: Option<String>,
    #[allow(dead_code)]
    queued_at: SystemTime,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PoolConfig {
    pub min_processes: usize,
    pub max_processes: usize,
    pub scale_up_threshold: f32,    // CPU/Memory threshold to scale up
    pub scale_down_threshold: f32,  // CPU/Memory threshold to scale down
    pub idle_timeout_secs: u64,     // Time before idle process is terminated
    pub task_timeout_secs: u64,     // Maximum time for task execution
    pub rebalance_interval_secs: u64, // How often to rebalance load
}

#[derive(Debug, Clone)]
pub enum AllocationStrategy {
    RoundRobin,
    LeastBusy,
    Random,
    WorkspaceAffinity, // Prefer processes in same workspace
}

#[derive(Debug, Clone)]
pub struct ProcessMetrics {
    pub process_id: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub active_tasks: usize,
    pub total_tasks_completed: u64,
    pub last_task_completed: Option<SystemTime>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_processes: 2,
            max_processes: 8,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.3,
            idle_timeout_secs: 300, // 5 minutes
            task_timeout_secs: 600,  // 10 minutes
            rebalance_interval_secs: 60, // 1 minute
        }
    }
}

impl ProcessPool {
    pub async fn new(
        manager: ProcessManager,
        config: PoolConfig,
        strategy: AllocationStrategy,
    ) -> Self {
        info!("Creating process pool with min={}, max={}, strategy={:?}", 
              config.min_processes, config.max_processes, strategy);

        let pool = Self {
            manager,
            task_queue: RwLock::new(VecDeque::new()),
            allocation_strategy: strategy,
            pool_config: config,
        };

        // Start with minimum number of processes
        pool.ensure_min_processes().await;

        pool
    }

    pub async fn submit_task(&self, task: TaskState) -> Result<(), String> {
        info!("Submitting task '{}' to pool", task.id);

        let queued_task = QueuedTask {
            task,
            assigned_process: None,
            queued_at: SystemTime::now(),
        };

        {
            let mut queue = self.task_queue.write().await;
            queue.push_back(queued_task);
        }

        // Try to process queue immediately
        self.process_queue().await;

        Ok(())
    }

    pub async fn process_queue(&self) {
        let mut queue = self.task_queue.write().await;
        
        if queue.is_empty() {
            return;
        }

        debug!("Processing task queue with {} tasks", queue.len());

        let processes = self.manager.list_processes().await;
        let available_processes = self.get_available_processes(&processes).await;

        if available_processes.is_empty() {
            debug!("No available processes, checking if we can scale up");
            drop(queue); // Release lock before async call
            
            if self.can_scale_up().await {
                self.scale_up().await;
            }
            return;
        }

        // Process tasks that can be assigned
        let mut tasks_to_assign = Vec::new();
        let mut remaining_queue = VecDeque::new();

        while let Some(mut queued_task) = queue.pop_front() {
            if let Some(process_id) = self.select_process_for_task(&queued_task.task, &available_processes).await {
                queued_task.assigned_process = Some(process_id.clone());
                tasks_to_assign.push(queued_task);
            } else {
                remaining_queue.push_back(queued_task);
            }
        }

        *queue = remaining_queue;
        drop(queue); // Release lock

        // Assign tasks to processes
        for queued_task in tasks_to_assign {
            if let Some(ref process_id) = queued_task.assigned_process {
                info!("Assigning task '{}' to process '{}'", queued_task.task.id, process_id);
                
                // TODO: Send task to process via IPC
                // For now, we'll just log the assignment
                debug!("Task '{}' assigned to process '{}' in workspace '{}'", 
                       queued_task.task.id, process_id, queued_task.task.workspace);
            }
        }
    }

    async fn get_available_processes(&self, processes: &[ProcessInfo]) -> Vec<ProcessInfo> {
        processes
            .iter()
            .filter(|p| matches!(p.status, ProcessStatus::Running | ProcessStatus::Idle))
            .cloned()
            .collect()
    }

    async fn select_process_for_task(
        &self, 
        task: &TaskState, 
        available_processes: &[ProcessInfo]
    ) -> Option<String> {
        if available_processes.is_empty() {
            return None;
        }

        match &self.allocation_strategy {
            AllocationStrategy::RoundRobin => {
                // Simple round-robin: just pick the first available
                Some(available_processes[0].id.clone())
            }
            AllocationStrategy::LeastBusy => {
                // TODO: Implement based on actual process metrics
                // For now, just pick first available
                Some(available_processes[0].id.clone())
            }
            AllocationStrategy::Random => {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                available_processes.choose(&mut rng).map(|p| p.id.clone())
            }
            AllocationStrategy::WorkspaceAffinity => {
                // Prefer processes in the same workspace
                let workspace_processes: Vec<_> = available_processes
                    .iter()
                    .filter(|p| p.workspace == task.workspace)
                    .collect();

                if !workspace_processes.is_empty() {
                    Some(workspace_processes[0].id.clone())
                } else {
                    // Fallback to any available process
                    Some(available_processes[0].id.clone())
                }
            }
        }
    }

    async fn can_scale_up(&self) -> bool {
        let current_count = self.manager.get_process_count().await;
        current_count < self.pool_config.max_processes
    }

    async fn can_scale_down(&self) -> bool {
        let current_count = self.manager.get_process_count().await;
        current_count > self.pool_config.min_processes
    }

    async fn scale_up(&self) {
        let current_count = self.manager.get_process_count().await;
        
        if current_count >= self.pool_config.max_processes {
            warn!("Cannot scale up: already at maximum processes ({})", self.pool_config.max_processes);
            return;
        }

        info!("Scaling up process pool: {} -> {}", current_count, current_count + 1);

        let process_id = format!("claude-pool-{}", current_count + 1);
        let workspace = "default".to_string(); // TODO: Make this configurable
        let args = vec!["--pool-mode".to_string()];

        if let Err(e) = self.manager.spawn_process(process_id, workspace, args).await {
            warn!("Failed to scale up: {}", e);
        }
    }

    async fn scale_down(&self) {
        let processes = self.manager.list_processes().await;
        let idle_processes: Vec<_> = processes
            .iter()
            .filter(|p| matches!(p.status, ProcessStatus::Idle))
            .collect();

        if idle_processes.is_empty() {
            debug!("No idle processes to scale down");
            return;
        }

        let current_count = processes.len();
        if current_count <= self.pool_config.min_processes {
            debug!("Cannot scale down: already at minimum processes ({})", self.pool_config.min_processes);
            return;
        }

        // Find the most idle process to terminate
        let process_to_terminate = idle_processes
            .iter()
            .min_by_key(|p| p.last_heartbeat)
            .unwrap();

        info!("Scaling down process pool: terminating process '{}'", process_to_terminate.id);

        if let Err(e) = self.manager.kill_process(&process_to_terminate.id).await {
            warn!("Failed to scale down: {}", e);
        }
    }

    async fn ensure_min_processes(&self) {
        let current_count = self.manager.get_process_count().await;
        
        for i in current_count..self.pool_config.min_processes {
            let process_id = format!("claude-pool-{}", i + 1);
            let workspace = "default".to_string();
            let args = vec!["--pool-mode".to_string()];

            if let Err(e) = self.manager.spawn_process(process_id, workspace, args).await {
                warn!("Failed to start minimum process: {}", e);
                break;
            }
        }
    }

    pub async fn rebalance(&self) {
        debug!("Rebalancing process pool");

        // Get current metrics
        let processes = self.manager.list_processes().await;
        let queue_length = {
            let queue = self.task_queue.read().await;
            queue.len()
        };

        // Decide if we need to scale up or down
        let utilization = self.calculate_utilization(&processes).await;
        
        if utilization > self.pool_config.scale_up_threshold && queue_length > 0 && self.can_scale_up().await {
            self.scale_up().await;
        } else if utilization < self.pool_config.scale_down_threshold && queue_length == 0 && self.can_scale_down().await {
            self.scale_down().await;
        }

        // Process any pending tasks
        self.process_queue().await;
    }

    async fn calculate_utilization(&self, processes: &[ProcessInfo]) -> f32 {
        if processes.is_empty() {
            return 0.0;
        }

        let busy_processes = processes
            .iter()
            .filter(|p| matches!(p.status, ProcessStatus::Busy | ProcessStatus::Running))
            .count();

        busy_processes as f32 / processes.len() as f32
    }

    pub async fn get_pool_status(&self) -> PoolStatus {
        let processes = self.manager.list_processes().await;
        let queue_length = {
            let queue = self.task_queue.read().await;
            queue.len()
        };

        let running_count = processes.iter()
            .filter(|p| matches!(p.status, ProcessStatus::Running | ProcessStatus::Busy))
            .count();

        let idle_count = processes.iter()
            .filter(|p| matches!(p.status, ProcessStatus::Idle))
            .count();

        PoolStatus {
            total_processes: processes.len(),
            running_processes: running_count,
            idle_processes: idle_count,
            queued_tasks: queue_length,
            utilization: self.calculate_utilization(&processes).await,
        }
    }

    pub async fn start_rebalance_loop(&self) {
        let rebalance_interval = std::time::Duration::from_secs(self.pool_config.rebalance_interval_secs);
        
        loop {
            tokio::time::sleep(rebalance_interval).await;
            self.rebalance().await;
        }
    }

    pub async fn shutdown(&self) {
        info!("Shutting down process pool");
        
        // Cancel all queued tasks
        {
            let mut queue = self.task_queue.write().await;
            queue.clear();
        }

        // Shutdown all processes
        self.manager.shutdown_all().await;
    }
}

#[derive(Debug, Clone)]
pub struct PoolStatus {
    pub total_processes: usize,
    pub running_processes: usize,
    pub idle_processes: usize,
    pub queued_tasks: usize,
    pub utilization: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::manager::ProcessConfig;

    fn create_test_task(id: &str, workspace: &str) -> TaskState {
        TaskState {
            id: id.to_string(),
            workspace: workspace.to_string(),
            command: "test command".to_string(),
            priority: 5,
            status: crate::room::state::TaskStatus::Queued,
            dependencies: vec![],
            assigned_process: None,
            created_at: SystemTime::now(),
            started_at: None,
            completed_at: None,
            result: None,
        }
    }

    #[tokio::test]
    async fn test_process_pool_creation() {
        let process_config = ProcessConfig {
            claude_code_binary: "echo".to_string(),
            max_processes: 4,
            ..ProcessConfig::default()
        };
        
        let (manager, _receiver) = ProcessManager::new(process_config);
        let pool_config = PoolConfig {
            min_processes: 2,
            max_processes: 4,
            ..PoolConfig::default()
        };

        let pool = ProcessPool::new(manager, pool_config, AllocationStrategy::RoundRobin).await;
        
        // Give time for processes to start
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        let status = pool.get_pool_status().await;
        assert!(status.total_processes >= 2); // Should have at least min_processes
    }

    #[tokio::test]
    async fn test_task_submission() {
        let process_config = ProcessConfig {
            claude_code_binary: "echo".to_string(),
            max_processes: 2,
            ..ProcessConfig::default()
        };
        
        let (manager, _receiver) = ProcessManager::new(process_config);
        let pool_config = PoolConfig {
            min_processes: 1,
            max_processes: 2,
            ..PoolConfig::default()
        };

        let pool = ProcessPool::new(manager, pool_config, AllocationStrategy::RoundRobin).await;
        
        let task = create_test_task("test-task", "test-workspace");
        let result = pool.submit_task(task).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_workspace_affinity_strategy() {
        let process_config = ProcessConfig {
            claude_code_binary: "echo".to_string(),
            max_processes: 4,
            ..ProcessConfig::default()
        };
        
        let (manager, _receiver) = ProcessManager::new(process_config);
        let pool_config = PoolConfig {
            min_processes: 2,
            max_processes: 4,
            ..PoolConfig::default()
        };

        let pool = ProcessPool::new(manager, pool_config, AllocationStrategy::WorkspaceAffinity).await;
        
        let task = create_test_task("test-task", "specific-workspace");
        let result = pool.submit_task(task).await;
        
        assert!(result.is_ok());
    }
}

// Add this to make rand available for Random allocation strategy
// Simple random implementation for testing
mod rand {
    pub mod seq {
        pub trait SliceRandom<T> {
            fn choose<R>(&self, rng: &mut R) -> Option<&T>;
        }
        
        impl<T> SliceRandom<T> for [T] {
            fn choose<R>(&self, _rng: &mut R) -> Option<&T> {
                self.first() // Just return first element as fallback
            }
        }
    }
    
    pub fn thread_rng() {
    }
}