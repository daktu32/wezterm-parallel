// WezTerm Multi-Process Development Framework - Workspace State Management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessRecord {
    pub timestamp: SystemTime,
    pub duration: u64, // in seconds
    pub action: AccessAction,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum AccessAction {
    Create,
    Switch,
    Delete,
    Restore,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomUsageStats {
    pub total_sessions: u32,
    pub total_duration: u64,
    pub last_accessed: SystemTime,
    pub created_at: SystemTime,
    pub average_session_duration: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkspaceState {
    pub name: String,
    pub template: String,
    pub layout: LayoutConfig,
    pub panes: Vec<PaneState>,
    pub processes: HashMap<String, ProcessInfo>,
    pub active_tasks: Vec<TaskState>,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub is_active: bool,
    pub access_history: Vec<AccessRecord>,
    pub session_count: u32,
    pub total_duration: u64, // in seconds
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LayoutConfig {
    pub layout_type: LayoutType,
    pub primary_direction: SplitDirection,
    pub pane_sizes: Vec<f32>, // Percentage values
    pub auto_balance: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LayoutType {
    Single,
    TwoPaneHorizontal,
    TwoPaneVertical,
    ThreePaneHorizontal,
    ThreePaneVertical,
    FourPaneGrid,
    Custom(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaneState {
    pub id: String,
    pub position: PanePosition,
    pub size: f32, // Percentage of container
    pub command: Option<String>,
    pub working_directory: String,
    pub is_active: bool,
    pub process_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PanePosition {
    pub row: u32,
    pub col: u32,
    pub span_rows: u32,
    pub span_cols: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessInfo {
    pub id: String,
    pub command: String,
    pub workspace: String,
    pub pane_id: Option<String>,
    pub status: ProcessStatus,
    pub pid: Option<u32>,
    pub started_at: SystemTime,
    pub last_heartbeat: SystemTime,
    pub restart_count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Starting,
    Running,
    Idle,
    Busy,
    Stopping,
    Stopped,
    Failed,
    Restarting,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskState {
    pub id: String,
    pub workspace: String,
    pub command: String,
    pub priority: u8,
    pub status: TaskStatus,
    pub dependencies: Vec<String>,
    pub assigned_process: Option<String>,
    pub created_at: SystemTime,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub result: Option<TaskResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    Blocked, // Waiting for dependencies
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkspaceConfig {
    pub name: String,
    pub template: String,
    pub auto_start_processes: bool,
    pub max_processes: u32,
    pub working_directory: String,
    pub environment_vars: HashMap<String, String>,
    pub startup_commands: Vec<String>,
    pub keybindings: HashMap<String, String>,
    pub theme: Option<String>,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            layout_type: LayoutType::ThreePaneHorizontal,
            primary_direction: SplitDirection::Horizontal,
            pane_sizes: vec![33.3, 33.3, 33.4],
            auto_balance: true,
        }
    }
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            template: "basic".to_string(),
            auto_start_processes: true,
            max_processes: 4,
            working_directory: std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("/"))
                .to_string_lossy()
                .to_string(),
            environment_vars: HashMap::new(),
            startup_commands: vec!["claude-code".to_string()],
            keybindings: HashMap::new(),
            theme: None,
        }
    }
}

impl WorkspaceState {
    pub fn new(name: String, config: WorkspaceConfig) -> Self {
        let now = SystemTime::now();

        Self {
            name: name.clone(),
            template: config.template.clone(),
            layout: LayoutConfig::default(),
            panes: Vec::new(),
            processes: HashMap::new(),
            active_tasks: Vec::new(),
            created_at: now,
            last_accessed: now,
            is_active: false,
            access_history: vec![AccessRecord {
                timestamp: now,
                duration: 0,
                action: AccessAction::Create,
            }],
            session_count: 1,
            total_duration: 0,
        }
    }

    pub fn record_access(&mut self, action: AccessAction, duration: u64) {
        self.access_history.push(AccessRecord {
            timestamp: SystemTime::now(),
            duration,
            action,
        });
        self.last_accessed = SystemTime::now();
        if matches!(action, AccessAction::Switch) {
            self.session_count += 1;
        }
        self.total_duration += duration;

        // 履歴が長くなりすぎないよう制限
        if self.access_history.len() > 100 {
            self.access_history.drain(0..50);
        }
    }

    pub fn get_usage_stats(&self) -> RoomUsageStats {
        RoomUsageStats {
            total_sessions: self.session_count,
            total_duration: self.total_duration,
            last_accessed: self.last_accessed,
            created_at: self.created_at,
            average_session_duration: if self.session_count > 0 {
                self.total_duration / self.session_count as u64
            } else {
                0
            },
        }
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.last_accessed = SystemTime::now();
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn add_process(&mut self, process: ProcessInfo) {
        self.processes.insert(process.id.clone(), process);
    }

    pub fn remove_process(&mut self, process_id: &str) -> Option<ProcessInfo> {
        self.processes.remove(process_id)
    }

    pub fn get_running_processes(&self) -> Vec<&ProcessInfo> {
        self.processes
            .values()
            .filter(|p| matches!(p.status, ProcessStatus::Running | ProcessStatus::Busy))
            .collect()
    }

    pub fn add_task(&mut self, task: TaskState) {
        self.active_tasks.push(task);
    }

    pub fn remove_completed_tasks(&mut self) {
        self.active_tasks.retain(|task| {
            !matches!(
                task.status,
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
            )
        });
    }

    pub fn get_pending_tasks(&self) -> Vec<&TaskState> {
        self.active_tasks
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Queued | TaskStatus::Blocked))
            .collect()
    }

    pub fn add_pane(&mut self, pane: PaneState) {
        self.panes.push(pane);
    }

    pub fn remove_pane(&mut self, pane_id: &str) -> Option<PaneState> {
        if let Some(pos) = self.panes.iter().position(|p| p.id == pane_id) {
            Some(self.panes.remove(pos))
        } else {
            None
        }
    }

    pub fn get_active_pane(&self) -> Option<&PaneState> {
        self.panes.iter().find(|p| p.is_active)
    }

    pub fn set_active_pane(&mut self, pane_id: &str) {
        for pane in &mut self.panes {
            pane.is_active = pane.id == pane_id;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_state_creation() {
        let config = WorkspaceConfig::default();
        let workspace = WorkspaceState::new("test".to_string(), config);

        assert_eq!(workspace.name, "test");
        assert!(!workspace.is_active);
        assert!(workspace.processes.is_empty());
        assert!(workspace.active_tasks.is_empty());
        assert!(workspace.panes.is_empty());
    }

    #[test]
    fn test_workspace_activation() {
        let config = WorkspaceConfig::default();
        let mut workspace = WorkspaceState::new("test".to_string(), config);

        assert!(!workspace.is_active);

        workspace.activate();
        assert!(workspace.is_active);

        workspace.deactivate();
        assert!(!workspace.is_active);
    }

    #[test]
    fn test_process_management() {
        let config = WorkspaceConfig::default();
        let mut workspace = WorkspaceState::new("test".to_string(), config);

        let process = ProcessInfo {
            id: "proc-1".to_string(),
            command: "claude-code".to_string(),
            workspace: "test".to_string(),
            pane_id: None,
            status: ProcessStatus::Running,
            pid: Some(1234),
            started_at: SystemTime::now(),
            last_heartbeat: SystemTime::now(),
            restart_count: 0,
        };

        workspace.add_process(process.clone());
        assert_eq!(workspace.processes.len(), 1);

        let running = workspace.get_running_processes();
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].id, "proc-1");

        let removed = workspace.remove_process("proc-1");
        assert!(removed.is_some());
        assert_eq!(workspace.processes.len(), 0);
    }

    #[test]
    fn test_pane_management() {
        let config = WorkspaceConfig::default();
        let mut workspace = WorkspaceState::new("test".to_string(), config);

        let pane = PaneState {
            id: "pane-1".to_string(),
            position: PanePosition {
                row: 0,
                col: 0,
                span_rows: 1,
                span_cols: 1,
            },
            size: 50.0,
            command: Some("claude-code".to_string()),
            working_directory: "/tmp".to_string(),
            is_active: true,
            process_id: None,
        };

        workspace.add_pane(pane);
        assert_eq!(workspace.panes.len(), 1);

        let active = workspace.get_active_pane();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, "pane-1");

        let removed = workspace.remove_pane("pane-1");
        assert!(removed.is_some());
        assert_eq!(workspace.panes.len(), 0);
    }
}
