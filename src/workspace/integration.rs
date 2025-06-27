// WezTerm Multi-Process Development Framework - Workspace-Process Integration

use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::workspace::manager::WorkspaceManager;
use crate::process::manager::ProcessManager;
use crate::workspace::state::{ProcessInfo, ProcessStatus};

/// Integrated manager that combines WorkspaceManager and ProcessManager
/// to provide unified workspace-process lifecycle management
#[derive(Debug)]
pub struct IntegratedWorkspaceManager {
    workspace_manager: WorkspaceManager,
    process_manager: ProcessManager,
    workspace_process_mapping: RwLock<HashMap<String, String>>, // workspace_name -> process_id
    monitoring_enabled: bool,
    health_check_interval: Duration,
}

impl IntegratedWorkspaceManager {
    pub fn new(workspace_manager: WorkspaceManager, process_manager: ProcessManager) -> Self {
        Self {
            workspace_manager,
            process_manager,
            workspace_process_mapping: RwLock::new(HashMap::new()),
            monitoring_enabled: true,
            health_check_interval: Duration::from_secs(30),
        }
    }

    pub fn with_monitoring(mut self, enabled: bool, interval: Duration) -> Self {
        self.monitoring_enabled = enabled;
        self.health_check_interval = interval;
        self
    }

    /// Create a workspace and automatically start a Claude Code process for it
    pub async fn create_workspace_with_process(
        &self,
        name: &str,
        template: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Creating workspace '{}' with template '{}'", name, template);

        // 1. Create the workspace first
        self.workspace_manager.create_workspace(name, template).await?;

        // 2. Generate a unique process ID for this workspace
        let process_id = format!("claude-code-{}", name);
        
        // 3. Start a Claude Code process for this workspace
        let command_args = vec![
            "--workspace".to_string(),
            name.to_string(),
        ];

        match self.process_manager.spawn_process(
            process_id.clone(),
            name.to_string(),
            command_args,
        ).await {
            Ok(_) => {
                // 4. Record the workspace-process mapping
                let mut mapping = self.workspace_process_mapping.write().await;
                mapping.insert(name.to_string(), process_id.clone());
                
                info!("Successfully created workspace '{}' with process '{}'", name, process_id);
                Ok(())
            }
            Err(e) => {
                // If process creation fails, clean up the workspace
                warn!("Failed to start process for workspace '{}': {}", name, e);
                let _ = self.workspace_manager.delete_workspace(name).await;
                Err(format!("Failed to start process for workspace: {}", e).into())
            }
        }
    }

    /// Delete a workspace and stop its associated process
    pub async fn delete_workspace_with_process(
        &self,
        name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Deleting workspace '{}' and its process", name);

        // 1. Get the associated process ID
        let process_id = {
            let mapping = self.workspace_process_mapping.read().await;
            mapping.get(name).cloned()
        };

        // 2. Stop the process if it exists
        if let Some(process_id) = process_id {
            if let Err(e) = self.process_manager.kill_process(&process_id).await {
                warn!("Failed to stop process '{}' for workspace '{}': {}", process_id, name, e);
            }
        }

        // 3. Delete the workspace
        self.workspace_manager.delete_workspace(name).await?;

        // 4. Remove the mapping
        let mut mapping = self.workspace_process_mapping.write().await;
        mapping.remove(name);

        info!("Successfully deleted workspace '{}' and its process", name);
        Ok(())
    }

    /// Get workspace information
    pub async fn get_workspace(&self, name: &str) -> Option<()> {
        // Simplified for testing - just check if workspace exists
        self.workspace_manager.get_workspace_info(name).await.map(|_| ())
    }

    /// Get the process associated with a workspace
    pub async fn get_workspace_process(&self, name: &str) -> Option<ProcessInfo> {
        // 1. Get the process ID for this workspace
        let process_id = {
            let mapping = self.workspace_process_mapping.read().await;
            mapping.get(name).cloned()?
        };

        // 2. Get the process information
        self.process_manager.get_process_info(&process_id).await
    }

    /// Restart the process associated with a workspace
    pub async fn restart_workspace_process(
        &self,
        name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Restarting process for workspace '{}'", name);

        // 1. Get the current process ID
        let old_process_id = {
            let mapping = self.workspace_process_mapping.read().await;
            mapping.get(name).cloned()
                .ok_or_else(|| format!("No process found for workspace '{}'", name))?
        };

        // 2. Stop the old process
        if let Err(e) = self.process_manager.kill_process(&old_process_id).await {
            warn!("Failed to stop old process '{}': {}", old_process_id, e);
        }

        // 3. Start a new process
        let new_process_id = format!("claude-code-{}-{}", name, chrono::Utc::now().timestamp());
        let command_args = vec![
            "--workspace".to_string(),
            name.to_string(),
        ];

        self.process_manager.spawn_process(
            new_process_id.clone(),
            name.to_string(),
            command_args,
        ).await
        .map_err(|e| format!("Failed to start new process: {}", e))?;

        // 4. Update the mapping
        let mut mapping = self.workspace_process_mapping.write().await;
        mapping.insert(name.to_string(), new_process_id.clone());

        info!("Successfully restarted process for workspace '{}' (new process: '{}')", name, new_process_id);
        Ok(())
    }

