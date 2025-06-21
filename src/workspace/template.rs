// WezTerm Multi-Process Development Framework - Workspace Template System

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::workspace::state::{WorkspaceConfig, LayoutConfig, LayoutType, SplitDirection};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkspaceTemplate {
    pub name: String,
    pub description: String,
    pub layout: LayoutConfig,
    pub default_commands: Vec<CommandTemplate>,
    pub environment_vars: HashMap<String, String>,
    pub required_tools: Vec<String>,
    pub startup_script: Option<String>,
    pub keybindings: HashMap<String, String>,
    pub theme: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandTemplate {
    pub name: String,
    pub command: String,
    pub working_directory: Option<String>,
    pub pane_position: Option<PaneTemplatePosition>,
    pub auto_start: bool,
    pub restart_on_exit: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaneTemplatePosition {
    pub row: u32,
    pub col: u32,
    pub size_percentage: f32,
}

#[derive(Debug)]
pub struct TemplateEngine {
    templates: HashMap<String, WorkspaceTemplate>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            templates: HashMap::new(),
        };
        
        // Register built-in templates
        engine.register_builtin_templates();
        engine
    }

    pub fn register_template(&mut self, template: WorkspaceTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    pub fn get_template(&self, name: &str) -> Option<&WorkspaceTemplate> {
        self.templates.get(name)
    }

    pub fn list_templates(&self) -> Vec<&WorkspaceTemplate> {
        self.templates.values().collect()
    }

    pub fn apply_template(&self, template_name: &str, workspace_name: &str) -> Result<WorkspaceConfig, String> {
        let template = self.get_template(template_name)
            .ok_or_else(|| format!("Template '{}' not found", template_name))?;

        let mut config = WorkspaceConfig {
            name: workspace_name.to_string(),
            template: template_name.to_string(),
            auto_start_processes: true,
            max_processes: 8,
            working_directory: std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("/"))
                .to_string_lossy()
                .to_string(),
            environment_vars: template.environment_vars.clone(),
            startup_commands: template.default_commands
                .iter()
                .filter(|cmd| cmd.auto_start)
                .map(|cmd| cmd.command.clone())
                .collect(),
            keybindings: template.keybindings.clone(),
            theme: template.theme.clone(),
        };

        // Apply template-specific workspace directory if needed
        if let Some(first_cmd) = template.default_commands.first() {
            if let Some(ref wd) = first_cmd.working_directory {
                config.working_directory = wd.clone();
            }
        }

        Ok(config)
    }

    fn register_builtin_templates(&mut self) {
        // Basic template
        let basic_template = WorkspaceTemplate {
            name: "basic".to_string(),
            description: "Basic single-process workspace".to_string(),
            layout: LayoutConfig {
                layout_type: LayoutType::Single,
                primary_direction: SplitDirection::Horizontal,
                pane_sizes: vec![100.0],
                auto_balance: false,
            },
            default_commands: vec![
                CommandTemplate {
                    name: "claude-code".to_string(),
                    command: "claude-code".to_string(),
                    working_directory: None,
                    pane_position: None,
                    auto_start: true,
                    restart_on_exit: true,
                }
            ],
            environment_vars: HashMap::new(),
            required_tools: vec!["claude-code".to_string()],
            startup_script: None,
            keybindings: HashMap::new(),
            theme: None,
        };

        // Web development template
        let web_dev_template = WorkspaceTemplate {
            name: "web_dev".to_string(),
            description: "Web development with frontend/backend separation".to_string(),
            layout: LayoutConfig {
                layout_type: LayoutType::FourPaneGrid,
                primary_direction: SplitDirection::Horizontal,
                pane_sizes: vec![25.0, 25.0, 25.0, 25.0],
                auto_balance: true,
            },
            default_commands: vec![
                CommandTemplate {
                    name: "frontend-claude".to_string(),
                    command: "claude-code --workspace=frontend".to_string(),
                    working_directory: Some("./frontend".to_string()),
                    pane_position: Some(PaneTemplatePosition { row: 0, col: 0, size_percentage: 25.0 }),
                    auto_start: true,
                    restart_on_exit: true,
                },
                CommandTemplate {
                    name: "backend-claude".to_string(),
                    command: "claude-code --workspace=backend".to_string(),
                    working_directory: Some("./backend".to_string()),
                    pane_position: Some(PaneTemplatePosition { row: 0, col: 1, size_percentage: 25.0 }),
                    auto_start: true,
                    restart_on_exit: true,
                },
                CommandTemplate {
                    name: "dev-server".to_string(),
                    command: "npm run dev".to_string(),
                    working_directory: Some("./frontend".to_string()),
                    pane_position: Some(PaneTemplatePosition { row: 1, col: 0, size_percentage: 25.0 }),
                    auto_start: false,
                    restart_on_exit: false,
                },
                CommandTemplate {
                    name: "logs".to_string(),
                    command: "tail -f logs/app.log".to_string(),
                    working_directory: Some("./backend".to_string()),
                    pane_position: Some(PaneTemplatePosition { row: 1, col: 1, size_percentage: 25.0 }),
                    auto_start: false,
                    restart_on_exit: false,
                }
            ],
            environment_vars: {
                let mut env = HashMap::new();
                env.insert("NODE_ENV".to_string(), "development".to_string());
                env.insert("RUST_LOG".to_string(), "info".to_string());
                env
            },
            required_tools: vec![
                "claude-code".to_string(),
                "npm".to_string(),
                "node".to_string(),
                "cargo".to_string()
            ],
            startup_script: Some("./scripts/setup-dev-env.sh".to_string()),
            keybindings: {
                let mut keys = HashMap::new();
                keys.insert("ctrl+shift+r".to_string(), "restart_dev_server".to_string());
                keys.insert("ctrl+shift+l".to_string(), "show_logs".to_string());
                keys
            },
            theme: Some("dark".to_string()),
        };

        // Parallel development template
        let parallel_dev_template = WorkspaceTemplate {
            name: "parallel_dev".to_string(),
            description: "High-performance parallel development with multiple Claude Code instances".to_string(),
            layout: LayoutConfig {
                layout_type: LayoutType::ThreePaneHorizontal,
                primary_direction: SplitDirection::Horizontal,
                pane_sizes: vec![33.3, 33.3, 33.4],
                auto_balance: true,
            },
            default_commands: vec![
                CommandTemplate {
                    name: "claude-main".to_string(),
                    command: "claude-code --workspace=main --priority=high".to_string(),
                    working_directory: None,
                    pane_position: Some(PaneTemplatePosition { row: 0, col: 0, size_percentage: 33.3 }),
                    auto_start: true,
                    restart_on_exit: true,
                },
                CommandTemplate {
                    name: "claude-test".to_string(),
                    command: "claude-code --workspace=test --priority=medium".to_string(),
                    working_directory: None,
                    pane_position: Some(PaneTemplatePosition { row: 0, col: 1, size_percentage: 33.3 }),
                    auto_start: true,
                    restart_on_exit: true,
                },
                CommandTemplate {
                    name: "claude-docs".to_string(),
                    command: "claude-code --workspace=docs --priority=low".to_string(),
                    working_directory: None,
                    pane_position: Some(PaneTemplatePosition { row: 0, col: 2, size_percentage: 33.4 }),
                    auto_start: true,
                    restart_on_exit: true,
                }
            ],
            environment_vars: {
                let mut env = HashMap::new();
                env.insert("CLAUDE_PARALLEL_MODE".to_string(), "true".to_string());
                env.insert("CLAUDE_MAX_INSTANCES".to_string(), "8".to_string());
                env
            },
            required_tools: vec!["claude-code".to_string()],
            startup_script: None,
            keybindings: {
                let mut keys = HashMap::new();
                keys.insert("ctrl+shift+1".to_string(), "focus_main".to_string());
                keys.insert("ctrl+shift+2".to_string(), "focus_test".to_string());
                keys.insert("ctrl+shift+3".to_string(), "focus_docs".to_string());
                keys.insert("ctrl+shift+s".to_string(), "sync_all_panes".to_string());
                keys
            },
            theme: Some("dark".to_string()),
        };

        // Research template
        let research_template = WorkspaceTemplate {
            name: "research".to_string(),
            description: "Research and exploration workspace with documentation focus".to_string(),
            layout: LayoutConfig {
                layout_type: LayoutType::TwoPaneVertical,
                primary_direction: SplitDirection::Vertical,
                pane_sizes: vec![70.0, 30.0],
                auto_balance: false,
            },
            default_commands: vec![
                CommandTemplate {
                    name: "claude-research".to_string(),
                    command: "claude-code --mode=research".to_string(),
                    working_directory: None,
                    pane_position: Some(PaneTemplatePosition { row: 0, col: 0, size_percentage: 70.0 }),
                    auto_start: true,
                    restart_on_exit: true,
                },
                CommandTemplate {
                    name: "notes".to_string(),
                    command: "vim notes.md".to_string(),
                    working_directory: None,
                    pane_position: Some(PaneTemplatePosition { row: 1, col: 0, size_percentage: 30.0 }),
                    auto_start: false,
                    restart_on_exit: false,
                }
            ],
            environment_vars: {
                let mut env = HashMap::new();
                env.insert("CLAUDE_MODE".to_string(), "research".to_string());
                env
            },
            required_tools: vec!["claude-code".to_string(), "vim".to_string()],
            startup_script: None,
            keybindings: {
                let mut keys = HashMap::new();
                keys.insert("ctrl+shift+n".to_string(), "new_note".to_string());
                keys.insert("ctrl+shift+s".to_string(), "save_research".to_string());
                keys
            },
            theme: Some("light".to_string()),
        };

        // Register all templates
        self.register_template(basic_template);
        self.register_template(web_dev_template);
        self.register_template(parallel_dev_template);
        self.register_template(research_template);
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        let templates = engine.list_templates();
        
        // Should have built-in templates
        assert!(templates.len() >= 4);
        
        let template_names: Vec<&str> = templates.iter().map(|t| t.name.as_str()).collect();
        assert!(template_names.contains(&"basic"));
        assert!(template_names.contains(&"web_dev"));
        assert!(template_names.contains(&"parallel_dev"));
        assert!(template_names.contains(&"research"));
    }

    #[test]
    fn test_template_application() {
        let engine = TemplateEngine::new();
        
        let config = engine.apply_template("basic", "my-workspace").unwrap();
        
        assert_eq!(config.name, "my-workspace");
        assert_eq!(config.template, "basic");
        assert!(config.auto_start_processes);
        assert!(!config.startup_commands.is_empty());
        assert_eq!(config.startup_commands[0], "claude-code");
    }

    #[test]
    fn test_web_dev_template() {
        let engine = TemplateEngine::new();
        
        let template = engine.get_template("web_dev").unwrap();
        
        assert_eq!(template.name, "web_dev");
        assert_eq!(template.default_commands.len(), 4);
        assert!(template.environment_vars.contains_key("NODE_ENV"));
        assert!(template.required_tools.contains(&"npm".to_string()));
    }

    #[test]
    fn test_parallel_dev_template() {
        let engine = TemplateEngine::new();
        
        let template = engine.get_template("parallel_dev").unwrap();
        
        assert_eq!(template.name, "parallel_dev");
        assert_eq!(template.default_commands.len(), 3);
        
        // Check that all Claude Code commands auto-start
        for cmd in &template.default_commands {
            assert!(cmd.auto_start);
            assert!(cmd.command.contains("claude-code"));
        }
    }

    #[test]
    fn test_custom_template_registration() {
        let mut engine = TemplateEngine::new();
        
        let custom_template = WorkspaceTemplate {
            name: "custom".to_string(),
            description: "Custom test template".to_string(),
            layout: LayoutConfig::default(),
            default_commands: vec![],
            environment_vars: HashMap::new(),
            required_tools: vec![],
            startup_script: None,
            keybindings: HashMap::new(),
            theme: None,
        };
        
        engine.register_template(custom_template);
        
        let retrieved = engine.get_template("custom");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "custom");
    }

    #[test]
    fn test_nonexistent_template() {
        let engine = TemplateEngine::new();
        
        let result = engine.apply_template("nonexistent", "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}