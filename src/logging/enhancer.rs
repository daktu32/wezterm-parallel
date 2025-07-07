// WezTerm Multi-Process Development Framework - Log Enhancer
// ログ強化機能とコンテキスト付きログ出力

use super::{UnifiedLogLevel, LogContext, UnifiedLogEntry};
use super::strategy::{LoggingStrategy, StrategyManager};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static;

lazy_static! {
    static ref STRATEGY_MANAGER: Arc<Mutex<StrategyManager>> = {
        Arc::new(Mutex::new(StrategyManager::from_environment()))
    };
}

/// コンテキスト付きログ出力のメイン関数
pub fn log_with_context(
    level: UnifiedLogLevel,
    context: LogContext,
    message: String,
    error: Option<String>,
    duration_ms: Option<u64>,
) {
    let strategy_manager = STRATEGY_MANAGER.lock().unwrap();
    let strategy = strategy_manager.get_strategy();

    // ログレベルチェック
    let component_level = strategy.get_log_level(&context.component);
    if level < component_level {
        return;
    }

    // レート制限チェック
    if strategy.should_rate_limit(&context) {
        return;
    }

    // ログエントリ作成
    let entry = create_log_entry(level, context, message, error, duration_ms);

    // 出力
    output_log_entry(&entry, strategy);
}

/// ログエントリを作成
fn create_log_entry(
    level: UnifiedLogLevel,
    context: LogContext,
    message: String,
    error: Option<String>,
    duration_ms: Option<u64>,
) -> UnifiedLogEntry {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_secs();
    
    let timestamp_str = chrono::DateTime::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| chrono::Utc::now())
        .to_rfc3339();

    // デバッグ用にファイル・行番号を取得
    let location = if level == UnifiedLogLevel::Debug || level == UnifiedLogLevel::Trace {
        Some(format!("{}:{}", file!(), line!()))
    } else {
        None
    };

    UnifiedLogEntry {
        timestamp: timestamp_str,
        level,
        context,
        message,
        error,
        duration_ms,
        location,
    }
}

/// ログエントリを実際に出力
fn output_log_entry(entry: &UnifiedLogEntry, strategy: &LoggingStrategy) {
    for output in &strategy.outputs {
        match output {
            super::strategy::LogOutput::Stdout => {
                if strategy.structured_output {
                    println!("{}", serde_json::to_string(entry).unwrap_or_else(|_| format!("{:?}", entry)));
                } else {
                    println!("{}", format_human_readable(entry));
                }
            },
            super::strategy::LogOutput::Stderr => {
                if strategy.structured_output {
                    eprintln!("{}", serde_json::to_string(entry).unwrap_or_else(|_| format!("{:?}", entry)));
                } else {
                    eprintln!("{}", format_human_readable(entry));
                }
            },
            super::strategy::LogOutput::File { path, .. } => {
                // ファイル出力は簡略化（実際には非同期で書き込み）
                let formatted = format_human_readable(entry);
                if let Err(e) = std::fs::write(path, format!("{}\n", formatted)) {
                    eprintln!("Failed to write to log file {}: {}", path, e);
                }
            },
            super::strategy::LogOutput::StructuredFile { path, .. } => {
                let json_line = serde_json::to_string(entry).unwrap_or_else(|_| format!("{:?}", entry));
                if let Err(e) = std::fs::write(path, format!("{}\n", json_line)) {
                    eprintln!("Failed to write to structured log file {}: {}", path, e);
                }
            },
            super::strategy::LogOutput::System => {
                // システムログは簡略化
                println!("[SYSTEM] {}", format_human_readable(entry));
            },
        }
    }
}

