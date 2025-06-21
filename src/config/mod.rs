// WezTerm Multi-Process Development Framework - Configuration Management
// Handles YAML configuration loading, validation, and hot reloading

pub mod loader;
pub mod validator;
pub mod hot_reload;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    
    /// Workspace configuration
    pub workspace: WorkspaceConfig,
    
    /// Process configuration
    pub process: ProcessConfig,
    
    /// UI configuration
    pub ui: UiConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// Plugin configuration
    pub plugins: HashMap<String, PluginConfig>,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Unix socket path
    pub socket_path: String,
    
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    
    /// Enable metrics collection
    pub enable_metrics: bool,
    
    /// Health check interval in seconds
    pub health_check_interval: u64,
}

/// Workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Maximum number of workspaces
    pub max_workspaces: usize,
    
    /// Default workspace template
    pub default_template: String,
    
    /// Workspace state persistence path
    pub state_path: PathBuf,
    
    /// Auto-save interval in seconds
    pub auto_save_interval: u64,
    
    /// Workspace templates directory
    pub templates_dir: PathBuf,
}

/// Process configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    /// Maximum processes per workspace
    pub max_processes_per_workspace: usize,
    
    /// Process startup timeout in seconds
    pub startup_timeout: u64,
    
    /// Process health check interval in seconds
    pub health_check_interval: u64,
    
    /// Enable auto-restart on failure
    pub auto_restart: bool,
    
    /// Maximum restart attempts
    pub max_restart_attempts: u32,
    
    /// Process environment variables
    pub environment: HashMap<String, String>,
    
    /// Working directory template
    pub working_dir_template: String,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Dashboard configuration
    pub dashboard: DashboardConfig,
    
    /// Theme configuration
    pub theme: ThemeConfig,
    
    /// Keybinding configuration
    pub keybindings: KeybindingConfig,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Update interval in seconds
    pub update_interval: f64,
    
    /// Dashboard width percentage
    pub width_percentage: u8,
    
    /// Dashboard position (left, right, top, bottom)
    pub position: String,
    
    /// Enable real-time updates
    pub real_time_updates: bool,
    
    /// Maximum log entries to display
    pub max_log_entries: usize,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Background color
    pub background: String,
    
    /// Foreground color
    pub foreground: String,
    
    /// Border color
    pub border: String,
    
    /// Header color
    pub header: String,
    
    /// Success color
    pub success: String,
    
    /// Warning color
    pub warning: String,
    
    /// Error color
    pub error: String,
    
    /// Info color
    pub info: String,
}

/// Keybinding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConfig {
    /// Leader key combination
    pub leader_key: String,
    
    /// Workspace prefix
    pub workspace_prefix: String,
    
    /// Process prefix  
    pub process_prefix: String,
    
    /// Pane prefix
    pub pane_prefix: String,
    
    /// Dashboard prefix
    pub dashboard_prefix: String,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (error, warn, info, debug, trace)
    pub level: String,
    
    /// Log file path
    pub file_path: Option<PathBuf>,
    
    /// Enable console logging
    pub console: bool,
    
    /// Maximum log file size in MB
    pub max_file_size: u64,
    
    /// Number of log files to keep
    pub max_files: u32,
    
    /// Log format (json, plain)
    pub format: String,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin enabled status
    pub enabled: bool,
    
    /// Plugin configuration parameters
    pub config: HashMap<String, serde_yaml::Value>,
    
    /// Plugin priority (lower = higher priority)
    pub priority: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            workspace: WorkspaceConfig::default(),
            process: ProcessConfig::default(),
            ui: UiConfig::default(),
            logging: LoggingConfig::default(),
            plugins: HashMap::new(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            socket_path: "/tmp/wezterm-multi-dev.sock".to_string(),
            max_connections: 100,
            connection_timeout: 30,
            enable_metrics: true,
            health_check_interval: 10,
        }
    }
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            max_workspaces: 8,
            default_template: "basic".to_string(),
            state_path: PathBuf::from("~/.config/wezterm-multi-dev/workspaces.json"),
            auto_save_interval: 30,
            templates_dir: PathBuf::from("~/.config/wezterm-multi-dev/templates"),
        }
    }
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            max_processes_per_workspace: 16,
            startup_timeout: 60,
            health_check_interval: 5,
            auto_restart: true,
            max_restart_attempts: 3,
            environment: HashMap::new(),
            working_dir_template: "~/projects/{{workspace_name}}".to_string(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            dashboard: DashboardConfig::default(),
            theme: ThemeConfig::default(),
            keybindings: KeybindingConfig::default(),
        }
    }
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            update_interval: 2.0,
            width_percentage: 30,
            position: "right".to_string(),
            real_time_updates: true,
            max_log_entries: 100,
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background: "#1e1e2e".to_string(),
            foreground: "#cdd6f4".to_string(),
            border: "#45475a".to_string(),
            header: "#89b4fa".to_string(),
            success: "#a6e3a1".to_string(),
            warning: "#f9e2af".to_string(),
            error: "#f38ba8".to_string(),
            info: "#89dceb".to_string(),
        }
    }
}

impl Default for KeybindingConfig {
    fn default() -> Self {
        Self {
            leader_key: "CTRL|SHIFT+Space".to_string(),
            workspace_prefix: "CTRL|SHIFT".to_string(),
            process_prefix: "CTRL|ALT".to_string(),
            pane_prefix: "ALT".to_string(),
            dashboard_prefix: "CTRL|SHIFT".to_string(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_path: Some(PathBuf::from("~/.config/wezterm-multi-dev/logs/framework.log")),
            console: true,
            max_file_size: 10,
            max_files: 5,
            format: "plain".to_string(),
        }
    }
}