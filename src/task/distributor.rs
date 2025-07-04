use crate::task::types::{TaskStatus, TaskPriority, Task as BaseTask};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskDependency {
    TaskCompletion(Uuid),
    FileAccess(String),
    ResourceAvailability(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedTask {
    pub base_task: BaseTask,
    pub distribution_id: Uuid,
    pub dependencies: Vec<TaskDependency>,
    pub cpu_requirement: f64,
    pub memory_requirement: f64,
    pub assigned_process: Option<Uuid>,
}

impl DistributedTask {
    pub fn new(title: String, priority: TaskPriority, dependencies: Vec<TaskDependency>) -> Self {
        use crate::task::types::TaskCategory;
        
        let mut base_task = BaseTask::new(title, TaskCategory::Development);
        base_task.priority = priority;
        Self {
            base_task,
            distribution_id: Uuid::new_v4(),
            dependencies,
            cpu_requirement: 0.5,
            memory_requirement: 0.5,
            assigned_process: None,
        }
    }
    
    pub fn new_with_resources(
        title: String,
        priority: TaskPriority,
        dependencies: Vec<TaskDependency>,
        cpu_requirement: f64,
        memory_requirement: f64,
    ) -> Self {
        use crate::task::types::{TaskCategory};
        
        let mut base_task = BaseTask::new(title, TaskCategory::Development);
        base_task.priority = priority;
        
        Self {
            base_task,
            distribution_id: Uuid::new_v4(),
            dependencies,
            cpu_requirement,
            memory_requirement,
            assigned_process: None,
        }
    }
    
    pub fn depends_on(&self, task_id: &Uuid) -> bool {
        self.dependencies.iter().any(|dep| {
            matches!(dep, TaskDependency::TaskCompletion(id) if id == task_id)
        })
    }
    
    pub fn has_file_dependency(&self, file_path: &str) -> bool {
        self.dependencies.iter().any(|dep| {
            matches!(dep, TaskDependency::FileAccess(path) if path == file_path)
        })
    }
    
    pub fn id(&self) -> &Uuid {
        &self.distribution_id
    }
    
    pub fn priority(&self) -> &TaskPriority {
        &self.base_task.priority
    }
    
    pub fn status(&self) -> &TaskStatus {
        &self.base_task.status
    }
}

#[derive(Debug, Clone)]
pub struct ProcessLoad {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub active_tasks: usize,
}

impl ProcessLoad {
    pub fn calculate_score(&self) -> f64 {
        // 負荷スコア計算（低いほど良い）
        (self.cpu_usage + self.memory_usage) / 2.0 + (self.active_tasks as f64 * 0.1)
    }
    
    pub fn can_handle_task(&self, task: &DistributedTask) -> bool {
        (self.cpu_usage + task.cpu_requirement) <= 1.0 &&
        (self.memory_usage + task.memory_requirement) <= 1.0
    }
}

pub struct TaskDistributor {
    tasks: HashMap<Uuid, DistributedTask>,
    process_loads: HashMap<Uuid, ProcessLoad>,
    task_dependencies: HashMap<Uuid, HashSet<Uuid>>,
    file_locks: HashMap<String, Uuid>, // ファイルパス -> プロセスID
}

impl TaskDistributor {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            process_loads: HashMap::new(),
            task_dependencies: HashMap::new(),
            file_locks: HashMap::new(),
        }
    }
    
    pub fn add_task(&mut self, task: DistributedTask) {
        let task_id = task.distribution_id;
        self.build_dependency_graph(&task);
        self.tasks.insert(task_id, task);
    }
    
    pub fn update_process_load(&mut self, process_id: Uuid, load: ProcessLoad) {
        self.process_loads.insert(process_id, load);
    }
    
    pub fn can_run_parallel(&self, task1: &DistributedTask, task2: &DistributedTask) -> bool {
        // 依存関係チェック
        if task1.depends_on(&task2.distribution_id) || task2.depends_on(&task1.distribution_id) {
            return false;
        }
        
        // ファイルアクセス競合チェック
        for dep1 in &task1.dependencies {
            if let TaskDependency::FileAccess(file1) = dep1 {
                for dep2 in &task2.dependencies {
                    if let TaskDependency::FileAccess(file2) = dep2 {
                        if file1 == file2 {
                            return false;
                        }
                    }
                }
            }
        }
        
        true
    }
    
    pub fn assign_task(&self, task: &DistributedTask) -> Result<Uuid> {
        let mut best_process = None;
        let mut best_score = f64::MAX;
        
        for (process_id, load) in &self.process_loads {
            if !load.can_handle_task(task) {
                continue;
            }
            
            // ファイルロック競合チェック
            let mut has_conflict = false;
            for dep in &task.dependencies {
                if let TaskDependency::FileAccess(file_path) = dep {
                    if let Some(locked_process) = self.file_locks.get(file_path) {
                        if *locked_process != *process_id {
                            has_conflict = true;
                            break;
                        }
                    }
                }
            }
            
            if has_conflict {
                continue;
            }
            
            let score = load.calculate_score();
            if score < best_score {
                best_score = score;
                best_process = Some(*process_id);
            }
        }
        
        best_process.ok_or_else(|| anyhow!("No suitable process found for task assignment"))
    }
    
    pub fn resolve_execution_order(&self) -> Result<Vec<Uuid>> {
        let mut execution_order = Vec::new();
        let mut completed_tasks = HashSet::new();
        let mut remaining_tasks: HashSet<Uuid> = self.tasks.keys().cloned().collect();
        
        // 循環依存検出のためのセット
        let mut visiting = HashSet::new();
        
        while !remaining_tasks.is_empty() {
            let mut ready_tasks = Vec::new();
            
            for &task_id in &remaining_tasks {
                if let Some(task) = self.tasks.get(&task_id) {
                    if self.is_task_ready(task, &completed_tasks) {
                        ready_tasks.push(task_id);
                    }
                }
            }
            
            if ready_tasks.is_empty() {
                // 循環依存の可能性をチェック
                if self.has_circular_dependency(&remaining_tasks, &mut visiting)? {
                    return Err(anyhow!("Circular dependency detected"));
                }
                return Err(anyhow!("Deadlock detected: no tasks can be executed"));
            }
            
            // 優先度順にソート
            ready_tasks.sort_by(|a, b| {
                let task_a = &self.tasks[a];
                let task_b = &self.tasks[b];
                task_b.priority().cmp(task_a.priority())
            });
            
            for task_id in ready_tasks {
                execution_order.push(task_id);
                completed_tasks.insert(task_id);
                remaining_tasks.remove(&task_id);
            }
        }
        
        Ok(execution_order)
    }
    
    pub fn get_next_task(&self) -> Option<DistributedTask> {
        let mut available_tasks: Vec<&DistributedTask> = self.tasks
            .values()
            .filter(|task| *task.status() == TaskStatus::Todo)
            .collect();
        
        if available_tasks.is_empty() {
            return None;
        }
        
        // 優先度順にソート
        available_tasks.sort_by(|a, b| b.priority().cmp(a.priority()));
        
        Some(available_tasks[0].clone())
    }
    
    fn build_dependency_graph(&mut self, task: &DistributedTask) {
        let mut dependencies = HashSet::new();
        
        for dep in &task.dependencies {
            if let TaskDependency::TaskCompletion(dep_task_id) = dep {
                dependencies.insert(*dep_task_id);
            }
        }
        
        self.task_dependencies.insert(task.distribution_id, dependencies);
    }
    
    fn is_task_ready(&self, task: &DistributedTask, completed_tasks: &HashSet<Uuid>) -> bool {
        for dep in &task.dependencies {
            match dep {
                TaskDependency::TaskCompletion(dep_task_id) => {
                    if !completed_tasks.contains(dep_task_id) {
                        return false;
                    }
                }
                TaskDependency::FileAccess(_) => {
                    // ファイルアクセス依存は実行時に解決
                    continue;
                }
                TaskDependency::ResourceAvailability(_) => {
                    // リソース依存は実行時に解決
                    continue;
                }
            }
        }
        true
    }
    
    fn has_circular_dependency(
        &self,
        remaining_tasks: &HashSet<Uuid>,
        visiting: &mut HashSet<Uuid>,
    ) -> Result<bool> {
        for &task_id in remaining_tasks {
            if visiting.contains(&task_id) {
                continue;
            }
            
            if self.dfs_circular_check(task_id, visiting, &mut HashSet::new())? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    
    fn dfs_circular_check(
        &self,
        task_id: Uuid,
        visiting: &mut HashSet<Uuid>,
        visited: &mut HashSet<Uuid>,
    ) -> Result<bool> {
        if visiting.contains(&task_id) {
            return Ok(true); // 循環依存発見
        }
        
        if visited.contains(&task_id) {
            return Ok(false);
        }
        
        visiting.insert(task_id);
        
        if let Some(dependencies) = self.task_dependencies.get(&task_id) {
            for &dep_id in dependencies {
                if self.dfs_circular_check(dep_id, visiting, visited)? {
                    return Ok(true);
                }
            }
        }
        
        visiting.remove(&task_id);
        visited.insert(task_id);
        
        Ok(false)
    }
    
    pub fn lock_file(&mut self, file_path: String, process_id: Uuid) -> Result<()> {
        if let Some(existing_process) = self.file_locks.get(&file_path) {
            if *existing_process != process_id {
                return Err(anyhow!("File {} is already locked by another process", file_path));
            }
        }
        
        self.file_locks.insert(file_path, process_id);
        Ok(())
    }
    
    pub fn unlock_file(&mut self, file_path: &str, process_id: Uuid) -> Result<()> {
        if let Some(existing_process) = self.file_locks.get(file_path) {
            if *existing_process != process_id {
                return Err(anyhow!("Cannot unlock file {}: not locked by this process", file_path));
            }
        }
        
        self.file_locks.remove(file_path);
        Ok(())
    }
    
    pub fn get_parallel_task_groups(&self) -> Vec<Vec<Uuid>> {
        let mut groups = Vec::new();
        let mut processed = HashSet::new();
        
        for task_id in self.tasks.keys() {
            if processed.contains(task_id) {
                continue;
            }
            
            let mut group = vec![*task_id];
            processed.insert(*task_id);
            
            // 並列実行可能なタスクを探す
            for other_task_id in self.tasks.keys() {
                if processed.contains(other_task_id) {
                    continue;
                }
                
                let task = &self.tasks[task_id];
                let other_task = &self.tasks[other_task_id];
                
                if self.can_run_parallel(task, other_task) {
                    // グループ内の他のタスクとも並列実行可能かチェック
                    let mut can_add_to_group = true;
                    for &group_task_id in &group {
                        let group_task = &self.tasks[&group_task_id];
                        if !self.can_run_parallel(group_task, other_task) {
                            can_add_to_group = false;
                            break;
                        }
                    }
                    
                    if can_add_to_group {
                        group.push(*other_task_id);
                        processed.insert(*other_task_id);
                    }
                }
            }
            
            groups.push(group);
        }
        
        groups
    }
}

impl Default for TaskDistributor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_task_creation() {
        let task = DistributedTask::new(
            "Test task".to_string(),
            TaskPriority::Medium,
            vec![]
        );
        
        assert_eq!(task.base_task.title, "Test task");
        assert_eq!(*task.priority(), TaskPriority::Medium);
        assert_eq!(*task.status(), TaskStatus::Todo);
    }
    
    #[test]
    fn test_process_load_calculation() {
        let load = ProcessLoad {
            cpu_usage: 0.6,
            memory_usage: 0.4,
            active_tasks: 2,
        };
        
        let expected_score = (0.6 + 0.4) / 2.0 + (2.0 * 0.1);
        assert_eq!(load.calculate_score(), expected_score);
    }
    
    #[test]
    fn test_task_distributor_creation() {
        let distributor = TaskDistributor::new();
        assert!(distributor.tasks.is_empty());
        assert!(distributor.process_loads.is_empty());
    }
}