/// 人間が読みやすい形式でログをフォーマット
fn format_human_readable(entry: &UnifiedLogEntry) -> String {
    let mut output = format!(
        "{} [{}] [{}:{}]",
        entry.timestamp,
        entry.level.as_str(),
        entry.context.component,
        entry.context.operation
    );

    if let Some(entity_id) = &entry.context.entity_id {
        output.push_str(&format!(" [{}]", entity_id));
    }

    if let Some(session_id) = &entry.context.session_id {
        output.push_str(&format!(" (session:{})", session_id));
    }

    output.push_str(&format!(" {}", entry.message));

    if let Some(duration) = entry.duration_ms {
        output.push_str(&format!(" ({}ms)", duration));
    }

    if let Some(error) = &entry.error {
        output.push_str(&format!(" ERROR: {}", error));
    }

    if !entry.context.metadata.is_empty() {
        let metadata_str = entry.context.metadata
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(" ");
        output.push_str(&format!(" [{}]", metadata_str));
    }

    if let Some(location) = &entry.location {
        output.push_str(&format!(" @{}", location));
    }

    output
}

/// プロセス管理用のログヘルパー
pub mod process {
    use super::*;

    pub fn log_process_start(process_id: &str, command: &str) {
        let context = LogContext::new("process", "start")
            .with_entity_id(process_id)
            .with_metadata("command", serde_json::json!(command));
        
        log_with_context(
            UnifiedLogLevel::Info,
            context,
            format!("Starting process: {}", command),
            None,
            None,
        );
    }

    pub fn log_process_stop(process_id: &str, exit_code: Option<i32>) {
        let context = LogContext::new("process", "stop")
            .with_entity_id(process_id)
            .with_metadata("exit_code", serde_json::json!(exit_code));
        
        let message = match exit_code {
            Some(0) => "Process stopped successfully".to_string(),
            Some(code) => format!("Process stopped with exit code: {}", code),
            None => "Process terminated".to_string(),
        };
        
        log_with_context(
            UnifiedLogLevel::Info,
            context,
            message,
            None,
            None,
        );
    }

    pub fn log_process_error(process_id: &str, error: &str) {
        let context = LogContext::new("process", "error")
            .with_entity_id(process_id);
        
        log_with_context(
            UnifiedLogLevel::Error,
            context,
            "Process error occurred".to_string(),
            Some(error.to_string()),
            None,
        );
    }

    pub fn log_process_heartbeat(process_id: &str, status: &str) {
        let context = LogContext::new("process", "heartbeat")
            .with_entity_id(process_id)
            .with_metadata("status", serde_json::json!(status));
        
        log_with_context(
            UnifiedLogLevel::Debug,
            context,
            format!("Process heartbeat: {}", status),
            None,
            None,
        );
    }
}

/// IPC通信用のログヘルパー
pub mod ipc {
    use super::*;

    pub fn log_message_send(from: &str, to: &str, message_type: &str, size_bytes: usize) {
        let context = LogContext::new("ipc", "send")
            .with_entity_id(from)
            .with_metadata("to", serde_json::json!(to))
            .with_metadata("message_type", serde_json::json!(message_type))
            .with_metadata("size_bytes", serde_json::json!(size_bytes));
        
        log_with_context(
            UnifiedLogLevel::Debug,
            context,
            format!("Sending {} message to {} ({} bytes)", message_type, to, size_bytes),
            None,
            None,
        );
    }

    pub fn log_message_receive(from: &str, to: &str, message_type: &str, processing_time_ms: u64) {
        let context = LogContext::new("ipc", "receive")
            .with_entity_id(to)
            .with_metadata("from", serde_json::json!(from))
            .with_metadata("message_type", serde_json::json!(message_type));
        
        log_with_context(
            UnifiedLogLevel::Debug,
            context,
            format!("Received {} message from {}", message_type, from),
            None,
            Some(processing_time_ms),
        );
    }

    pub fn log_connection_error(endpoint: &str, error: &str) {
        let context = LogContext::new("ipc", "connection_error")
            .with_entity_id(endpoint);
        
        log_with_context(
            UnifiedLogLevel::Error,
            context,
            format!("IPC connection error to {}", endpoint),
            Some(error.to_string()),
            None,
        );
    }
}

