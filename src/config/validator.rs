use super::{Config, LoggingConfig, ProcessConfig, ServerConfig, UiConfig, WorkspaceConfig};

pub struct ConfigValidator;

impl ConfigValidator {
    pub fn validate(config: &Config) -> Result<(), String> {
        Self::validate_server_config(&config.server)?;
        Self::validate_workspace_config(&config.workspace)?;
        Self::validate_process_config(&config.process)?;
        Self::validate_ui_config(&config.ui)?;
        Self::validate_logging_config(&config.logging)?;
        Ok(())
    }

    fn validate_server_config(config: &ServerConfig) -> Result<(), String> {
        if config.socket_path.is_empty() {
            return Err("Socket path cannot be empty".to_string());
        }
        if config.max_connections == 0 {
            return Err("Maximum connections cannot be 0".to_string());
        }
        Ok(())
    }

    fn validate_workspace_config(config: &WorkspaceConfig) -> Result<(), String> {
        if config.max_workspaces == 0 {
            return Err("Maximum workspaces cannot be 0".to_string());
        }
        if config.default_template.is_empty() {
            return Err("Default template cannot be empty".to_string());
        }
        Ok(())
    }

    fn validate_process_config(config: &ProcessConfig) -> Result<(), String> {
        if config.max_processes_per_workspace == 0 {
            return Err("Maximum processes per workspace cannot be 0".to_string());
        }
        Ok(())
    }

    fn validate_ui_config(_config: &UiConfig) -> Result<(), String> {
        Ok(())
    }

    fn validate_logging_config(config: &LoggingConfig) -> Result<(), String> {
        match config.level.as_str() {
            "error" | "warn" | "info" | "debug" | "trace" => Ok(()),
            _ => Err(format!("Invalid log level: {}", config.level)),
        }
    }
}
