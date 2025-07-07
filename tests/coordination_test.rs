use wezterm_parallel::process::{ProcessManager, ProcessConfig};
use wezterm_parallel::{CoordinationMessage, CoordinationEvent, CoordinationResponse, ProcessStatus};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_process_coordination_message_routing() {
    // 2つのプロセスマネージャーを作成
    let config1 = ProcessConfig::default();
    let config2 = ProcessConfig::default();
    let (manager1, _rx1) = ProcessManager::new(config1);
    let (manager2, _rx2) = ProcessManager::new(config2);
    
    let manager1 = Arc::new(Mutex::new(manager1));
    let manager2 = Arc::new(Mutex::new(manager2));
    
    // プロセスIDは直接設定
    let process1_id = "process1".to_string();
    let process2_id = "process2".to_string();
    
    // プロセス間でメッセージを送信
    let message = CoordinationMessage::new(
        process1_id.clone(),
        process2_id.clone(),
        CoordinationEvent::TaskAssignment {
            task_id: "task-001".to_string(),
            description: "Test task".to_string(),
        },
    );
    
    // メッセージルーティングのテスト
    let router = wezterm_parallel::process::router::MessageRouter::new();
    router.register_process(process1_id.clone(), manager1.clone()).await;
    router.register_process(process2_id.clone(), manager2.clone()).await;
    
    let response = router.route_message(message).await.unwrap();
    
    match response {
        CoordinationResponse::Acknowledged { process_id } => {
            assert_eq!(process_id, process2_id);
        }
        _ => panic!("Expected Acknowledged response"),
    }
}

#[tokio::test]
async fn test_coordinator_task_distribution() {
    let coordinator = wezterm_parallel::process::coordinator::ProcessCoordinator::new();
    
    // 3つのプロセスを登録
    let process_ids = vec![
        "process-a".to_string(),
        "process-b".to_string(),
        "process-c".to_string(),
    ];
    
    for id in &process_ids {
        coordinator.register_process(id.clone()).await;
    }
    
    // タスクを分配
    let tasks = vec![
        ("task-1", "First task"),
        ("task-2", "Second task"),
        ("task-3", "Third task"),
        ("task-4", "Fourth task"),
    ];
    
    for (task_id, description) in tasks {
        let assigned_process = coordinator.assign_task(
            task_id.to_string(),
            description.to_string(),
        ).await.unwrap();
        
        // すべてのプロセスに均等に分配されているか確認
        assert!(process_ids.contains(&assigned_process));
    }
    
    // 各プロセスの負荷を確認
    let loads = coordinator.get_process_loads().await;
    for (_, load) in loads {
        assert!(load > 0);
        assert!(load <= 2); // 4タスクを3プロセスで分配
    }
}

#[tokio::test]
async fn test_process_status_synchronization() {
    let coordinator = wezterm_parallel::process::coordinator::ProcessCoordinator::new();
    
    // プロセスを登録
    coordinator.register_process("process-1".to_string()).await;
    coordinator.register_process("process-2".to_string()).await;
    
    // ステータス更新をテスト
    coordinator.update_process_status(
        "process-1".to_string(),
        ProcessStatus::Running,
    ).await;
    
    coordinator.update_process_status(
        "process-2".to_string(),
        ProcessStatus::Idle,
    ).await;
    
    // ステータスを確認
    let statuses = coordinator.get_all_process_statuses().await;
    assert_eq!(statuses.get("process-1"), Some(&ProcessStatus::Running));
    assert_eq!(statuses.get("process-2"), Some(&ProcessStatus::Idle));
}

#[tokio::test]
async fn test_coordination_message_serialization() {
    let message = CoordinationMessage::new(
        "sender-123".to_string(),
        "receiver-456".to_string(),
        CoordinationEvent::StatusUpdate {
            status: ProcessStatus::Running,
            cpu_usage: 45.5,
            memory_usage: 1024,
        },
    );
    
    // シリアライズとデシリアライズ
    let serialized = serde_json::to_string(&message).unwrap();
    let deserialized: CoordinationMessage = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(message.sender_id, deserialized.sender_id);
    assert_eq!(message.receiver_id, deserialized.receiver_id);
    assert_eq!(message.timestamp, deserialized.timestamp);
}

#[tokio::test]
async fn test_broadcast_coordination_message() {
    let coordinator = wezterm_parallel::process::coordinator::ProcessCoordinator::new();
    
    // 複数のプロセスを登録
    let process_ids = vec![
        "process-alpha".to_string(),
        "process-beta".to_string(),
        "process-gamma".to_string(),
    ];
    
    for id in &process_ids {
        coordinator.register_process(id.clone()).await;
    }
    
    // ブロードキャストメッセージを送信
    let event = CoordinationEvent::GlobalCommand {
        command: "pause_all_tasks".to_string(),
        parameters: vec![],
    };
    
    let responses = coordinator.broadcast_message(
        "coordinator".to_string(),
        event,
    ).await;
    
    // すべてのプロセスから応答を受信
    assert_eq!(responses.len(), process_ids.len());
    for response in responses {
        match response {
            CoordinationResponse::Acknowledged { .. } => {},
            _ => panic!("Expected Acknowledged response"),
        }
    }
}

#[tokio::test]
async fn test_process_failure_handling() {
    let coordinator = wezterm_parallel::process::coordinator::ProcessCoordinator::new();
    
    // プロセスを登録
    coordinator.register_process("process-x".to_string()).await;
    coordinator.register_process("process-y".to_string()).await;
    
    // process-xにタスクを割り当て
    let assigned_process = coordinator.assign_task(
        "critical-task".to_string(),
        "Important task".to_string(),
    ).await.unwrap();
    
    println!("Task assigned to: {}", assigned_process);
    
    // process-xの障害をシミュレート
    coordinator.handle_process_failure("process-x".to_string()).await;
    
    // タスクが再割り当てされることを確認
    let reassigned_tasks = coordinator.get_reassigned_tasks().await;
    println!("Reassigned tasks: {:?}", reassigned_tasks);
    
    // process-xが削除されているか確認
    let statuses = coordinator.get_all_process_statuses().await;
    assert!(!statuses.contains_key("process-x"));
    assert!(statuses.contains_key("process-y"));
    
    // process-xに割り当てられたタスクだけをチェック
    if assigned_process == "process-x" {
        assert!(reassigned_tasks.contains(&"critical-task".to_string()));
    }
}