use crate::{CoordinationEvent, CoordinationResponse, ProcessStatus};
use crate::task::{TaskDistributor, Task, ProcessLoad};
use crate::sync::FileSyncManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::error::Error;
use uuid::Uuid;

/// プロセス協調のためのコーディネーター
pub struct ProcessCoordinator {
    /// 登録されたプロセスの状態
    processes: Arc<RwLock<HashMap<String, ProcessState>>>,
    /// タスクの割り当て状況
    task_assignments: Arc<RwLock<HashMap<String, String>>>,
    /// 再割り当てが必要なタスク
    reassigned_tasks: Arc<RwLock<Vec<String>>>,
    /// タスク分散マネージャー
    task_distributor: Arc<RwLock<TaskDistributor>>,
    /// ファイル同期マネージャー
    file_sync_manager: Arc<RwLock<FileSyncManager>>,
}

#[derive(Debug, Clone)]
struct ProcessState {
    id: String,
    status: ProcessStatus,
    task_count: usize,
    cpu_usage: f64,
    memory_usage: u64,
    uuid: Uuid,
}

impl ProcessCoordinator {
    /// 新しいコーディネーターを作成
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            task_assignments: Arc::new(RwLock::new(HashMap::new())),
            reassigned_tasks: Arc::new(RwLock::new(Vec::new())),
            task_distributor: Arc::new(RwLock::new(TaskDistributor::new())),
            file_sync_manager: Arc::new(RwLock::new(FileSyncManager::new())),
        }
    }

    /// プロセスを登録
    pub async fn register_process(&self, process_id: String) {
        let process_uuid = Uuid::new_v4();
        let mut processes = self.processes.write().await;
        processes.insert(
            process_id.clone(),
            ProcessState {
                id: process_id.clone(),
                status: ProcessStatus::Idle,
                task_count: 0,
                cpu_usage: 0.0,
                memory_usage: 0,
                uuid: process_uuid,
            },
        );
        
        // ファイル同期マネージャーにプロセスを登録
        let mut file_sync = self.file_sync_manager.write().await;
        file_sync.register_process(process_uuid);
    }

    /// タスクを割り当て
    pub async fn assign_task(
        &self,
        task_id: String,
        _description: String,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut processes = self.processes.write().await;
        let mut task_assignments = self.task_assignments.write().await;

        // 最も負荷の低いプロセスを選択
        let selected_process = processes
            .values_mut()
            .filter(|p| matches!(p.status, ProcessStatus::Idle | ProcessStatus::Running))
            .min_by_key(|p| p.task_count)
            .ok_or("No available processes")?;

        selected_process.task_count += 1;
        let process_id = selected_process.id.clone();
        
        task_assignments.insert(task_id.clone(), process_id.clone());
        
        Ok(process_id)
    }

    /// プロセスの負荷を取得
    pub async fn get_process_loads(&self) -> HashMap<String, usize> {
        let processes = self.processes.read().await;
        processes
            .iter()
            .map(|(id, state)| (id.clone(), state.task_count))
            .collect()
    }

    /// プロセスのステータスを更新
    pub async fn update_process_status(&self, process_id: String, status: ProcessStatus) {
        let mut processes = self.processes.write().await;
        if let Some(process) = processes.get_mut(&process_id) {
            process.status = status;
        }
    }

    /// すべてのプロセスのステータスを取得
    pub async fn get_all_process_statuses(&self) -> HashMap<String, ProcessStatus> {
        let processes = self.processes.read().await;
        processes
            .iter()
            .map(|(id, state)| (id.clone(), state.status.clone()))
            .collect()
    }

    /// メッセージをブロードキャスト
    pub async fn broadcast_message(
        &self,
        sender_id: String,
        _event: CoordinationEvent,
    ) -> Vec<CoordinationResponse> {
        let processes = self.processes.read().await;
        let mut responses = Vec::new();

        for (process_id, _) in processes.iter() {
            if process_id != &sender_id {
                // 実際の実装では、ここでメッセージを送信し、レスポンスを待つ
                responses.push(CoordinationResponse::Acknowledged {
                    process_id: process_id.clone(),
                });
            }
        }

        responses
    }

    /// プロセスの障害を処理
    pub async fn handle_process_failure(&self, failed_process_id: String) {
        let mut processes = self.processes.write().await;
        let mut task_assignments = self.task_assignments.write().await;
        let mut reassigned_tasks = self.reassigned_tasks.write().await;

        // 失敗したプロセスを削除
        processes.remove(&failed_process_id);

        // 失敗したプロセスに割り当てられていたタスクを特定
        let failed_tasks: Vec<String> = task_assignments
            .iter()
            .filter_map(|(task_id, process_id)| {
                if process_id == &failed_process_id {
                    Some(task_id.clone())
                } else {
                    None
                }
            })
            .collect();

        // タスクを再割り当てリストに追加
        for task_id in failed_tasks {
            task_assignments.remove(&task_id);
            reassigned_tasks.push(task_id);
        }
    }

    /// 再割り当てが必要なタスクを取得
    pub async fn get_reassigned_tasks(&self) -> Vec<String> {
        let reassigned_tasks = self.reassigned_tasks.read().await;
        reassigned_tasks.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_registration() {
        let coordinator = ProcessCoordinator::new();
        
        coordinator.register_process("process-1".to_string()).await;
        coordinator.register_process("process-2".to_string()).await;
        
        let statuses = coordinator.get_all_process_statuses().await;
        assert_eq!(statuses.len(), 2);
        assert!(statuses.contains_key("process-1"));
        assert!(statuses.contains_key("process-2"));
    }

    #[tokio::test]
    async fn test_task_assignment() {
        let coordinator = ProcessCoordinator::new();
        
        coordinator.register_process("process-a".to_string()).await;
        coordinator.register_process("process-b".to_string()).await;
        
        let assigned1 = coordinator
            .assign_task("task-1".to_string(), "First task".to_string())
            .await
            .unwrap();
        
        let assigned2 = coordinator
            .assign_task("task-2".to_string(), "Second task".to_string())
            .await
            .unwrap();
        
        assert!(assigned1 == "process-a" || assigned1 == "process-b");
        assert!(assigned2 == "process-a" || assigned2 == "process-b");
    }

    #[tokio::test]
    async fn test_load_balancing() {
        let coordinator = ProcessCoordinator::new();
        
        coordinator.register_process("process-1".to_string()).await;
        coordinator.register_process("process-2".to_string()).await;
        coordinator.register_process("process-3".to_string()).await;
        
        // 6つのタスクを割り当て
        for i in 0..6 {
            coordinator
                .assign_task(format!("task-{}", i), format!("Task {}", i))
                .await
                .unwrap();
        }
        
        let loads = coordinator.get_process_loads().await;
        
        // 各プロセスの負荷が2になるはず（6タスク / 3プロセス）
        for (_, load) in loads {
            assert_eq!(load, 2);
        }
    }

    #[tokio::test]
    async fn test_process_failure_handling() {
        let coordinator = ProcessCoordinator::new();
        
        coordinator.register_process("process-x".to_string()).await;
        coordinator.register_process("process-y".to_string()).await;
        
        // process-xにタスクを割り当て
        let assigned_process = coordinator
            .assign_task("critical-task".to_string(), "Important task".to_string())
            .await
            .unwrap();
        
        // process-xの障害を処理
        coordinator.handle_process_failure("process-x".to_string()).await;
        
        // タスクが再割り当てリストに入っているか確認
        let reassigned = coordinator.get_reassigned_tasks().await;
        
        // process-xに割り当てられたタスクだけをチェック
        if assigned_process == "process-x" {
            assert!(reassigned.contains(&"critical-task".to_string()));
        }
        
        // process-xが削除されているか確認
        let statuses = coordinator.get_all_process_statuses().await;
        assert!(!statuses.contains_key("process-x"));
        assert!(statuses.contains_key("process-y"));
    }
}