/// 設定管理用のログヘルパー
pub mod config {
    use super::*;

    pub fn log_config_load(file_path: &str, load_time_ms: u64) {
        let context = LogContext::new("config", "load")
            .with_entity_id(file_path);
        
        log_with_context(
            UnifiedLogLevel::Info,
            context,
            format!("Loaded configuration from {}", file_path),
            None,
            Some(load_time_ms),
        );
    }

    pub fn log_config_reload(file_path: &str, changes: usize) {
        let context = LogContext::new("config", "reload")
            .with_entity_id(file_path)
            .with_metadata("changes", serde_json::json!(changes));
        
        log_with_context(
            UnifiedLogLevel::Info,
            context,
            format!("Reloaded configuration from {} ({} changes)", file_path, changes),
            None,
            None,
        );
    }

    pub fn log_config_error(file_path: &str, error: &str) {
        let context = LogContext::new("config", "error")
            .with_entity_id(file_path);
        
        log_with_context(
            UnifiedLogLevel::Error,
            context,
            format!("Configuration error in {}", file_path),
            Some(error.to_string()),
            None,
        );
    }

    pub fn log_config_validation(file_path: &str, is_valid: bool, warnings: usize) {
        let context = LogContext::new("config", "validation")
            .with_entity_id(file_path)
            .with_metadata("is_valid", serde_json::json!(is_valid))
            .with_metadata("warnings", serde_json::json!(warnings));
        
        let level = if is_valid { UnifiedLogLevel::Info } else { UnifiedLogLevel::Error };
        let message = if is_valid {
            format!("Configuration validation passed ({} warnings)", warnings)
        } else {
            "Configuration validation failed".to_string()
        };
        
        log_with_context(level, context, message, None, None);
    }
}

/// ログ戦略の動的更新
pub fn update_logging_strategy(strategy: LoggingStrategy) {
    if let Ok(mut manager) = STRATEGY_MANAGER.lock() {
        manager.update_strategy(strategy);
        
        let context = LogContext::new("logging", "strategy_update");
        log_with_context(
            UnifiedLogLevel::Info,
            context,
            "Updated logging strategy".to_string(),
            None,
            None,
        );
    }
}

/// ログレベルの動的変更
pub fn set_component_log_level(component: &str, level: UnifiedLogLevel) {
    if let Ok(mut manager) = STRATEGY_MANAGER.lock() {
        let mut strategy = manager.get_strategy().clone();
        strategy.component_levels.insert(component.to_string(), level);
        manager.update_strategy(strategy);
        
        let context = LogContext::new("logging", "level_change")
            .with_metadata("component", serde_json::json!(component))
            .with_metadata("level", serde_json::json!(level.as_str()));
        
        log_with_context(
            UnifiedLogLevel::Info,
            context,
            format!("Changed log level for {} to {}", component, level.as_str()),
            None,
            None,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_creation() {
        let context = LogContext::new("test", "operation");
        let entry = create_log_entry(
            UnifiedLogLevel::Info,
            context,
            "Test message".to_string(),
            None,
            Some(100),
        );
        
        assert_eq!(entry.level, UnifiedLogLevel::Info);
        assert_eq!(entry.context.component, "test");
        assert_eq!(entry.context.operation, "operation");
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.duration_ms, Some(100));
    }

    #[test]
    fn test_human_readable_format() {
        let context = LogContext::new("process", "start")
            .with_entity_id("test-123")
            .with_metadata("cpu", serde_json::json!(50.5));
        
        let entry = UnifiedLogEntry {
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            level: UnifiedLogLevel::Info,
            context,
            message: "Test message".to_string(),
            error: None,
            duration_ms: Some(150),
            location: None,
        };
        
        let formatted = format_human_readable(&entry);
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("process:start"));
        assert!(formatted.contains("test-123"));
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("150ms"));
        assert!(formatted.contains("cpu=50.5"));
    }
}