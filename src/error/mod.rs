// WezTerm Multi-Process Development Framework - Error Handling
// ユーザーフレンドリーなエラーハンドリングシステム

pub mod recovery;

use serde::{Deserialize, Serialize};
use std::fmt;

pub use recovery::{ErrorRecoveryManager, RecoveryStats};

/// ログレベル設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLevel {
    pub default: String,
    pub modules: std::collections::HashMap<String, String>,
}

impl Default for LogLevel {
    fn default() -> Self {
        let mut modules = std::collections::HashMap::new();
        modules.insert("wezterm_parallel".to_string(), "info".to_string());
        modules.insert("wezterm_parallel::room".to_string(), "debug".to_string());
        modules.insert("wezterm_parallel::error".to_string(), "debug".to_string());
        modules.insert("wezterm_parallel::process".to_string(), "info".to_string());
        
        Self {
            default: "warn".to_string(),
            modules,
        }
    }
}

/// デバッグ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    pub enabled: bool,
    pub verbose_errors: bool,
    pub stack_traces: bool,
    pub performance_metrics: bool,
    pub memory_tracking: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: cfg!(debug_assertions),
            verbose_errors: true,
            stack_traces: cfg!(debug_assertions),
            performance_metrics: false,
            memory_tracking: false,
        }
    }
}

