use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::env;
use serde::{Deserialize, Serialize};
use log::{debug, info};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Claude Code固有のプロセス起動設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeConfig {
    /// Claude Codeバイナリのパス
    pub binary_path: PathBuf,
    
    /// ワークディレクトリ
    pub working_directory: PathBuf,
    
    /// Claude Code用の環境変数
    pub environment: HashMap<String, String>,
    
    /// コマンドライン引数
    pub arguments: Vec<String>,
    
    /// プロセス起動時のタイムアウト（秒）
    pub startup_timeout: u64,
    
    /// メモリ制限（MB）
    pub memory_limit: Option<u64>,
    
    /// CPU使用率制限（％）
    pub cpu_limit: Option<f64>,
    
    /// ワークスペース固有の設定
    pub workspace_specific: WorkspaceSpecificConfig,
}

/// ワークスペース固有の設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSpecificConfig {
    /// プロジェクトルートディレクトリ
    pub project_root: Option<PathBuf>,
    
    /// プロジェクト名
    pub project_name: Option<String>,
    
    /// 追加の環境変数
    pub additional_env: HashMap<String, String>,
    
    /// Claude Code用の追加引数
    pub additional_args: Vec<String>,
    
    /// プロセス優先度（-20〜19、低い値ほど高優先度）
    pub process_priority: Option<i8>,
}

impl ClaudeCodeConfig {
    /// 新しいClaudeCodeConfigを作成
    pub fn new(binary_path: PathBuf, workspace_name: &str) -> Self {
        let mut environment = HashMap::new();
        
        // Claude Code用の基本環境変数を設定
        environment.insert("CLAUDE_WORKSPACE".to_string(), workspace_name.to_string());
        environment.insert("CLAUDE_PROCESS_ID".to_string(), uuid::Uuid::new_v4().to_string());
        
        // 色やUIの設定
        environment.insert("FORCE_COLOR".to_string(), "1".to_string());
        environment.insert("NO_COLOR".to_string(), "0".to_string());
        
        // デフォルトの作業ディレクトリを設定
        let working_directory = env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/tmp"));

        Self {
            binary_path,
            working_directory,
            environment,
            arguments: vec!["--interactive".to_string()],
            startup_timeout: 30,
            memory_limit: Some(2048), // 2GB
            cpu_limit: Some(80.0),    // 80%
            workspace_specific: WorkspaceSpecificConfig::default(),
        }
    }

    /// ワークスペース専用の設定を構築
    pub fn for_workspace(binary_path: PathBuf, workspace_name: &str, project_root: Option<PathBuf>) -> Self {
        let mut config = Self::new(binary_path, workspace_name);
        
        // プロジェクトルートが指定されている場合は設定
        if let Some(root) = project_root {
            config.working_directory = root.clone();
            config.workspace_specific.project_root = Some(root);
        }
        
        config.workspace_specific.project_name = Some(workspace_name.to_string());
        
        // ワークスペース名をClaude Codeのタイトルに設定
        config.arguments.push(format!("--title={}", workspace_name));
        
        config
    }

    /// 環境変数を追加
    pub fn add_environment_variable(&mut self, key: String, value: String) {
        self.environment.insert(key, value);
    }

    /// コマンドライン引数を追加
    pub fn add_argument(&mut self, arg: String) {
        self.arguments.push(arg);
    }

    /// ワークスペース固有の環境変数を追加
    pub fn add_workspace_environment(&mut self, key: String, value: String) {
        self.workspace_specific.additional_env.insert(key, value);
    }

    /// ワークスペース固有の引数を追加
    pub fn add_workspace_argument(&mut self, arg: String) {
        self.workspace_specific.additional_args.push(arg);
    }

    /// メモリ制限を設定
    pub fn set_memory_limit(&mut self, limit_mb: u64) {
        self.memory_limit = Some(limit_mb);
    }

    /// CPU制限を設定
    pub fn set_cpu_limit(&mut self, limit_percent: f64) {
        self.cpu_limit = Some(limit_percent.clamp(0.0, 100.0));
    }

    /// プロセス優先度を設定
    pub fn set_process_priority(&mut self, priority: i8) {
        self.workspace_specific.process_priority = Some(priority.clamp(-20, 19));
    }

    /// 完全な環境変数マップを取得（基本環境変数 + ワークスペース固有）
    pub fn get_complete_environment(&self) -> HashMap<String, String> {
        let mut env = self.environment.clone();
        
        // ワークスペース固有の環境変数をマージ
        for (key, value) in &self.workspace_specific.additional_env {
            env.insert(key.clone(), value.clone());
        }
        
        // プロジェクトルートが設定されている場合はPWDとして設定
        if let Some(ref project_root) = self.workspace_specific.project_root {
            env.insert("PWD".to_string(), project_root.to_string_lossy().to_string());
        }
        
        env
    }

