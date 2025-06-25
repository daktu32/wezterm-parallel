// Configuration loading and parsing functionality

use super::Config;
use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use log::{info, warn, debug};

/// Configuration loader with support for multiple sources
pub struct ConfigLoader {
    /// Search paths for configuration files
    search_paths: Vec<PathBuf>,
    
    /// Override values from environment variables
    env_overrides: HashMap<String, String>,
    
    /// Override values from command line
    cli_overrides: HashMap<String, String>,
}

/// Configuration loading error
#[derive(Debug)]
pub enum ConfigError {
    /// File not found
    FileNotFound(PathBuf),
    
    /// IO error
    Io(std::io::Error),
    
    /// YAML parsing error
    Yaml(serde_yaml::Error),
    
    /// Validation error
    Validation(String),
    
    /// Environment variable error
    Environment(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileNotFound(path) => write!(f, "Configuration file not found: {}", path.display()),
            ConfigError::Io(err) => write!(f, "IO error: {}", err),
            ConfigError::Yaml(err) => write!(f, "YAML parsing error: {}", err),
            ConfigError::Validation(msg) => write!(f, "Validation error: {}", msg),
            ConfigError::Environment(msg) => write!(f, "Environment error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Io(err)
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> Self {
        ConfigError::Yaml(err)
    }
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Self {
        let mut search_paths = Vec::new();
        
        // Add default search paths
        if let Ok(home) = std::env::var("HOME") {
            search_paths.push(PathBuf::from(home.clone()).join(".config/wezterm-parallel/config.yaml"));
            search_paths.push(PathBuf::from(home).join(".wezterm-parallel.yaml"));
        }
        
        // Add current directory
        search_paths.push(PathBuf::from("./wezterm-parallel.yaml"));
        search_paths.push(PathBuf::from("./config.yaml"));
        
        // Add system-wide config
        search_paths.push(PathBuf::from("/etc/wezterm-parallel/config.yaml"));
        
        Self {
            search_paths,
            env_overrides: Self::load_env_overrides(),
            cli_overrides: HashMap::new(),
        }
    }
    
    /// Create a new configuration loader with custom search paths
    pub fn with_search_paths(paths: Vec<PathBuf>) -> Self {
        Self {
            search_paths: paths,
            env_overrides: Self::load_env_overrides(),
            cli_overrides: HashMap::new(),
        }
    }
    
    /// Add a search path
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }
    
    /// Set CLI overrides
    pub fn set_cli_overrides(&mut self, overrides: HashMap<String, String>) {
        self.cli_overrides = overrides;
    }
    
    /// Load configuration synchronously
    pub fn load(&self) -> Result<Config, ConfigError> {
        info!("Loading configuration from search paths: {:?}", self.search_paths);
        
        // Try to find and load config file
        let mut config = match self.find_and_load_config() {
            Ok(config) => config,
            Err(ConfigError::FileNotFound(_)) => {
                warn!("No configuration file found, using defaults");
                Config::default()
            }
            Err(err) => return Err(err),
        };
        
        // Apply environment overrides
        self.apply_env_overrides(&mut config)?;
        
        // Apply CLI overrides
        self.apply_cli_overrides(&mut config)?;
        
        // Validate configuration
        self.validate_config(&config)?;
        
        info!("Configuration loaded successfully");
        debug!("Final configuration: {:?}", config);
        
        Ok(config)
    }
    
    /// Load configuration asynchronously
    pub async fn load_async(&self) -> Result<Config, ConfigError> {
        info!("Loading configuration asynchronously from search paths: {:?}", self.search_paths);
        
        // Try to find and load config file
        let mut config = match self.find_and_load_config_async().await {
            Ok(config) => config,
            Err(ConfigError::FileNotFound(_)) => {
                warn!("No configuration file found, using defaults");
                Config::default()
            }
            Err(err) => return Err(err),
        };
        
        // Apply environment overrides
        self.apply_env_overrides(&mut config)?;
        
        // Apply CLI overrides
        self.apply_cli_overrides(&mut config)?;
        
        // Validate configuration
        self.validate_config(&config)?;
        
        info!("Configuration loaded successfully");
        debug!("Final configuration: {:?}", config);
        
        Ok(config)
    }
    
    /// Find and load configuration file
    fn find_and_load_config(&self) -> Result<Config, ConfigError> {
        for path in &self.search_paths {
            if path.exists() {
                info!("Found configuration file: {}", path.display());
                let content = fs::read_to_string(path)?;
                let config: Config = serde_yaml::from_str(&content)?;
                return Ok(config);
            }
        }
        
        Err(ConfigError::FileNotFound(
            self.search_paths.first().cloned().unwrap_or_default()
        ))
    }
    
    /// Find and load configuration file asynchronously
    async fn find_and_load_config_async(&self) -> Result<Config, ConfigError> {
        for path in &self.search_paths {
            if path.exists() {
                info!("Found configuration file: {}", path.display());
                let content = async_fs::read_to_string(path).await?;
                let config: Config = serde_yaml::from_str(&content)?;
                return Ok(config);
            }
        }
        
        Err(ConfigError::FileNotFound(
            self.search_paths.first().cloned().unwrap_or_default()
        ))
    }
    
    /// Load environment variable overrides
    fn load_env_overrides() -> HashMap<String, String> {
        let mut overrides = HashMap::new();
        
        // Define environment variable mappings
        let env_mappings = [
            ("WEZTERM_MULTI_DEV_SOCKET_PATH", "server.socket_path"),
            ("WEZTERM_MULTI_DEV_MAX_WORKSPACES", "workspace.max_workspaces"),
            ("WEZTERM_MULTI_DEV_LOG_LEVEL", "logging.level"),
            ("WEZTERM_MULTI_DEV_MAX_PROCESSES", "process.max_processes_per_workspace"),
        ];
        
        for (env_var, config_path) in &env_mappings {
            if let Ok(value) = std::env::var(env_var) {
                overrides.insert(config_path.to_string(), value);
            }
        }
        
        overrides
    }
    
    /// Apply environment variable overrides
    fn apply_env_overrides(&self, config: &mut Config) -> Result<(), ConfigError> {
        for (path, value) in &self.env_overrides {
            self.apply_override(config, path, value)?;
        }
        Ok(())
    }
    
    /// Apply CLI overrides
    fn apply_cli_overrides(&self, config: &mut Config) -> Result<(), ConfigError> {
        for (path, value) in &self.cli_overrides {
            self.apply_override(config, path, value)?;
        }
        Ok(())
    }
    
    /// Apply a single override value
    fn apply_override(&self, config: &mut Config, path: &str, value: &str) -> Result<(), ConfigError> {
        debug!("Applying override: {} = {}", path, value);
        
        match path {
            "server.socket_path" => config.server.socket_path = value.to_string(),
            "server.max_connections" => {
                config.server.max_connections = value.parse()
                    .map_err(|_| ConfigError::Environment(format!("Invalid value for {}: {}", path, value)))?;
            }
            "workspace.max_workspaces" => {
                config.workspace.max_workspaces = value.parse()
                    .map_err(|_| ConfigError::Environment(format!("Invalid value for {}: {}", path, value)))?;
            }
            "workspace.default_template" => config.workspace.default_template = value.to_string(),
            "process.max_processes_per_workspace" => {
                config.process.max_processes_per_workspace = value.parse()
                    .map_err(|_| ConfigError::Environment(format!("Invalid value for {}: {}", path, value)))?;
            }
            "logging.level" => config.logging.level = value.to_string(),
            "logging.console" => {
                config.logging.console = value.parse()
                    .map_err(|_| ConfigError::Environment(format!("Invalid value for {}: {}", path, value)))?;
            }
            _ => {
                warn!("Unknown configuration override path: {}", path);
            }
        }
        
        Ok(())
    }
    
    /// Validate configuration
    fn validate_config(&self, config: &Config) -> Result<(), ConfigError> {
        // Validate server configuration
        if config.server.socket_path.is_empty() {
            return Err(ConfigError::Validation("Socket path cannot be empty".to_string()));
        }
        
        if config.server.max_connections == 0 {
            return Err(ConfigError::Validation("Max connections must be greater than 0".to_string()));
        }
        
        // Validate workspace configuration
        if config.workspace.max_workspaces == 0 {
            return Err(ConfigError::Validation("Max workspaces must be greater than 0".to_string()));
        }
        
        if config.workspace.default_template.is_empty() {
            return Err(ConfigError::Validation("Default template cannot be empty".to_string()));
        }
        
        // Validate process configuration
        if config.process.max_processes_per_workspace == 0 {
            return Err(ConfigError::Validation("Max processes per workspace must be greater than 0".to_string()));
        }
        
        // Validate logging configuration
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&config.logging.level.as_str()) {
            return Err(ConfigError::Validation(format!(
                "Invalid log level: {}. Valid levels: {:?}",
                config.logging.level, valid_log_levels
            )));
        }
        
        // Validate UI configuration
        if config.ui.dashboard.width_percentage > 100 {
            return Err(ConfigError::Validation("Dashboard width percentage cannot exceed 100".to_string()));
        }
        
        let valid_positions = ["left", "right", "top", "bottom"];
        if !valid_positions.contains(&config.ui.dashboard.position.as_str()) {
            return Err(ConfigError::Validation(format!(
                "Invalid dashboard position: {}. Valid positions: {:?}",
                config.ui.dashboard.position, valid_positions
            )));
        }
        
        Ok(())
    }
    
