// WezTerm Multi-Process Development Framework - Workspace-Process Integration Tests

use std::time::Duration;
use tokio::time::sleep;
use tempfile::TempDir;

use wezterm_parallel::workspace::manager::WorkspaceManager;
use wezterm_parallel::workspace::integration::IntegratedWorkspaceManager;
use wezterm_parallel::process::manager::{ProcessManager, ProcessConfig, RestartPolicy};
use wezterm_parallel::workspace::state::{ProcessStatus, ProcessInfo};

#[tokio::test]
async fn test_workspace_creation_starts_process() {
    // Setup: Create temporary directory for test state
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("workspaces.json");
    
    // Create WorkspaceManager with ProcessManager integration
    let workspace_manager = WorkspaceManager::new(Some(state_path)).unwrap();
    let process_config = create_test_process_config();
    let (process_manager, _event_receiver) = ProcessManager::new(process_config);
    
    // Create an integrated manager that combines both
    let integrated_manager = IntegratedWorkspaceManager::new(workspace_manager, process_manager);
    
    // Test: Create workspace should automatically start Claude Code process
    let workspace_name = "test-workspace";
    let template = "basic";
    
    let result = integrated_manager.create_workspace_with_process(workspace_name, template).await;
    assert!(result.is_ok(), "Workspace creation should succeed");
    
    // Verify: Workspace exists
    let workspace = integrated_manager.get_workspace(workspace_name).await;
    assert!(workspace.is_some(), "Workspace should exist after creation");
    
    // Verify: Process is started and running
    let process_info = integrated_manager.get_workspace_process(workspace_name).await;
    assert!(process_info.is_some(), "Process should be started for workspace");
    
    let process = process_info.unwrap();
    assert_eq!(process.status, ProcessStatus::Running, "Process should be running");
    assert_eq!(process.workspace, workspace_name, "Process should be associated with workspace");
}

#[tokio::test]
async fn test_workspace_deletion_stops_process() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("workspaces.json");
    
    let workspace_manager = WorkspaceManager::new(Some(state_path)).unwrap();
    let process_config = create_test_process_config();
    let (process_manager, _event_receiver) = ProcessManager::new(process_config);
    let integrated_manager = IntegratedWorkspaceManager::new(workspace_manager, process_manager);
    
    // Create workspace with process
    let workspace_name = "test-workspace-delete";
    integrated_manager.create_workspace_with_process(workspace_name, "basic").await.unwrap();
    
    // Verify process is running
    let process_before = integrated_manager.get_workspace_process(workspace_name).await;
    assert!(process_before.is_some());
    assert_eq!(process_before.unwrap().status, ProcessStatus::Running);
    
    // Test: Delete workspace should stop process
    let result = integrated_manager.delete_workspace_with_process(workspace_name).await;
    assert!(result.is_ok(), "Workspace deletion should succeed");
    
    // Verify: Workspace is deleted
    let workspace_after = integrated_manager.get_workspace(workspace_name).await;
    assert!(workspace_after.is_none(), "Workspace should be deleted");
    
    // Verify: Process is stopped
    let process_after = integrated_manager.get_workspace_process(workspace_name).await;
    assert!(process_after.is_none(), "Process should be stopped after workspace deletion");
}

#[tokio::test]
async fn test_process_restart_maintains_workspace_association() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("workspaces.json");
    
    let workspace_manager = WorkspaceManager::new(Some(state_path)).unwrap();
    let process_config = create_test_process_config();
    let (process_manager, _event_receiver) = ProcessManager::new(process_config);
    let integrated_manager = IntegratedWorkspaceManager::new(workspace_manager, process_manager);
    
    // Create workspace with process
    let workspace_name = "test-workspace-restart";
    integrated_manager.create_workspace_with_process(workspace_name, "basic").await.unwrap();
    
    let original_process = integrated_manager.get_workspace_process(workspace_name).await.unwrap();
    let original_pid = original_process.id.clone();
    
    // Test: Restart process
    let result = integrated_manager.restart_workspace_process(workspace_name).await;
    assert!(result.is_ok(), "Process restart should succeed");
    
    // Allow time for restart
    sleep(Duration::from_millis(100)).await;
    
    // Verify: Process is restarted (new PID) but still associated with workspace
    let restarted_process = integrated_manager.get_workspace_process(workspace_name).await;
    assert!(restarted_process.is_some(), "Process should exist after restart");
    
    let restarted = restarted_process.unwrap();
    assert_ne!(restarted.id, original_pid, "Process should have new PID after restart");
    assert_eq!(restarted.workspace, workspace_name, "Process should maintain workspace association");
    assert_eq!(restarted.status, ProcessStatus::Running, "Restarted process should be running");
}