    /// 完全なコマンドライン引数リストを取得（基本引数 + ワークスペース固有）
    pub fn get_complete_arguments(&self) -> Vec<String> {
        let mut args = self.arguments.clone();
        
        // ワークスペース固有の引数を追加
        args.extend(self.workspace_specific.additional_args.clone());
        
        args
    }

    /// 設定をvalidate
    pub fn validate(&self) -> Result<()> {
        // バイナリパスの存在確認
        if !self.binary_path.exists() {
            return Err(format!("Claude Code binary not found: {:?}", self.binary_path).into());
        }

        // 作業ディレクトリの存在確認
        if !self.working_directory.exists() {
            return Err(format!("Working directory not found: {:?}", self.working_directory).into());
        }

        // メモリ制限の妥当性チェック
        if let Some(memory) = self.memory_limit {
            if memory < 128 {
                return Err("Memory limit too low (minimum 128MB)".into());
            }
            if memory > 32768 {
                return Err("Memory limit too high (maximum 32GB)".into());
            }
        }

        // CPU制限の妥当性チェック
        if let Some(cpu) = self.cpu_limit {
            if cpu <= 0.0 || cpu > 100.0 {
                return Err("CPU limit must be between 0 and 100".into());
            }
        }

        Ok(())
    }

    /// 設定をコマンドライン用の形式で出力（デバッグ用）
    pub fn to_command_string(&self) -> String {
        let env_vars: Vec<String> = self.get_complete_environment()
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        let args = self.get_complete_arguments().join(" ");
        
        format!(
            "cd {:?} && {} {} {}",
            self.working_directory,
            env_vars.join(" "),
            self.binary_path.display(),
            args
        )
    }
}

impl Default for WorkspaceSpecificConfig {
    fn default() -> Self {
        Self {
            project_root: None,
            project_name: None,
            additional_env: HashMap::new(),
            additional_args: Vec::new(),
            process_priority: None,
        }
    }
}

/// 設定ビルダー - 流暢なインターフェースで設定を構築
pub struct ClaudeCodeConfigBuilder {
    config: ClaudeCodeConfig,
}

impl ClaudeCodeConfigBuilder {
    /// 新しいビルダーを作成
    pub fn new(binary_path: PathBuf, workspace_name: &str) -> Self {
        Self {
            config: ClaudeCodeConfig::new(binary_path, workspace_name),
        }
    }

