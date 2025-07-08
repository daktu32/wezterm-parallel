// WezTerm Multi-Process Development Framework - Workspace Manager

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::error::{Result, UserError};
use crate::process::{
    ClaudeCodeConfig, ClaudeCodeConfigBuilder, ClaudeCodeDetector, ProcessManager,
};
use crate::room::state::{ProcessInfo, ProcessStatus, WorkspaceState};
use crate::room::template::{TemplateEngine, WorkspaceTemplate};

#[derive(Debug)]
pub struct WorkspaceManager {
    workspaces: RwLock<HashMap<String, WorkspaceState>>,
    template_engine: TemplateEngine,
    state_file_path: PathBuf,
    auto_save_enabled: bool,
    max_workspaces: usize,
    claude_code_detector: ClaudeCodeDetector,
    process_manager: Option<std::sync::Arc<ProcessManager>>,
    auto_start_claude_code: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct PersistedState {
    workspaces: HashMap<String, WorkspaceState>,
    last_saved: SystemTime,
    version: String,
}

impl WorkspaceManager {
    pub fn new(state_file_path: Option<PathBuf>) -> Result<Self> {
        let state_path = state_file_path.unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| {
                log::warn!("設定ディレクトリが取得できません。カレントディレクトリを使用します。");
                PathBuf::from(".")
            });
            path.push("wezterm-parallel");
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
            claude_code_detector: ClaudeCodeDetector::new(),
            process_manager: None,
            auto_start_claude_code: true,
        };

        // Load existing state if available
        if let Err(e) = manager.load_state() {
            warn!("Failed to load workspace state: {}, starting fresh", e);
        }

        // Ensure we have at least the default workspace
        let has_workspaces = {
            let workspaces = manager
                .workspaces
                .try_read()
                .map_err(|_| UserError::system_resource_exhausted("lockコンテンション"))?;
            !workspaces.is_empty()
        };

        if !has_workspaces {
            manager.create_default_workspace().map_err(|e| {
                UserError::config_load_failed("デフォルトワークスペース", &e.to_string())
            })?;
        }

        let workspace_count = manager
            .workspaces
            .try_read()
            .map(|w| w.len())
            .unwrap_or_else(|_| {
                log::warn!("ワークスペース数の取得でロック競合が発生しました");
                0
            });
        info!(
            "WorkspaceManager initialized with {} workspaces",
            workspace_count
        );

