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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DashboardConfig, KeybindingConfig, ThemeConfig};

    fn create_valid_config() -> Config {
        Config {
            server: ServerConfig {
                socket_path: "/tmp/test.sock".to_string(),
                max_connections: 100,
                connection_timeout: 30,
                enable_metrics: true,
                health_check_interval: 10,
            },
            workspace: WorkspaceConfig {
                max_workspaces: 10,
                default_template: "default".to_string(),
                state_path: std::path::PathBuf::from("/tmp/workspaces.json"),
                auto_save_interval: 30,
                templates_dir: std::path::PathBuf::from("/tmp/templates"),
            },
            process: ProcessConfig {
                max_processes_per_workspace: 16,
                startup_timeout: 60,
                health_check_interval: 5,
                auto_restart: true,
                max_restart_attempts: 3,
                environment: std::collections::HashMap::new(),
                working_dir_template: "~/projects/{{workspace_name}}".to_string(),
            },
            ui: UiConfig {
                dashboard: DashboardConfig {
                    update_interval: 2.0,
                    width_percentage: 30,
                    position: "right".to_string(),
                    real_time_updates: true,
                    max_log_entries: 100,
                },
                theme: ThemeConfig {
                    background: "#1e1e2e".to_string(),
                    foreground: "#cdd6f4".to_string(),
                    border: "#45475a".to_string(),
                    header: "#89b4fa".to_string(),
                    success: "#a6e3a1".to_string(),
                    warning: "#f9e2af".to_string(),
                    error: "#f38ba8".to_string(),
                    info: "#89dceb".to_string(),
                },
                keybindings: KeybindingConfig {
                    leader_key: "CTRL|SHIFT+Space".to_string(),
                    workspace_prefix: "CTRL|SHIFT".to_string(),
                    process_prefix: "CTRL|ALT".to_string(),
                    pane_prefix: "ALT".to_string(),
                    dashboard_prefix: "CTRL|SHIFT".to_string(),
                },
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
                console: true,
                max_file_size: 104857600,
                max_files: 5,
                format: "json".to_string(),
            },
            plugins: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_validate_valid_config() {
        let config = create_valid_config();
        let result = ConfigValidator::validate(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_server_config_empty_socket_path() {
        let mut config = create_valid_config();
        config.server.socket_path = String::new();

        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Socket path cannot be empty");
    }

    #[test]
    fn test_validate_server_config_zero_max_connections() {
        let mut config = create_valid_config();
        config.server.max_connections = 0;

        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Maximum connections cannot be 0");
    }

    #[test]
    fn test_validate_server_config_valid() {
        let server_config = ServerConfig {
            socket_path: "/tmp/valid.sock".to_string(),
            max_connections: 50,
            connection_timeout: 30,
            enable_metrics: true,
            health_check_interval: 10,
        };

        let result = ConfigValidator::validate_server_config(&server_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_workspace_config_zero_max_workspaces() {
        let mut config = create_valid_config();
        config.workspace.max_workspaces = 0;

        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Maximum workspaces cannot be 0");
    }

    #[test]
    fn test_validate_workspace_config_empty_default_template() {
        let mut config = create_valid_config();
        config.workspace.default_template = String::new();

        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Default template cannot be empty");
    }

    #[test]
    fn test_validate_workspace_config_valid() {
        let workspace_config = WorkspaceConfig {
            max_workspaces: 5,
            default_template: "claude-dev".to_string(),
            state_path: std::path::PathBuf::from("/tmp/workspaces.json"),
            auto_save_interval: 30,
            templates_dir: std::path::PathBuf::from("/tmp/templates"),
        };

        let result = ConfigValidator::validate_workspace_config(&workspace_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_process_config_zero_max_processes() {
        let mut config = create_valid_config();
        config.process.max_processes_per_workspace = 0;

        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Maximum processes per workspace cannot be 0"
        );
    }

    #[test]
    fn test_validate_process_config_valid() {
        let process_config = ProcessConfig {
            max_processes_per_workspace: 8,
            startup_timeout: 60,
            health_check_interval: 5,
            auto_restart: true,
            max_restart_attempts: 3,
            environment: std::collections::HashMap::new(),
            working_dir_template: "~/projects/{{workspace_name}}".to_string(),
        };

        let result = ConfigValidator::validate_process_config(&process_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_ui_config_always_valid() {
        let ui_config = UiConfig {
            dashboard: DashboardConfig {
                update_interval: 2.0,
                width_percentage: 30,
                position: "left".to_string(),
                real_time_updates: false,
                max_log_entries: 200,
            },
            theme: ThemeConfig {
                background: "#000000".to_string(),
                foreground: "#ffffff".to_string(),
                border: "#333333".to_string(),
                header: "#0066cc".to_string(),
                success: "#00cc00".to_string(),
                warning: "#ff9900".to_string(),
                error: "#cc0000".to_string(),
                info: "#0099cc".to_string(),
            },
            keybindings: KeybindingConfig {
                leader_key: "CTRL+Space".to_string(),
                workspace_prefix: "CTRL".to_string(),
                process_prefix: "ALT".to_string(),
                pane_prefix: "SHIFT".to_string(),
                dashboard_prefix: "CTRL+SHIFT".to_string(),
            },
        };

        let result = ConfigValidator::validate_ui_config(&ui_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_logging_config_valid_levels() {
        let valid_levels = vec!["error", "warn", "info", "debug", "trace"];

        for level in valid_levels {
            let logging_config = LoggingConfig {
                level: level.to_string(),
                file_path: None,
                console: true,
                max_file_size: 104857600,
                max_files: 5,
                format: "json".to_string(),
            };

            let result = ConfigValidator::validate_logging_config(&logging_config);
            assert!(result.is_ok(), "Level '{}' should be valid", level);
        }
    }

    #[test]
    fn test_validate_logging_config_invalid_level() {
        let logging_config = LoggingConfig {
            level: "invalid".to_string(),
            file_path: None,
            console: true,
            max_file_size: 104857600,
            max_files: 5,
            format: "json".to_string(),
        };

        let result = ConfigValidator::validate_logging_config(&logging_config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid log level: invalid");
    }

    #[test]
    fn test_validate_logging_config_case_sensitive() {
        let logging_config = LoggingConfig {
            level: "INFO".to_string(), // Should be lowercase
            file_path: None,
            console: true,
            max_file_size: 104857600,
            max_files: 5,
            format: "json".to_string(),
        };

        let result = ConfigValidator::validate_logging_config(&logging_config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid log level: INFO");
    }

    #[test]
    fn test_validate_multiple_errors() {
        let mut config = create_valid_config();
        config.server.socket_path = String::new();
        config.workspace.max_workspaces = 0;
        config.process.max_processes_per_workspace = 0;
        config.logging.level = "invalid".to_string();

        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());

        // The validation should fail on the first error encountered
        assert_eq!(result.unwrap_err(), "Socket path cannot be empty");
    }

    #[test]
    fn test_validate_individual_components() {
        let config = create_valid_config();

        // Test each component individually
        assert!(ConfigValidator::validate_server_config(&config.server).is_ok());
        assert!(ConfigValidator::validate_workspace_config(&config.workspace).is_ok());
        assert!(ConfigValidator::validate_process_config(&config.process).is_ok());
        assert!(ConfigValidator::validate_ui_config(&config.ui).is_ok());
        assert!(ConfigValidator::validate_logging_config(&config.logging).is_ok());
    }

    #[test]
    fn test_validate_edge_cases() {
        // Test with minimal valid values
        let mut config = create_valid_config();
        config.server.max_connections = 1;
        config.workspace.max_workspaces = 1;
        config.process.max_processes_per_workspace = 1;

        let result = ConfigValidator::validate(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_boundary_values() {
        let mut config = create_valid_config();

        // Test with very large valid values
        config.server.max_connections = usize::MAX;
        config.workspace.max_workspaces = usize::MAX;
        config.process.max_processes_per_workspace = usize::MAX;

        let result = ConfigValidator::validate(&config);
        assert!(result.is_ok());
    }
}
