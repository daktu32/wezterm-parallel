// WezTerm Multi-Process Development Framework - Workspace Manager

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::time::SystemTime;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::workspace::state::WorkspaceState;
use crate::workspace::template::{TemplateEngine, WorkspaceTemplate};

#[derive(Debug)]
pub struct WorkspaceManager {
    workspaces: RwLock<HashMap<String, WorkspaceState>>,
    template_engine: TemplateEngine,
    state_file_path: PathBuf,
    auto_save_enabled: bool,
    max_workspaces: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct PersistedState {
    workspaces: HashMap<String, WorkspaceState>,
    last_saved: SystemTime,
    version: String,
}

impl WorkspaceManager {
    pub fn new(state_file_path: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let state_path = state_file_path.unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("wezterm-multi-dev");
            path.push("workspaces.json");
            path
        });

        // Ensure the parent directory exists
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let template_engine = TemplateEngine::new();
        
        let manager = Self {
            workspaces: RwLock::new(HashMap::new()),
            template_engine,
            state_file_path: state_path,
            auto_save_enabled: true,
            max_workspaces: 8,
        };

        // Load existing state if available
        if let Err(e) = manager.load_state() {
            warn!("Failed to load workspace state: {}, starting fresh", e);
        }
        
        // Ensure we have at least the default workspace
        let has_workspaces = {
            let workspaces = manager.workspaces.try_read()
                .map_err(|_| "Failed to read workspaces")?;
            !workspaces.is_empty()
        };
        
        if !has_workspaces {
            manager.create_default_workspace()?;
        }

        info!("WorkspaceManager initialized with {} workspaces", 
              manager.workspaces.try_read().map(|w| w.len()).unwrap_or(0));