    /// Start automatic monitoring of all workspace processes
    pub async fn start_monitoring(&self) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error>> {
        if !self.monitoring_enabled {
            return Err("Monitoring is disabled".into());
        }

        // Create a reference to self for the monitoring task
        let health_check_interval = self.health_check_interval;

        let monitoring_task = tokio::spawn(async move {
            info!("Starting workspace process monitoring (interval: {:?})", health_check_interval);
            
            loop {
                sleep(health_check_interval).await;
                
                // Note: In a full implementation, we would check process health here
                // For now, this is a placeholder monitoring loop
                // The actual health checking would require shared state or channels
            }
        });

        Ok(monitoring_task)
    }

    /// Get health status of all workspace processes
    pub async fn get_workspace_health_status(&self) -> HashMap<String, ProcessStatus> {
        let mut health_status = HashMap::new();
        
        let mappings = {
            let mapping = self.workspace_process_mapping.read().await;
            mapping.clone()
        };

        for (workspace_name, process_id) in mappings {
            if let Some(process_info) = self.process_manager.get_process_info(&process_id).await {
                health_status.insert(workspace_name, process_info.status);
            } else {
                health_status.insert(workspace_name, ProcessStatus::Failed);
            }
        }

        health_status
    }

    /// Get list of all active workspaces with their process information
    pub async fn list_active_workspaces(&self) -> Vec<(String, Option<ProcessInfo>)> {
        let mut workspaces = Vec::new();
        
        let mappings = {
            let mapping = self.workspace_process_mapping.read().await;
            mapping.clone()
        };

        for (workspace_name, process_id) in mappings {
            let process_info = self.process_manager.get_process_info(&process_id).await;
            workspaces.push((workspace_name, process_info));
        }

        workspaces
    }

    /// Simulate process failure for testing
    pub async fn simulate_process_failure(
        &self,
        name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let process_id = {
            let mapping = self.workspace_process_mapping.read().await;
            mapping.get(name).cloned()
                .ok_or_else(|| format!("No process found for workspace '{}'", name))?
        };

        // Kill the process to simulate failure
        self.process_manager.kill_process(&process_id).await
            .map_err(|e| format!("Failed to simulate process failure: {}", e).into())
    }

    /// Stop monitoring for graceful shutdown
    pub async fn stop_monitoring(&self) {
        info!("Stopping workspace process monitoring");
        // In a full implementation, we would signal the monitoring task to stop
        // For now, this is a placeholder for graceful shutdown
    }

    /// Get workspace count for metrics
    pub async fn get_workspace_count(&self) -> usize {
        let mapping = self.workspace_process_mapping.read().await;
        mapping.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::manager::ProcessConfig;
    use std::collections::HashMap;
    use std::time::Duration;
    use tempfile::TempDir;

    async fn create_test_managers() -> (IntegratedWorkspaceManager, TempDir) {
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
            default_restart_policy: crate::process::manager::RestartPolicy::OnFailure,
            environment_vars: HashMap::new(),
            working_directory: None,
        };
        
        let (process_manager, _event_receiver) = ProcessManager::new(config);
        
        let integrated_manager = IntegratedWorkspaceManager::new(workspace_manager, process_manager)
            .with_monitoring(true, Duration::from_millis(100));
        
        (integrated_manager, temp_dir)
    }

    #[tokio::test]
    async fn test_monitoring_configuration() {
        let (manager, _temp_dir) = create_test_managers().await;
        
        assert!(manager.monitoring_enabled);
        assert_eq!(manager.health_check_interval, Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_workspace_health_status() {
        let (manager, _temp_dir) = create_test_managers().await;
        
        // Initially no workspaces
        let health_status = manager.get_workspace_health_status().await;
        assert!(health_status.is_empty());
        
        // Create a workspace with a process
        let result = manager.create_workspace_with_process("test-workspace", "basic").await;
        if let Err(e) = &result {
            eprintln!("Failed to create workspace: {}", e);
        }
        assert!(result.is_ok());
        
        // Check health status
        let health_status = manager.get_workspace_health_status().await;
        assert_eq!(health_status.len(), 1);
        assert!(health_status.contains_key("test-workspace"));
    }

    #[tokio::test]
    async fn test_list_active_workspaces() {
        let (manager, _temp_dir) = create_test_managers().await;
        
        // Initially no workspaces
        let workspaces = manager.list_active_workspaces().await;
        assert!(workspaces.is_empty());
        
        // Create a workspace
        let result = manager.create_workspace_with_process("test-workspace", "basic").await;
        assert!(result.is_ok());
        
        // List active workspaces
        let workspaces = manager.list_active_workspaces().await;
        assert_eq!(workspaces.len(), 1);
        assert_eq!(workspaces[0].0, "test-workspace");
    }

    #[tokio::test]
    async fn test_workspace_count() {
        let (manager, _temp_dir) = create_test_managers().await;
        
        // Initially no workspaces
        assert_eq!(manager.get_workspace_count().await, 0);
        
        // Create a workspace
        let result = manager.create_workspace_with_process("test-workspace", "basic").await;
        assert!(result.is_ok());
        
        assert_eq!(manager.get_workspace_count().await, 1);
    }

    #[tokio::test]
    async fn test_monitoring_start() {
        let (manager, _temp_dir) = create_test_managers().await;
        
        // Start monitoring should succeed
        let monitoring_task = manager.start_monitoring().await;
        assert!(monitoring_task.is_ok());
        
        // Abort the task to clean up
        monitoring_task.unwrap().abort();
    }

    #[tokio::test]
    async fn test_monitoring_disabled() {
        let (mut manager, _temp_dir) = create_test_managers().await;
        manager.monitoring_enabled = false;
        
        // Start monitoring should fail when disabled
        let monitoring_task = manager.start_monitoring().await;
        assert!(monitoring_task.is_err());
    }
}