    /// 作業ディレクトリを設定
    pub fn working_directory<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.config.working_directory = dir.as_ref().to_path_buf();
        self
    }

    /// 環境変数を設定
    pub fn environment(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.add_environment_variable(key.into(), value.into());
        self
    }

    /// 引数を追加
    pub fn argument(mut self, arg: impl Into<String>) -> Self {
        self.config.add_argument(arg.into());
        self
    }

    /// メモリ制限を設定
    pub fn memory_limit(mut self, limit_mb: u64) -> Self {
        self.config.set_memory_limit(limit_mb);
        self
    }

    /// CPU制限を設定
    pub fn cpu_limit(mut self, limit_percent: f64) -> Self {
        self.config.set_cpu_limit(limit_percent);
        self
    }

    /// プロジェクトルートを設定
    pub fn project_root<P: AsRef<Path>>(mut self, root: P) -> Self {
        let root_path = root.as_ref().to_path_buf();
        self.config.working_directory = root_path.clone();
        self.config.workspace_specific.project_root = Some(root_path);
        self
    }

    /// プロセス優先度を設定
    pub fn process_priority(mut self, priority: i8) -> Self {
        self.config.set_process_priority(priority);
        self
    }

    /// 設定を構築
    pub fn build(self) -> Result<ClaudeCodeConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_new_config() {
        let binary_path = PathBuf::from("/usr/local/bin/claude-code");
        let config = ClaudeCodeConfig::new(binary_path.clone(), "test-workspace");

        assert_eq!(config.binary_path, binary_path);
        assert_eq!(config.environment.get("CLAUDE_WORKSPACE").unwrap(), "test-workspace");
        assert!(config.environment.contains_key("CLAUDE_PROCESS_ID"));
        assert_eq!(config.arguments, vec!["--interactive"]);
        assert_eq!(config.startup_timeout, 30);
        assert_eq!(config.memory_limit, Some(2048));
        assert_eq!(config.cpu_limit, Some(80.0));
    }

    #[test]
    fn test_for_workspace() {
        let binary_path = PathBuf::from("/usr/local/bin/claude-code");
        let project_root = PathBuf::from("/tmp/test-project");
        let config = ClaudeCodeConfig::for_workspace(
            binary_path.clone(), 
            "my-workspace", 
            Some(project_root.clone())
        );

        assert_eq!(config.working_directory, project_root);
        assert_eq!(config.workspace_specific.project_root, Some(project_root));
        assert_eq!(config.workspace_specific.project_name, Some("my-workspace".to_string()));
        assert!(config.arguments.contains(&"--title=my-workspace".to_string()));
    }

    #[test]
    fn test_environment_variables() {
        let binary_path = PathBuf::from("/usr/local/bin/claude-code");
        let mut config = ClaudeCodeConfig::new(binary_path, "test");

        config.add_environment_variable("TEST_VAR".to_string(), "test_value".to_string());
        config.add_workspace_environment("WORKSPACE_VAR".to_string(), "workspace_value".to_string());

        let complete_env = config.get_complete_environment();
        assert_eq!(complete_env.get("TEST_VAR").unwrap(), "test_value");
        assert_eq!(complete_env.get("WORKSPACE_VAR").unwrap(), "workspace_value");
    }

    #[test]
    fn test_arguments() {
        let binary_path = PathBuf::from("/usr/local/bin/claude-code");
        let mut config = ClaudeCodeConfig::new(binary_path, "test");

        config.add_argument("--verbose".to_string());
        config.add_workspace_argument("--project=test".to_string());

        let complete_args = config.get_complete_arguments();
        assert!(complete_args.contains(&"--interactive".to_string()));
        assert!(complete_args.contains(&"--verbose".to_string()));
        assert!(complete_args.contains(&"--project=test".to_string()));
    }

    #[test]
    fn test_limits() {
        let binary_path = PathBuf::from("/usr/local/bin/claude-code");
        let mut config = ClaudeCodeConfig::new(binary_path, "test");

        config.set_memory_limit(4096);
        config.set_cpu_limit(50.0);
        config.set_process_priority(-5);

        assert_eq!(config.memory_limit, Some(4096));
        assert_eq!(config.cpu_limit, Some(50.0));
        assert_eq!(config.workspace_specific.process_priority, Some(-5));
    }

    #[test]
    fn test_limit_clamping() {
        let binary_path = PathBuf::from("/usr/local/bin/claude-code");
        let mut config = ClaudeCodeConfig::new(binary_path, "test");

        // CPU制限のクランピングテスト
        config.set_cpu_limit(150.0);
        assert_eq!(config.cpu_limit, Some(100.0));

        config.set_cpu_limit(-10.0);
        assert_eq!(config.cpu_limit, Some(0.0));

        // プロセス優先度のクランピングテスト
        config.set_process_priority(50);
        assert_eq!(config.workspace_specific.process_priority, Some(19));

        config.set_process_priority(-50);
        assert_eq!(config.workspace_specific.process_priority, Some(-20));
    }

    #[test]
    fn test_validation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let binary_path = temp_dir.path().join("claude-code");
        fs::File::create(&binary_path)?;

        let config = ClaudeCodeConfig::new(binary_path, "test");
        
        // 作業ディレクトリが存在しないケース
        let mut invalid_config = config.clone();
        invalid_config.working_directory = PathBuf::from("/nonexistent/directory");
        assert!(invalid_config.validate().is_err());

        // メモリ制限が低すぎるケース
        let mut invalid_config = config.clone();
        invalid_config.memory_limit = Some(64);
        assert!(invalid_config.validate().is_err());

        // CPU制限が無効なケース
        let mut invalid_config = config.clone();
        invalid_config.cpu_limit = Some(150.0);
        assert!(invalid_config.validate().is_err());

        Ok(())
    }

    #[test]
    fn test_builder_pattern() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let binary_path = temp_dir.path().join("claude-code");
        fs::File::create(&binary_path)?;

        let config = ClaudeCodeConfigBuilder::new(binary_path, "test-workspace")
            .working_directory(temp_dir.path())
            .environment("TEST_ENV", "test_value")
            .argument("--verbose")
            .memory_limit(1024)
            .cpu_limit(60.0)
            .process_priority(5)
            .build()?;

        assert_eq!(config.working_directory, temp_dir.path());
        assert_eq!(config.environment.get("TEST_ENV").unwrap(), "test_value");
        assert!(config.arguments.contains(&"--verbose".to_string()));
        assert_eq!(config.memory_limit, Some(1024));
        assert_eq!(config.cpu_limit, Some(60.0));
        assert_eq!(config.workspace_specific.process_priority, Some(5));

        Ok(())
    }

    #[test]
    fn test_command_string_generation() {
        let binary_path = PathBuf::from("/usr/local/bin/claude-code");
        let config = ClaudeCodeConfig::new(binary_path, "test");

        let command_str = config.to_command_string();
        assert!(command_str.contains("claude-code"));
        assert!(command_str.contains("CLAUDE_WORKSPACE=test"));
        assert!(command_str.contains("--interactive"));
    }
}