        Ok(manager)
    }

    pub async fn create_workspace(&self, name: &str, template_name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Workspace name cannot be empty".to_string());
        }

        // Check if workspace already exists
        {
            let workspaces = self.workspaces.read().await;
            if workspaces.contains_key(name) {
                return Err(format!("Workspace '{}' already exists", name));
            }

            // Check workspace limit
            if workspaces.len() >= self.max_workspaces {
                return Err(format!("Maximum number of workspaces ({}) reached", self.max_workspaces));
            }
        }

        // Apply template to create config
        let config = self.template_engine.apply_template(template_name, name)?;
        
        // Create workspace state
        let workspace_state = WorkspaceState::new(name.to_string(), config);

        // Add to collection
        {
            let mut workspaces = self.workspaces.write().await;
            workspaces.insert(name.to_string(), workspace_state);
        }

        info!("Created workspace '{}' with template '{}'", name, template_name);

        // Auto-save if enabled
        if self.auto_save_enabled {
            if let Err(e) = self.save_state().await {
                warn!("Failed to auto-save workspace state: {}", e);
            }
        }

        Ok(())
    }

    pub async fn delete_workspace(&self, name: &str) -> Result<(), String> {
        if name == "default" {
            return Err("Cannot delete the default workspace".to_string());
        }

        let removed = {
            let mut workspaces = self.workspaces.write().await;
            workspaces.remove(name)
        };

        match removed {
            Some(workspace) => {
                info!("Deleted workspace '{}' (had {} processes)", name, workspace.processes.len());
                
                // Auto-save if enabled
                if self.auto_save_enabled {
                    if let Err(e) = self.save_state().await {
                        warn!("Failed to auto-save after workspace deletion: {}", e);
                    }
                }
                
                Ok(())
            }
            None => Err(format!("Workspace '{}' not found", name))
        }
    }

    pub async fn switch_workspace(&self, name: &str) -> Result<(), String> {
        let mut workspaces = self.workspaces.write().await;
        
        // Deactivate all workspaces
        for workspace in workspaces.values_mut() {
            workspace.deactivate();
        }

        // Activate target workspace
        match workspaces.get_mut(name) {
            Some(workspace) => {
                workspace.activate();
                info!("Switched to workspace '{}'", name);
                Ok(())
            }
            None => Err(format!("Workspace '{}' not found", name))
        }
    }

    pub async fn list_workspaces(&self) -> Vec<String> {
        let workspaces = self.workspaces.read().await;
        workspaces.keys().cloned().collect()
    }

    pub async fn get_workspace_info(&self, name: &str) -> Option<WorkspaceState> {
        let workspaces = self.workspaces.read().await;
        workspaces.get(name).cloned()
    }

    pub async fn get_active_workspace(&self) -> Option<(String, WorkspaceState)> {
        let workspaces = self.workspaces.read().await;
        workspaces
            .iter()
            .find(|(_, workspace)| workspace.is_active)
            .map(|(name, workspace)| (name.clone(), workspace.clone()))
    }

    pub async fn update_workspace_state<F>(&self, name: &str, updater: F) -> Result<(), String>
    where
        F: FnOnce(&mut WorkspaceState),
    {
        let mut workspaces = self.workspaces.write().await;
        
        match workspaces.get_mut(name) {
            Some(workspace) => {
                updater(workspace);
                
                // Auto-save if enabled
                drop(workspaces); // Release lock before async operation
                if self.auto_save_enabled {
                    if let Err(e) = self.save_state().await {
                        warn!("Failed to auto-save workspace state: {}", e);
                    }
                }
                
                Ok(())
            }
            None => Err(format!("Workspace '{}' not found", name))
        }
    }

    pub fn register_template(&mut self, template: WorkspaceTemplate) {
        self.template_engine.register_template(template);
    }

    pub fn list_templates(&self) -> Vec<&WorkspaceTemplate> {
        self.template_engine.list_templates()
    }

    pub fn get_template(&self, name: &str) -> Option<&WorkspaceTemplate> {
        self.template_engine.get_template(name)
    }

    pub async fn save_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        let workspaces = self.workspaces.read().await;
        
        let state = PersistedState {
            workspaces: workspaces.clone(),
            last_saved: SystemTime::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let json = serde_json::to_string_pretty(&state)?;
        
        // Write to temporary file first, then rename for atomic operation
        let temp_path = self.state_file_path.with_extension("tmp");
        fs::write(&temp_path, json)?;
        fs::rename(&temp_path, &self.state_file_path)?;

        info!("Saved workspace state to {:?}", self.state_file_path);
        Ok(())
    }

    pub fn load_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.state_file_path.exists() {
            info!("No existing workspace state file found");
            return Ok(());
        }

        let json = fs::read_to_string(&self.state_file_path)?;
        let persisted_state: PersistedState = serde_json::from_str(&json)?;

        // Check version compatibility
        let current_version = env!("CARGO_PKG_VERSION");
        if persisted_state.version != current_version {
            warn!("State file version mismatch: {} vs {}, proceeding anyway", 
                  persisted_state.version, current_version);
        }

        // This is a blocking operation, but it's only called during initialization
        let mut workspaces = self.workspaces.try_write()
            .map_err(|_| "Failed to acquire write lock during initialization")?;
        
        *workspaces = persisted_state.workspaces;

        info!("Loaded {} workspaces from state file", workspaces.len());
        Ok(())
    }

    fn create_default_workspace(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.template_engine
            .apply_template("basic", "default")
            .map_err(|e| format!("Failed to create default workspace: {}", e))?;

        let workspace_state = WorkspaceState::new("default".to_string(), config);

        let mut workspaces = self.workspaces.try_write()
            .map_err(|_| "Failed to acquire write lock")?;
        
        workspaces.insert("default".to_string(), workspace_state);

        info!("Created default workspace");
        Ok(())
    }

    pub async fn cleanup_inactive_workspaces(&self, max_age_hours: u64) -> usize {
        let cutoff_time = SystemTime::now()
            .checked_sub(std::time::Duration::from_secs(max_age_hours * 3600))
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let mut workspaces = self.workspaces.write().await;
        let initial_count = workspaces.len();

        // Keep default workspace and active workspaces
        workspaces.retain(|name, workspace| {
            name == "default" || 
            workspace.is_active || 
            workspace.last_accessed > cutoff_time ||
            !workspace.processes.is_empty()
        });

        let cleaned_count = initial_count - workspaces.len();
        
        if cleaned_count > 0 {
            info!("Cleaned up {} inactive workspaces", cleaned_count);
            
            // Auto-save if enabled
            drop(workspaces);
            if self.auto_save_enabled {
                if let Err(e) = self.save_state().await {
                    warn!("Failed to auto-save after cleanup: {}", e);
                }
            }
        }

        cleaned_count
    }

    pub fn set_auto_save(&mut self, enabled: bool) {
        self.auto_save_enabled = enabled;
    }

    pub fn set_max_workspaces(&mut self, max: usize) {
        self.max_workspaces = max;
    }

    pub async fn get_workspace_count(&self) -> usize {
        let workspaces = self.workspaces.read().await;
        workspaces.len()
    }

    pub async fn get_total_process_count(&self) -> usize {
        let workspaces = self.workspaces.read().await;
        workspaces.values()
            .map(|w| w.processes.len())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_manager() -> WorkspaceManager {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("test_workspaces.json");
        WorkspaceManager::new(Some(state_path)).unwrap()
    }

    #[tokio::test]
    async fn test_workspace_manager_creation() {
        let manager = create_test_manager().await;
        
        // Should have default workspace
        let workspaces = manager.list_workspaces().await;
        assert!(!workspaces.is_empty());
        assert!(workspaces.contains(&"default".to_string()));
    }

    #[tokio::test]
    async fn test_create_workspace() {
        let manager = create_test_manager().await;
        
        let result = manager.create_workspace("test", "basic").await;
        assert!(result.is_ok());
        
        let workspaces = manager.list_workspaces().await;
        assert!(workspaces.contains(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_create_duplicate_workspace() {
        let manager = create_test_manager().await;
        
        manager.create_workspace("test", "basic").await.unwrap();
        let result = manager.create_workspace("test", "basic").await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[tokio::test]
    async fn test_switch_workspace() {
        let manager = create_test_manager().await;
        
        manager.create_workspace("test", "basic").await.unwrap();
        
        let result = manager.switch_workspace("test").await;
        assert!(result.is_ok());
        
        let active = manager.get_active_workspace().await;
        assert!(active.is_some());
        assert_eq!(active.unwrap().0, "test");
    }

    #[tokio::test]
    async fn test_delete_workspace() {
        let manager = create_test_manager().await;
        
        manager.create_workspace("test", "basic").await.unwrap();
        
        let result = manager.delete_workspace("test").await;
        assert!(result.is_ok());
        
        let workspaces = manager.list_workspaces().await;
        assert!(!workspaces.contains(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_cannot_delete_default_workspace() {
        let manager = create_test_manager().await;
        
        let result = manager.delete_workspace("default").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot delete the default workspace"));
    }

    #[tokio::test]
    async fn test_workspace_state_persistence() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("test_workspaces.json");
        
        // Create manager and workspace
        {
            let manager = WorkspaceManager::new(Some(state_path.clone())).unwrap();
            manager.create_workspace("test", "basic").await.unwrap();
            manager.save_state().await.unwrap();
        }
        
        // Create new manager with same state file
        {
            let manager = WorkspaceManager::new(Some(state_path)).unwrap();
            let workspaces = manager.list_workspaces().await;
            assert!(workspaces.contains(&"test".to_string()));
        }
    }

    #[tokio::test]
    async fn test_workspace_limit() {
        let mut manager = create_test_manager().await;
        manager.set_max_workspaces(2); // Default + 1 more
        
        // First workspace should succeed
        let result = manager.create_workspace("test1", "basic").await;
        assert!(result.is_ok());
        
        // Second workspace should fail (already have default + test1 = 2)
        let result = manager.create_workspace("test2", "basic").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Maximum number of workspaces"));
    }
}