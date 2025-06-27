// WezTerm Multi-Process Development Framework - End-to-End Integration Tests
// Tests the complete system integration including IPC, WebSocket, and workspace management

use std::time::Duration;
use tokio::time::{timeout, sleep};
use tempfile::TempDir;
use wezterm_parallel::{
    workspace::{WorkspaceManager, IntegratedWorkspaceManager},
    process::manager::{ProcessManager, ProcessConfig},
    dashboard::{WebSocketServer, DashboardConfig},
    Message,
};

#[tokio::test]
async fn test_complete_workspace_lifecycle() {
    // Setup test environment
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("test_workspaces.json");
    
    // Initialize workspace manager
    let workspace_manager = WorkspaceManager::new(Some(state_path)).unwrap();
    
    // Test workspace creation
    let result = workspace_manager.create_workspace("test-e2e", "basic").await;
    assert!(result.is_ok());
    
    // Test workspace listing
    let workspaces = workspace_manager.list_workspaces().await;
    assert!(!workspaces.is_empty());
    
    // Test workspace deletion
    let result = workspace_manager.delete_workspace("test-e2e").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_integrated_workspace_process_management() {
    // Setup test environment
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("test_workspaces.json");
    
    let workspace_manager = WorkspaceManager::new(Some(state_path)).unwrap();
    
    let config = ProcessConfig {
        claude_code_binary: "echo".to_string(),
        max_processes: 10,
        health_check_interval_secs: 1,
        restart_delay_secs: 1,
        max_restart_attempts: 3,
        process_timeout_secs: 30,
        default_restart_policy: wezterm_parallel::process::manager::RestartPolicy::OnFailure,
        environment_vars: std::collections::HashMap::new(),
        working_directory: None,
    };
    
    let (process_manager, _event_receiver) = ProcessManager::new(config);
    
    // Create integrated manager
    let integrated_manager = IntegratedWorkspaceManager::new(workspace_manager, process_manager)
        .with_monitoring(true, Duration::from_millis(100));
    
    // Test workspace creation with process
    let result = integrated_manager.create_workspace_with_process("e2e-test", "basic").await;
    assert!(result.is_ok());
    
    // Test workspace health status
    let health_status = integrated_manager.get_workspace_health_status().await;
    assert!(!health_status.is_empty());
    
    // Test workspace count
    assert_eq!(integrated_manager.get_workspace_count().await, 1);
    
    // Test workspace deletion with process cleanup
    let result = integrated_manager.delete_workspace_with_process("e2e-test").await;
    assert!(result.is_ok());
    
    // Verify cleanup
    assert_eq!(integrated_manager.get_workspace_count().await, 0);
}

#[tokio::test]
async fn test_websocket_dashboard_integration() {
    // Setup WebSocket server
    let config = DashboardConfig {
        port: 9994, // Use different port for test isolation
        enabled: true,
        update_interval: 100,
        max_clients: 5,
        auth_enabled: false,
        auth_token: None,
        compression: false,
    };
    
    let (websocket_server, _metrics_tx) = WebSocketServer::new(config);
    
    // Test server statistics
    let stats = websocket_server.get_stats().await;
    assert_eq!(stats.connected_clients, 0);
    assert_eq!(stats.total_workspaces, 0);
    assert_eq!(stats.total_processes, 0);
    
    // Test server state
    let state = websocket_server.get_state();
    let client_count = state.client_count().await;
    assert_eq!(client_count, 0);
}

#[tokio::test]
async fn test_metrics_update_flow() {
    // Setup WebSocket server
    let config = DashboardConfig {
        port: 9993,
        enabled: true,
        update_interval: 50,
        max_clients: 5,
        auth_enabled: false,
        auth_token: None,
        compression: false,
    };
    
    let (websocket_server, metrics_tx) = WebSocketServer::new(config);
    
    // Send metrics update
    let update = wezterm_parallel::dashboard::MetricsUpdate {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        system: Some(wezterm_parallel::metrics::SystemMetrics::new()),
        processes: Vec::new(),
        workspaces: Vec::new(),
        framework: Some(wezterm_parallel::metrics::FrameworkMetrics::new()),
        update_type: wezterm_parallel::dashboard::UpdateType::Full,
    };
    
    let result = metrics_tx.send(update).await;
    assert!(result.is_ok());
    
    // Verify metrics were processed
    let stats = websocket_server.get_stats().await;
    assert!(stats.last_update > 0);
}

#[tokio::test]
async fn test_message_serialization_compatibility() {
    // Test various message types for IPC compatibility
    
    // Workspace create message
    let workspace_msg = Message::WorkspaceCreate {
        name: "test-workspace".to_string(),
        template: "basic".to_string(),
    };
    
    let serialized = serde_json::to_string(&workspace_msg).unwrap();
    let deserialized: Message = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        Message::WorkspaceCreate { name, template } => {
            assert_eq!(name, "test-workspace");
            assert_eq!(template, "basic");
        }
        _ => panic!("Message type mismatch"),
    }
    
    // Process spawn message
    let process_msg = Message::ProcessSpawn {
        workspace: "test-workspace".to_string(),
        command: "echo hello".to_string(),
    };
    
    let serialized = serde_json::to_string(&process_msg).unwrap();
    let deserialized: Message = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        Message::ProcessSpawn { workspace, command } => {
            assert_eq!(workspace, "test-workspace");
            assert_eq!(command, "echo hello");
        }
        _ => panic!("Message type mismatch"),
    }
    
    // Status update message
    let status_msg = Message::StatusUpdate {
        process_id: "test-process".to_string(),
        status: "running".to_string(),
    };
    
    let serialized = serde_json::to_string(&status_msg).unwrap();
    let deserialized: Message = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        Message::StatusUpdate { process_id, status } => {
            assert_eq!(process_id, "test-process");
            assert_eq!(status, "running");
        }
        _ => panic!("Message type mismatch"),
    }
}