    /// Save configuration to file
    pub fn save_config(&self, config: &Config, path: &Path) -> Result<(), ConfigError> {
        let yaml_str = serde_yaml::to_string(config)?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, yaml_str)?;
        info!("Configuration saved to: {}", path.display());
        
        Ok(())
    }
    
    /// Save configuration to file asynchronously
    pub async fn save_config_async(&self, config: &Config, path: &Path) -> Result<(), ConfigError> {
        let yaml_str = serde_yaml::to_string(config)?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            async_fs::create_dir_all(parent).await?;
        }
        
        async_fs::write(path, yaml_str).await?;
        info!("Configuration saved to: {}", path.display());
        
        Ok(())
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use serial_test::serial;
    
    #[test]
    #[serial]
    fn test_load_default_config() {
        // Backup and clean environment variables
        let original_value = std::env::var("WEZTERM_MULTI_DEV_SOCKET_PATH").ok();
        unsafe {
            std::env::remove_var("WEZTERM_MULTI_DEV_SOCKET_PATH");
        }
        
        let loader = ConfigLoader::with_search_paths(vec![]); // Empty search paths to force default
        let config = loader.load().unwrap();
        
        assert_eq!(config.server.socket_path, "/tmp/wezterm-parallel.sock");
        assert_eq!(config.workspace.max_workspaces, 8);
        
        // Restore environment variable
        unsafe {
            match original_value {
                Some(value) => std::env::set_var("WEZTERM_MULTI_DEV_SOCKET_PATH", value),
                None => std::env::remove_var("WEZTERM_MULTI_DEV_SOCKET_PATH"),
            }
        }
    }
    
    #[test]
    #[serial]
    fn test_save_and_load_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        let loader = ConfigLoader::new();
        let config = Config::default();
        
        loader.save_config(&config, path).unwrap();
        
        let loader_with_path = ConfigLoader::with_search_paths(vec![path.to_path_buf()]);
        let loaded_config = loader_with_path.load().unwrap();
        
        assert_eq!(config.server.socket_path, loaded_config.server.socket_path);
    }
    
    #[test]
    #[serial]
    fn test_env_overrides() {
        // 現在の値を保存
        let original_value = std::env::var("WEZTERM_MULTI_DEV_SOCKET_PATH").ok();
        
        // Clear any existing environment to start clean
        unsafe {
            std::env::remove_var("WEZTERM_MULTI_DEV_SOCKET_PATH");
        }
        
        // Set the test environment variable
        unsafe {
            std::env::set_var("WEZTERM_MULTI_DEV_SOCKET_PATH", "/tmp/test.sock");
        }
        
        let loader = ConfigLoader::with_search_paths(vec![]); // Empty search paths to force default
        let config = loader.load().unwrap();
        
        assert_eq!(config.server.socket_path, "/tmp/test.sock");
        
        // 元の値を復元
        unsafe {
            match original_value {
                Some(value) => std::env::set_var("WEZTERM_MULTI_DEV_SOCKET_PATH", value),
                None => std::env::remove_var("WEZTERM_MULTI_DEV_SOCKET_PATH"),
            }
        }
    }
}