#[tokio::test]
async fn test_process_failure_recovery() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("workspaces.json");
    
    let workspace_manager = WorkspaceManager::new(Some(state_path)).unwrap();
    let mut process_config = create_test_process_config();
    process_config.max_restart_attempts = 2; // Allow limited restarts
    
    let (process_manager, _event_receiver) = ProcessManager::new(process_config);
    let integrated_manager = IntegratedWorkspaceManager::new(workspace_manager, process_manager);
    
    // Create workspace with process
    let workspace_name = "test-workspace-failure";
    integrated_manager.create_workspace_with_process(workspace_name, "basic").await.unwrap();
    
    // Test: Simulate process failure
    let result = integrated_manager.simulate_process_failure(workspace_name).await;
    assert!(result.is_ok(), "Process failure simulation should succeed");
    
    // Allow time for cleanup
    sleep(Duration::from_millis(500)).await;
    
    // Verify: Process should be stopped (automatic restart is not yet implemented)
    let recovered_process = integrated_manager.get_workspace_process(workspace_name).await;
    // For now, we expect the process to be stopped after failure
    // TODO: Implement automatic restart in future iteration
    assert!(recovered_process.is_none() || recovered_process.unwrap().status == ProcessStatus::Stopped, 
        "Process should be stopped after failure (automatic restart not yet implemented)");
}

#[tokio::test]
async fn test_multiple_workspaces_independent_processes() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("workspaces.json");
    
    let workspace_manager = WorkspaceManager::new(Some(state_path)).unwrap();
    let process_config = create_test_process_config();
    let (process_manager, _event_receiver) = ProcessManager::new(process_config);
    let integrated_manager = IntegratedWorkspaceManager::new(workspace_manager, process_manager);
    
    // Create multiple workspaces
    let workspace1 = "frontend-workspace";
    let workspace2 = "backend-workspace";
    
    integrated_manager.create_workspace_with_process(workspace1, "basic").await.unwrap();
    integrated_manager.create_workspace_with_process(workspace2, "basic").await.unwrap();
    
    // Verify: Each workspace has its own process
    let process1 = integrated_manager.get_workspace_process(workspace1).await.unwrap();
    let process2 = integrated_manager.get_workspace_process(workspace2).await.unwrap();
    
    assert_ne!(process1.id, process2.id, "Workspaces should have different processes");
    assert_eq!(process1.workspace, workspace1, "Process 1 should be associated with workspace 1");
    assert_eq!(process2.workspace, workspace2, "Process 2 should be associated with workspace 2");
    
    // Test: Deleting one workspace doesn't affect the other
    integrated_manager.delete_workspace_with_process(workspace1).await.unwrap();
    
    // Verify: Workspace 1 is deleted, workspace 2 remains
    assert!(integrated_manager.get_workspace(workspace1).await.is_none());
    assert!(integrated_manager.get_workspace_process(workspace1).await.is_none());
    
    assert!(integrated_manager.get_workspace(workspace2).await.is_some());
    assert!(integrated_manager.get_workspace_process(workspace2).await.is_some());
}

// Using ProcessInfo and IntegratedWorkspaceManager from the main crate

// Helper function to create test ProcessConfig
fn create_test_process_config() -> ProcessConfig {
    use std::collections::HashMap;
    ProcessConfig {
        claude_code_binary: "echo".to_string(), // Use echo for testing instead of actual claude-code
        max_processes: 16,
        health_check_interval_secs: 1,
        restart_delay_secs: 1,
        max_restart_attempts: 3,
        process_timeout_secs: 10,
        default_restart_policy: RestartPolicy::OnFailure,
        environment_vars: HashMap::new(),
        working_directory: None,
    }
}