/// エラーハンドリング設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingConfig {
    pub log_level: LogLevel,
    pub debug: DebugConfig,
    pub auto_recovery: bool,
    pub max_recovery_attempts: u32,
    pub error_reporting: bool,
}

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        Self {
            log_level: LogLevel::default(),
            debug: DebugConfig::default(),
            auto_recovery: true,
            max_recovery_attempts: 3,
            error_reporting: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserError {
    pub error_type: ErrorType,
    pub message_jp: String,
    pub message_en: String,
    pub guidance: String,
    pub recovery_actions: Vec<RecoveryAction>,
    pub error_code: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ErrorType {
    RoomError,
    ProcessError,
    FileError,
    ConfigError,
    NetworkError,
    SystemError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAction {
    pub description: String,
    pub command: Option<String>,
    pub automatic: bool,
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.error_code, self.message_jp)
    }
}

impl std::error::Error for UserError {}

impl UserError {
    pub fn room_not_found(room_name: &str) -> Self {
        Self {
            error_type: ErrorType::RoomError,
            message_jp: format!("Room '{}' が見つかりません", room_name),
            message_en: format!("Room '{}' not found", room_name),
            guidance: "Room名を確認するか、新しいRoomを作成してください".to_string(),
            recovery_actions: vec![
                RecoveryAction {
                    description: "利用可能なRoom一覧を表示 (Ctrl+Shift+W)".to_string(),
                    command: None,
                    automatic: false,
                },
                RecoveryAction {
                    description: "新しいRoomを作成 (Ctrl+Shift+N)".to_string(),
                    command: None,
                    automatic: false,
                },
            ],
            error_code: "ROOM_001".to_string(),
        }
    }

    pub fn room_creation_failed(room_name: &str, reason: &str) -> Self {
        Self {
            error_type: ErrorType::RoomError,
            message_jp: format!("Room '{}' の作成に失敗しました: {}", room_name, reason),
            message_en: format!("Failed to create room '{}': {}", room_name, reason),
            guidance: "Room名に使用できない文字が含まれていないか確認してください".to_string(),
            recovery_actions: vec![
                RecoveryAction {
                    description: "異なるRoom名で再試行".to_string(),
                    command: None,
                    automatic: false,
                },
            ],
            error_code: "ROOM_002".to_string(),
        }
    }

    pub fn claude_code_startup_failed(reason: &str) -> Self {
        Self {
            error_type: ErrorType::ProcessError,
            message_jp: format!("Claude Codeの起動に失敗しました: {}", reason),
            message_en: format!("Failed to start Claude Code: {}", reason),
            guidance: "Claude Codeが正しくインストールされているか確認してください".to_string(),
            recovery_actions: vec![
                RecoveryAction {
                    description: "Claude Codeのインストール状況を確認".to_string(),
                    command: Some("which claude-code".to_string()),
                    automatic: false,
                },
                RecoveryAction {
                    description: "手動でClaude Codeを起動".to_string(),
                    command: Some("claude-code".to_string()),
                    automatic: false,
                },
            ],
            error_code: "PROC_001".to_string(),
        }
    }

    pub fn process_communication_failed(process_id: &str) -> Self {
        Self {
            error_type: ErrorType::ProcessError,
            message_jp: format!("プロセス '{}' との通信に失敗しました", process_id),
            message_en: format!("Failed to communicate with process '{}'", process_id),
            guidance: "プロセスが正常に動作しているか確認してください".to_string(),
            recovery_actions: vec![
                RecoveryAction {
                    description: "プロセスを再起動".to_string(),
                    command: None,
                    automatic: true,
                },
            ],
            error_code: "PROC_002".to_string(),
        }
    }

    pub fn config_load_failed(file_path: &str, reason: &str) -> Self {
        Self {
            error_type: ErrorType::ConfigError,
            message_jp: format!("設定ファイルの読み込みに失敗しました: {} ({})", file_path, reason),
            message_en: format!("Failed to load config file: {} ({})", file_path, reason),
            guidance: "設定ファイルの形式が正しいか確認してください".to_string(),
            recovery_actions: vec![
                RecoveryAction {
                    description: "デフォルト設定で継続".to_string(),
                    command: None,
                    automatic: true,
                },
                RecoveryAction {
                    description: "設定ファイルを初期化".to_string(),
                    command: None,
                    automatic: false,
                },
            ],
            error_code: "CONF_001".to_string(),
        }
    }

    pub fn file_operation_failed(operation: &str, file_path: &str, reason: &str) -> Self {
        Self {
            error_type: ErrorType::FileError,
            message_jp: format!("ファイル操作に失敗しました ({}): {} - {}", operation, file_path, reason),
            message_en: format!("File operation failed ({}): {} - {}", operation, file_path, reason),
            guidance: "ファイルのアクセス権限と存在を確認してください".to_string(),
            recovery_actions: vec![
                RecoveryAction {
                    description: "ディレクトリを作成".to_string(),
                    command: None,
                    automatic: true,
                },
            ],
            error_code: "FILE_001".to_string(),
        }
    }

    pub fn system_resource_exhausted(resource: &str) -> Self {
        Self {
            error_type: ErrorType::SystemError,
            message_jp: format!("システムリソースが不足しています: {}", resource),
            message_en: format!("System resource exhausted: {}", resource),
            guidance: "システムの負荷を下げるか、不要なプロセスを終了してください".to_string(),
            recovery_actions: vec![
                RecoveryAction {
                    description: "古いプロセスを自動停止".to_string(),
                    command: None,
                    automatic: true,
                },
            ],
            error_code: "SYS_001".to_string(),
        }
    }

    pub fn task_not_found(task_id: &str) -> Self {
        Self {
            error_type: ErrorType::ProcessError,
            message_jp: format!("タスク '{}' が見つかりません", task_id),
            message_en: format!("Task '{}' not found", task_id),
            guidance: "タスクIDが正しいか確認するか、タスク一覧を確認してください".to_string(),
            recovery_actions: vec![
                RecoveryAction {
                    description: "アクティブなタスク一覧を表示".to_string(),
                    command: None,
                    automatic: false,
                },
            ],
            error_code: "TASK_001".to_string(),
        }
    }

    pub fn task_queue_full() -> Self {
        Self {
            error_type: ErrorType::ProcessError,
            message_jp: "タスクキューが満杯です".to_string(),
            message_en: "Task queue is full".to_string(),
            guidance: "既存のタスクが完了するまで待つか、タスクをキャンセルしてください".to_string(),
            recovery_actions: vec![
                RecoveryAction {
                    description: "完了済みタスクを自動クリーンアップ".to_string(),
                    command: None,
                    automatic: true,
                },
                RecoveryAction {
                    description: "古いタスクを停止".to_string(),
                    command: None,
                    automatic: false,
                },
            ],
            error_code: "TASK_002".to_string(),
        }
    }

    pub fn task_timeout(task_id: &str, timeout_duration: std::time::Duration) -> Self {
        Self {
            error_type: ErrorType::ProcessError,
            message_jp: format!("タスク '{}' がタイムアウトしました ({:?})", task_id, timeout_duration),
            message_en: format!("Task '{}' timed out ({:?})", task_id, timeout_duration),
            guidance: "タスクの処理時間を確認し、必要に応じてタイムアウト値を調整してください".to_string(),
            recovery_actions: vec![
                RecoveryAction {
                    description: "タスクを再実行".to_string(),
                    command: None,
                    automatic: false,
                },
                RecoveryAction {
                    description: "タスクをキャンセル".to_string(),
                    command: None,
                    automatic: true,
                },
            ],
            error_code: "TASK_003".to_string(),
        }
    }

    pub fn task_dependency_failed(task_id: &str, dependency: &str) -> Self {
        Self {
            error_type: ErrorType::ProcessError,
            message_jp: format!("タスク '{}' の依存関係 '{}' が満たされていません", task_id, dependency),
            message_en: format!("Task '{}' dependency '{}' not met", task_id, dependency),
            guidance: "依存するタスクまたはリソースが利用可能か確認してください".to_string(),
            recovery_actions: vec![
                RecoveryAction {
                    description: "依存関係を自動解決".to_string(),
                    command: None,
                    automatic: true,
                },
            ],
            error_code: "TASK_004".to_string(),
        }
    }

    /// エラーの重要度を取得
    pub fn severity(&self) -> ErrorSeverity {
        match self.error_type {
            ErrorType::SystemError => ErrorSeverity::Critical,
            ErrorType::ProcessError => ErrorSeverity::High,
            ErrorType::RoomError | ErrorType::ConfigError => ErrorSeverity::Medium,
            ErrorType::FileError | ErrorType::NetworkError => ErrorSeverity::Low,
        }
    }

    /// 自動回復を実行
    pub fn execute_auto_recovery(&self) -> bool {
        self.recovery_actions.iter().any(|action| action.automatic)
    }

    /// デバッグ情報付きエラーメッセージを生成
    pub fn with_debug_info(&self, config: &DebugConfig) -> String {
        let mut message = format!("[{}] {}", self.error_code, self.message_jp);
        
        if config.verbose_errors {
            message.push_str(&format!("\n英語メッセージ: {}", self.message_en));
            message.push_str(&format!("\nエラータイプ: {:?}", self.error_type));
            message.push_str(&format!("\n重要度: {:?}", self.severity()));
        }
        
        if config.stack_traces && config.enabled {
            message.push_str("\nスタックトレース: (実装中)");
        }
        
        message.push_str(&format!("\nガイダンス: {}", self.guidance));
        
        if !self.recovery_actions.is_empty() {
            message.push_str("\n回復アクション:");
            for (i, action) in self.recovery_actions.iter().enumerate() {
                message.push_str(&format!("\n  {}. {} (自動: {})", i + 1, action.description, action.automatic));
            }
        }
        
        message
    }

    /// パフォーマンス情報を追加
    pub fn with_performance_info(&self, duration: std::time::Duration, memory_usage: u64) -> String {
        format!("{}\n[パフォーマンス] 処理時間: {:?}, メモリ使用量: {}MB", 
                self.message_jp, duration, memory_usage / 1024 / 1024)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// エラー結果型のエイリアス
pub type Result<T> = std::result::Result<T, UserError>;

/// 安全なunwrap操作のためのヘルパーマクロ
#[macro_export]
macro_rules! safe_unwrap {
    ($result:expr, $error_msg:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => {
                log::error!("Operation failed: {} - {}", $error_msg, e);
                return Err(UserError::system_resource_exhausted(&format!("{}: {}", $error_msg, e)));
            }
        }
    };
    ($option:expr, $error_msg:expr, $error_type:expr) => {
        match $option {
            Some(val) => val,
            None => {
                log::error!("Value not found: {}", $error_msg);
                return Err($error_type);
            }
        }
    };
}

/// 安全なファイル操作ヘルパー
pub fn safe_file_operation<F, T>(operation: &str, file_path: &str, f: F) -> Result<T>
where
    F: FnOnce() -> std::result::Result<T, std::io::Error>,
{
    match f() {
        Ok(result) => Ok(result),
        Err(e) => {
            log::error!("File operation '{}' failed for '{}': {}", operation, file_path, e);
            Err(UserError::file_operation_failed(operation, file_path, &e.to_string()))
        }
    }
}

/// 安全なプロセス操作ヘルパー
pub fn safe_process_operation<F, T>(process_id: &str, f: F) -> Result<T>
where
    F: FnOnce() -> std::result::Result<T, Box<dyn std::error::Error>>,
{
    match f() {
        Ok(result) => Ok(result),
        Err(e) => {
            log::error!("Process operation failed for '{}': {}", process_id, e);
            Err(UserError::process_communication_failed(process_id))
        }
    }
}

/// ロック競合を安全に処理するヘルパー
pub fn safe_lock_operation<T, F, R>(operation_name: &str, f: F) -> std::result::Result<R, UserError>
where
    F: FnOnce() -> std::result::Result<R, T>,
{
    match f() {
        Ok(result) => Ok(result),
        Err(_) => {
            log::error!("Lock contention in operation: {}", operation_name);
            Err(UserError::system_resource_exhausted(&format!("Lock contention: {}", operation_name)))
        }
    }
}

/// 非同期タスクを安全に実行するヘルパー
pub async fn safe_async_operation<F, Fut, T>(operation_name: &str, f: F) -> Result<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, Box<dyn std::error::Error>>>,
{
    match f().await {
        Ok(result) => Ok(result),
        Err(e) => {
            log::error!("Async operation '{}' failed: {}", operation_name, e);
            Err(UserError::system_resource_exhausted(&format!("{}: {}", operation_name, e)))
        }
    }
}

/// 標準エラーからUserErrorへの変換
impl From<std::io::Error> for UserError {
    fn from(err: std::io::Error) -> Self {
        Self::file_operation_failed("IO操作", "未知のファイル", &err.to_string())
    }
}

impl From<serde_json::Error> for UserError {
    fn from(err: serde_json::Error) -> Self {
        Self::config_load_failed("JSON設定", &err.to_string())
    }
}

impl From<serde_yaml::Error> for UserError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::config_load_failed("YAML設定", &err.to_string())
    }
}

