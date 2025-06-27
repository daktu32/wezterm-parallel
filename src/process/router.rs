use crate::{CoordinationMessage, CoordinationResponse};
use crate::process::ProcessManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use std::error::Error;

/// プロセス間メッセージのルーティングを管理
pub struct MessageRouter {
    /// 登録されたプロセスマネージャー
    processes: Arc<RwLock<HashMap<String, Arc<Mutex<ProcessManager>>>>>,
}

impl MessageRouter {
    /// 新しいメッセージルーターを作成
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// プロセスを登録
    pub async fn register_process(
        &self,
        process_id: String,
        manager: Arc<Mutex<ProcessManager>>,
    ) {
        let mut processes = self.processes.write().await;
        processes.insert(process_id, manager);
    }

    /// プロセスの登録を解除
    pub async fn unregister_process(&self, process_id: &str) {
        let mut processes = self.processes.write().await;
        processes.remove(process_id);
    }

    /// メッセージをルーティング
    pub async fn route_message(
        &self,
        message: CoordinationMessage,
    ) -> Result<CoordinationResponse, Box<dyn Error + Send + Sync>> {
        let processes = self.processes.read().await;
        
        // 宛先プロセスを取得
        let _receiver = processes
            .get(&message.receiver_id)
            .ok_or_else(|| format!("Process {} not found", message.receiver_id))?;
        
        // TODO: 実際のメッセージ送信とレスポンス処理を実装
        // 現在はモックレスポンスを返す
        Ok(CoordinationResponse::Acknowledged {
            process_id: message.receiver_id.clone(),
        })
    }

    /// 複数のプロセスにメッセージをブロードキャスト
    pub async fn broadcast_message(
        &self,
        message: CoordinationMessage,
        exclude_sender: bool,
    ) -> Vec<(String, Result<CoordinationResponse, String>)> {
        let processes = self.processes.read().await;
        let mut responses = Vec::new();

        for (process_id, _manager) in processes.iter() {
            if exclude_sender && process_id == &message.sender_id {
                continue;
            }

            let _msg_clone = CoordinationMessage {
                sender_id: message.sender_id.clone(),
                receiver_id: process_id.clone(),
                timestamp: message.timestamp,
                event: message.event.clone(),
            };

            // TODO: 実際のメッセージ送信とレスポンス処理を実装
            let response = Ok(CoordinationResponse::Acknowledged {
                process_id: process_id.clone(),
            });

            responses.push((process_id.clone(), response));
        }

        responses
    }

    /// 登録されているプロセスのIDリストを取得
    pub async fn get_registered_processes(&self) -> Vec<String> {
        let processes = self.processes.read().await;
        processes.keys().cloned().collect()
    }

    /// 特定のプロセスが登録されているか確認
    pub async fn is_process_registered(&self, process_id: &str) -> bool {
        let processes = self.processes.read().await;
        processes.contains_key(process_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CoordinationEvent;
    use crate::process::ProcessConfig;

    #[tokio::test]
    async fn test_process_registration() {
        let router = MessageRouter::new();
        let config = ProcessConfig::default();
        let (manager1, _) = ProcessManager::new(config.clone());
        let (manager2, _) = ProcessManager::new(config);
        
        router.register_process(
            "process-1".to_string(),
            Arc::new(Mutex::new(manager1)),
        ).await;
        
        router.register_process(
            "process-2".to_string(),
            Arc::new(Mutex::new(manager2)),
        ).await;
        
        assert!(router.is_process_registered("process-1").await);
        assert!(router.is_process_registered("process-2").await);
        assert!(!router.is_process_registered("process-3").await);
    }

    #[tokio::test]
    async fn test_message_routing() {
        let router = MessageRouter::new();
        let config = ProcessConfig::default();
        let (manager, _) = ProcessManager::new(config);
        
        router.register_process(
            "receiver".to_string(),
            Arc::new(Mutex::new(manager)),
        ).await;
        
        let message = CoordinationMessage::new(
            "sender".to_string(),
            "receiver".to_string(),
            CoordinationEvent::TaskAssignment {
                task_id: "task-1".to_string(),
                description: "Test task".to_string(),
            },
        );
        
        let response = router.route_message(message).await.unwrap();
        
        match response {
            CoordinationResponse::Acknowledged { process_id } => {
                assert_eq!(process_id, "receiver");
            }
            _ => panic!("Expected Acknowledged response"),
        }
    }

    #[tokio::test]
    async fn test_broadcast_message() {
        let router = MessageRouter::new();
        let config = ProcessConfig::default();
        
        for i in 1..=3 {
            let (manager, _) = ProcessManager::new(config.clone());
            router.register_process(
                format!("process-{}", i),
                Arc::new(Mutex::new(manager)),
            ).await;
        }
        
        let message = CoordinationMessage::new(
            "process-1".to_string(),
            "".to_string(), // Broadcast doesn't need specific receiver
            CoordinationEvent::GlobalCommand {
                command: "pause".to_string(),
                parameters: vec![],
            },
        );
        
        let responses = router.broadcast_message(message, true).await;
        
        // process-1を除外するので、2つのレスポンスが返るはず
        assert_eq!(responses.len(), 2);
        
        for (process_id, response) in responses {
            assert_ne!(process_id, "process-1");
            assert!(response.is_ok());
        }
    }

    #[tokio::test]
    async fn test_unregister_process() {
        let router = MessageRouter::new();
        let config = ProcessConfig::default();
        let (manager, _) = ProcessManager::new(config);
        
        router.register_process(
            "temp-process".to_string(),
            Arc::new(Mutex::new(manager)),
        ).await;
        
        assert!(router.is_process_registered("temp-process").await);
        
        router.unregister_process("temp-process").await;
        
        assert!(!router.is_process_registered("temp-process").await);
    }

    #[tokio::test]
    async fn test_route_to_nonexistent_process() {
        let router = MessageRouter::new();
        
        let message = CoordinationMessage::new(
            "sender".to_string(),
            "nonexistent".to_string(),
            CoordinationEvent::TaskAssignment {
                task_id: "task-1".to_string(),
                description: "Test task".to_string(),
            },
        );
        
        let result = router.route_message(message).await;
        assert!(result.is_err());
    }
}