        Ok(manager)
    }

    pub async fn create_workspace(&self, name: &str, template_name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(UserError::room_creation_failed(name, "Room名が空です"));
        }

        // Check if workspace already exists
        {
            let workspaces = self.workspaces.read().await;
            if workspaces.contains_key(name) {
                return Err(UserError::room_creation_failed(
                    name,
                    "同名のRoomが既に存在します",
                ));
            }

            // Check workspace limit
            if workspaces.len() >= self.max_workspaces {
                return Err(UserError::room_creation_failed(
                    name,
                    &format!("Room数の上限（{}個）に達しています", self.max_workspaces),
                ));
            }
        }

        // Apply template to create config
        let config = self
            .template_engine
            .apply_template(template_name, name)
            .map_err(|e| {
                UserError::room_creation_failed(name, &format!("テンプレートの適用に失敗: {e}"))
            })?;

        // Create workspace state
        let workspace_state = WorkspaceState::new(name.to_string(), config);

        // Add to collection
        {
            let mut workspaces = self.workspaces.write().await;
            workspaces.insert(name.to_string(), workspace_state);
        }

        info!(
            "Room '{}' をテンプレート '{}' で作成しました",
            name, template_name
        );

        // Auto-start Claude Code if enabled
        if self.auto_start_claude_code {
            if let Err(e) = self.auto_start_claude_code_for_workspace(name).await {
                warn!("Room '{}' でのClaude Code自動起動に失敗: {}", name, e);
            }
        }

        // Auto-save if enabled
        if self.auto_save_enabled {
            if let Err(e) = self.save_state().await {
                warn!("Room状態の自動保存に失敗: {}", e);
            }
        }

        Ok(())
    }

    pub async fn delete_workspace(&self, name: &str) -> Result<()> {
        if name == "default" {
            return Err(UserError::room_creation_failed(
                name,
                "デフォルトRoomは削除できません",
            ));
        }

        let removed = {
            let mut workspaces = self.workspaces.write().await;
            workspaces.remove(name)
        };

        match removed {
            Some(workspace) => {
                info!(
                    "Deleted workspace '{}' (had {} processes)",
                    name,
                    workspace.processes.len()
                );

                // Auto-save if enabled
                if self.auto_save_enabled {
                    if let Err(e) = self.save_state().await {
                        warn!("Failed to auto-save after workspace deletion: {}", e);
                    }
                }

                Ok(())
            }
            None => Err(UserError::room_not_found(name)),
        }
    }

    pub async fn switch_workspace(&self, name: &str) -> Result<()> {
        let mut workspaces = self.workspaces.write().await;

        // Check if workspace exists first
        if !workspaces.contains_key(name) {
            return Err(UserError::room_not_found(name));
        }

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
            None => Err(UserError::room_not_found(name)),
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

    pub async fn update_workspace_state<F>(&self, name: &str, updater: F) -> Result<()>
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
            None => Err(UserError::room_not_found(name)),
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

    pub async fn save_state(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
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

    pub fn load_state(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if !self.state_file_path.exists() {
            info!("No existing workspace state file found");
            return Ok(());
        }

        let json = fs::read_to_string(&self.state_file_path)?;
        let persisted_state: PersistedState = serde_json::from_str(&json)?;

        // Check version compatibility
        let current_version = env!("CARGO_PKG_VERSION");
        if persisted_state.version != current_version {
            warn!(
                "State file version mismatch: {} vs {}, proceeding anyway",
                persisted_state.version, current_version
            );
        }

        // This is a blocking operation, but it's only called during initialization
        let mut workspaces = self
            .workspaces
            .try_write()
            .map_err(|_| "Failed to acquire write lock during initialization")?;

        *workspaces = persisted_state.workspaces;

        info!("Loaded {} workspaces from state file", workspaces.len());
        Ok(())
    }

    fn create_default_workspace(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let config = self
            .template_engine
            .apply_template("basic", "default")
            .map_err(|e| format!("Failed to create default workspace: {e}"))?;

        let workspace_state = WorkspaceState::new("default".to_string(), config);

        let mut workspaces = self
            .workspaces
            .try_write()
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
            name == "default"
                || workspace.is_active
                || workspace.last_accessed > cutoff_time
                || !workspace.processes.is_empty()
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
        workspaces.values().map(|w| w.processes.len()).sum()
    }

    // Claude Code自動起動機能

    /// プロセスマネージャーを設定
    pub fn set_process_manager(&mut self, process_manager: std::sync::Arc<ProcessManager>) {
        self.process_manager = Some(process_manager);
    }

    /// Claude Code自動起動を有効/無効にする
    pub fn set_auto_start_claude_code(&mut self, enabled: bool) {
        self.auto_start_claude_code = enabled;
    }

    /// 指定されたワークスペースでClaude Codeを自動起動
    async fn auto_start_claude_code_for_workspace(&self, workspace_name: &str) -> Result<()> {
        // Claude Codeバイナリを検出
        let binary_path = match self.claude_code_detector.detect() {
            Ok(path) => path,
            Err(e) => {
                return Err(UserError::claude_code_startup_failed(&format!(
                    "バイナリ検出に失敗: {e}"
                )));
            }
        };

        info!("Detected Claude Code binary at: {:?}", binary_path);

        // ワークスペース情報を取得
        let _workspace_info = self
            .get_workspace_info(workspace_name)
            .await
            .ok_or_else(|| UserError::room_not_found(workspace_name))?;

        // プロジェクトルートを取得（現在のディレクトリ、または指定されたディレクトリ）
        let project_root = std::env::current_dir().unwrap_or_else(|_| {
            log::warn!("現在のディレクトリが取得できません。カレントディレクトリを使用します。");
            PathBuf::from(".")
        });

        // Claude Code設定を構築
        let claude_config = match ClaudeCodeConfigBuilder::new(binary_path, workspace_name)
            .project_root(project_root)
            .environment("WEZTERM_WORKSPACE", workspace_name)
            .argument("--workspace")
            .argument(workspace_name)
            .memory_limit(4096) // 4GB
            .cpu_limit(75.0) // 75%
            .build()
        {
            Ok(config) => config,
            Err(e) => {
                return Err(UserError::claude_code_startup_failed(&format!(
                    "設定構築に失敗: {e}"
                )));
            }
        };

        info!(
            "Built Claude Code config: {}",
            claude_config.to_command_string()
        );

        // プロセスマネージャーが設定されている場合のみ起動
        if let Some(ref process_manager) = self.process_manager {
            match self
                .spawn_claude_code_process(process_manager, workspace_name, claude_config)
                .await
            {
                Ok(process_id) => {
                    info!(
                        "Successfully started Claude Code process '{}' for workspace '{}'",
                        process_id, workspace_name
                    );

                    // ワークスペース状態を更新してプロセス情報を追加
                    self.update_workspace_state(workspace_name, |workspace| {
                        workspace.processes.insert(
                            process_id.clone(),
                            ProcessInfo {
                                id: process_id.clone(),
                                command: format!("claude-code --workspace {workspace_name}"),
                                workspace: workspace_name.to_string(),
                                pane_id: None,
                                status: ProcessStatus::Starting,
                                pid: None, // プロセス起動後に更新される
                                started_at: SystemTime::now(),
                                last_heartbeat: SystemTime::now(),
                                restart_count: 0,
                            },
                        );
                    })
                    .await?;

                    Ok(())
                }
                Err(e) => Err(UserError::claude_code_startup_failed(&format!(
                    "プロセス起動に失敗: {e}"
                ))),
            }
        } else {
            warn!(
                "ProcessManager not set, cannot start Claude Code for workspace '{}'",
                workspace_name
            );
            Ok(())
        }
    }

    /// Claude Codeプロセスを起動
    async fn spawn_claude_code_process(
        &self,
        process_manager: &ProcessManager,
        workspace_name: &str,
        claude_config: ClaudeCodeConfig,
    ) -> std::result::Result<String, String> {
        // プロセスIDを生成
        let process_id = format!(
            "claude-{}-{}",
            workspace_name,
            uuid::Uuid::new_v4().simple()
        );

        // コマンド引数を構築（バイナリパス + 引数）
        let mut command_args = vec![claude_config.binary_path.to_string_lossy().to_string()];
        command_args.extend(claude_config.get_complete_arguments());

        // プロセスを起動
        match process_manager
            .spawn_process(process_id.clone(), workspace_name.to_string(), command_args)
            .await
        {
            Ok(_) => {
                info!("Successfully spawned Claude Code process '{}'", process_id);
                Ok(process_id)
            }
            Err(e) => Err(format!("Failed to spawn process: {e}")),
        }
    }

    /// ワークスペース用のClaude Codeプロセスを手動で起動
    pub async fn start_claude_code_for_workspace(&self, workspace_name: &str) -> Result<String> {
        self.auto_start_claude_code_for_workspace(workspace_name)
            .await?;
        // プロセスIDを返す（実際の実装では起動したプロセスIDを返す）
        Ok(format!("claude-{workspace_name}-manual"))
    }

    /// ワークスペース用のClaude Codeプロセスを停止
    pub async fn stop_claude_code_for_workspace(&self, workspace_name: &str) -> Result<()> {
        if let Some(ref process_manager) = self.process_manager {
            let workspace_info = self
                .get_workspace_info(workspace_name)
                .await
                .ok_or_else(|| UserError::room_not_found(workspace_name))?;

            // ワークスペースに関連するプロセスを停止
            for process_id in workspace_info.processes.keys() {
                if let Err(e) = process_manager.kill_process(process_id).await {
                    warn!("Failed to stop process '{}': {}", process_id, e);
                }
            }

            // ワークスペース状態からプロセス情報を削除
            self.update_workspace_state(workspace_name, |workspace| {
                workspace.processes.clear();
            })
            .await?;

            info!(
                "Stopped all Claude Code processes for workspace '{}'",
                workspace_name
            );
            Ok(())
        } else {
            Err(UserError::process_communication_failed("ProcessManager"))
        }
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
        assert!(result
            .unwrap_err()
            .message_jp
            .contains("同名のRoomが既に存在します"));
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
        assert!(result
            .unwrap_err()
            .message_jp
            .contains("デフォルトRoomは削除できません"));
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
        assert!(result.unwrap_err().message_jp.contains("Room数の上限"));
    }
}