impl From<crate::task::TaskError> for UserError {
    fn from(err: crate::task::TaskError) -> Self {
        match err {
            crate::task::TaskError::TaskNotFound(id) => Self::task_not_found(&id),
            crate::task::TaskError::QueueFull => Self::task_queue_full(),
            crate::task::TaskError::Timeout(id) => Self::task_timeout(&id, std::time::Duration::from_secs(300)),
            crate::task::TaskError::DependencyNotMet(dep) => Self::task_dependency_failed("unknown", &dep),
            crate::task::TaskError::ExecutionFailed(msg) => Self::system_resource_exhausted(&format!("Task execution: {}", msg)),
            crate::task::TaskError::InvalidConfig(msg) => Self::config_load_failed("Task configuration", &msg),
            crate::task::TaskError::ResourceUnavailable(res) => Self::system_resource_exhausted(&res),
            crate::task::TaskError::PersistenceError(msg) => Self::file_operation_failed("persistence", "task_data", &msg),
            crate::task::TaskError::SerializationError(msg) => Self::config_load_failed("Task serialization", &msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_not_found_error() {
        let error = UserError::room_not_found("test-room");
        assert_eq!(error.error_code, "ROOM_001");
        assert!(error.message_jp.contains("test-room"));
        assert!(!error.recovery_actions.is_empty());
    }

    #[test]
    fn test_claude_code_startup_error() {
        let error = UserError::claude_code_startup_failed("バイナリが見つかりません");
        assert_eq!(error.error_code, "PROC_001");
        assert_eq!(error.severity() as u8, ErrorSeverity::High as u8);
    }

    #[test]
    fn test_auto_recovery_detection() {
        let error = UserError::process_communication_failed("test-process");
        assert!(error.execute_auto_recovery());
    }

    #[test]
    fn test_error_serialization() {
        let error = UserError::room_not_found("test");
        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: UserError = serde_json::from_str(&serialized).unwrap();
        assert_eq!(error.error_code, deserialized.error_code);
    }

    #[test]
    fn test_debug_info_generation() {
        let error = UserError::room_not_found("test-room");
        let debug_config = DebugConfig {
            enabled: true,
            verbose_errors: true,
            stack_traces: false,
            performance_metrics: true,
            memory_tracking: true,
        };
        
        let debug_message = error.with_debug_info(&debug_config);
        assert!(debug_message.contains("ROOM_001"));
        assert!(debug_message.contains("test-room"));
        assert!(debug_message.contains("英語メッセージ"));
        assert!(debug_message.contains("エラータイプ"));
    }

    #[test]
    fn test_performance_info() {
        let error = UserError::room_not_found("test");
        let duration = std::time::Duration::from_millis(100);
        let memory_usage = 1024 * 1024 * 50; // 50MB
        
        let perf_message = error.with_performance_info(duration, memory_usage);
        assert!(perf_message.contains("処理時間"));
        assert!(perf_message.contains("50MB"));
    }

    #[test]
    fn test_error_handling_config_default() {
        let config = ErrorHandlingConfig::default();
        assert!(config.auto_recovery);
        assert_eq!(config.max_recovery_attempts, 3);
        assert!(config.error_reporting);
    }

    #[test]
    fn test_log_level_config() {
        let log_level = LogLevel::default();
        assert_eq!(log_level.default, "warn");
        assert!(log_level.modules.contains_key("wezterm_parallel"));
    }
}