#[tokio::test]
async fn test_concurrent_workspace_operations() {
    // Test concurrent workspace operations for thread safety
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("concurrent_test_workspaces.json");
    
    let workspace_manager = std::sync::Arc::new(
        WorkspaceManager::new(Some(state_path)).unwrap()
    );
    
    // Create multiple workspaces concurrently
    let mut tasks = Vec::new();
    
    for i in 0..5 {
        let manager = std::sync::Arc::clone(&workspace_manager);
        let workspace_name = format!("concurrent-workspace-{}", i);
        
        let task = tokio::spawn(async move {
            let result = manager.create_workspace(&workspace_name, "basic").await;
            assert!(result.is_ok());
            
            // Give some time for state synchronization
            sleep(Duration::from_millis(10)).await;
            
            let result = manager.delete_workspace(&workspace_name).await;
            assert!(result.is_ok());
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    for task in tasks {
        let result = timeout(Duration::from_secs(5), task).await;
        assert!(result.is_ok());
    }
    
    // Verify final state
    let workspaces = workspace_manager.list_workspaces().await;
    // Should have at most the default workspace
    assert!(workspaces.len() <= 1);
}

#[tokio::test]
async fn test_error_handling_and_recovery() {
    // Test error conditions and recovery mechanisms
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("error_test_workspaces.json");
    
    let workspace_manager = WorkspaceManager::new(Some(state_path)).unwrap();
    
    // Test creating workspace with invalid template
    let result = workspace_manager.create_workspace("test-invalid", "nonexistent-template").await;
    assert!(result.is_err());
    
    // Test deleting non-existent workspace
    let result = workspace_manager.delete_workspace("non-existent-workspace").await;
    assert!(result.is_err());
    
    // Test that system recovers and can still create valid workspaces
    let result = workspace_manager.create_workspace("test-recovery", "basic").await;
    assert!(result.is_ok());
    
    let workspaces = workspace_manager.list_workspaces().await;
    assert!(!workspaces.is_empty());
}

#[tokio::test]
async fn test_system_performance_under_load() {
    // Test system performance under moderate load
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("performance_test_workspaces.json");
    
    let workspace_manager = std::sync::Arc::new(
        WorkspaceManager::new(Some(state_path)).unwrap()
    );
    
    let start_time = std::time::Instant::now();
    
    // Create and manage multiple workspaces rapidly (within limits)
    for i in 0..7 { // Limit to avoid hitting workspace limit
        let workspace_name = format!("perf-test-{}", i);
        
        let result = workspace_manager.create_workspace(&workspace_name, "basic").await;
        if let Err(e) = &result {
            eprintln!("Failed to create workspace {}: {}", workspace_name, e);
        }
        assert!(result.is_ok());
        
        // Simulate some workspace operations
        let _workspaces = workspace_manager.list_workspaces().await;
        
        if i % 2 == 0 && workspace_name != "default" {
            let result = workspace_manager.delete_workspace(&workspace_name).await;
            assert!(result.is_ok());
        }
    }
    
    let elapsed = start_time.elapsed();
    
    // Operations should complete within reasonable time (< 5 seconds)
    assert!(elapsed < Duration::from_secs(5));
    
    // Verify system is still responsive
    let _workspaces = workspace_manager.list_workspaces().await;
    // Just verify it responds without error
}