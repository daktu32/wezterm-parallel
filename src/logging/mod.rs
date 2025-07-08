// WezTerm Multi-Process Development Framework - Unified Logging System
// 統一されたログシステム - デバッグ効率化とトラブルシューティング強化

pub mod enhancer;
pub mod formatter;
pub mod strategy;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 統一ログレベル定義
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum UnifiedLogLevel {
    /// 極詳細なトレース情報 (関数呼び出し、詳細な実行フロー)
    Trace = 1,
    /// デバッグ情報 (変数値、中間状態、内部動作)
    Debug = 2,
    /// 一般的な情報 (起動・停止、重要な状態変化)
    Info = 3,
    /// 警告 (潜在的な問題、回復可能なエラー)
    Warn = 4,
    /// エラー (処理失敗、ユーザー対応が必要)
    Error = 5,
}

/// ログコンテキスト - 構造化ログの基盤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogContext {
    /// コンポーネント名 (process, room, config, ipc, etc.)
    pub component: String,
    /// 操作名 (start, stop, create, delete, send, receive, etc.)
    pub operation: String,
    /// エンティティID (process_id, room_name, config_file, etc.)
    pub entity_id: Option<String>,
    /// ユーザーID (将来の拡張用)
    pub user_id: Option<String>,
    /// セッションID (操作の追跡用)
    pub session_id: Option<String>,
    /// 追加フィールド
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 統一ログエントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedLogEntry {
    /// タイムスタンプ (ISO 8601)
    pub timestamp: String,
    /// ログレベル
    pub level: UnifiedLogLevel,
    /// ログコンテキスト
    pub context: LogContext,
    /// メッセージ
    pub message: String,
    /// エラー情報 (該当する場合)
    pub error: Option<String>,
    /// パフォーマンス情報
    pub duration_ms: Option<u64>,
    /// ファイル・行番号 (デバッグ用)
    pub location: Option<String>,
}

impl UnifiedLogLevel {
    /// ログレベルを文字列に変換
    pub fn as_str(&self) -> &'static str {
        match self {
            UnifiedLogLevel::Trace => "TRACE",
            UnifiedLogLevel::Debug => "DEBUG",
            UnifiedLogLevel::Info => "INFO",
            UnifiedLogLevel::Warn => "WARN",
            UnifiedLogLevel::Error => "ERROR",
        }
    }

    /// 文字列からログレベルを変換
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TRACE" => Some(UnifiedLogLevel::Trace),
            "DEBUG" => Some(UnifiedLogLevel::Debug),
            "INFO" => Some(UnifiedLogLevel::Info),
            "WARN" => Some(UnifiedLogLevel::Warn),
            "ERROR" => Some(UnifiedLogLevel::Error),
            _ => None,
        }
    }
}

impl LogContext {
    /// 新しいログコンテキストを作成
    pub fn new(component: &str, operation: &str) -> Self {
        Self {
            component: component.to_string(),
            operation: operation.to_string(),
            entity_id: None,
            user_id: None,
            session_id: None,
            metadata: HashMap::new(),
        }
    }

    /// エンティティIDを設定
    pub fn with_entity_id(mut self, entity_id: &str) -> Self {
        self.entity_id = Some(entity_id.to_string());
        self
    }

    /// セッションIDを設定
    pub fn with_session_id(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    /// メタデータを追加
    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}

/// 統一ログマクロ定義
#[macro_export]
macro_rules! log_trace {
    ($context:expr, $msg:expr) => {
        $crate::logging::enhancer::log_with_context(
            $crate::logging::UnifiedLogLevel::Trace,
            $context,
            $msg.to_string(),
            None,
            None,
        );
    };
    ($context:expr, $msg:expr, $($arg:tt)*) => {
        $crate::logging::enhancer::log_with_context(
            $crate::logging::UnifiedLogLevel::Trace,
            $context,
            format!($msg, $($arg)*),
            None,
            None,
        );
    };
}

#[macro_export]
macro_rules! log_debug {
    ($context:expr, $msg:expr) => {
        $crate::logging::enhancer::log_with_context(
            $crate::logging::UnifiedLogLevel::Debug,
            $context,
            $msg.to_string(),
            None,
            None,
        );
    };
    ($context:expr, $msg:expr, $($arg:tt)*) => {
        $crate::logging::enhancer::log_with_context(
            $crate::logging::UnifiedLogLevel::Debug,
            $context,
            format!($msg, $($arg)*),
            None,
            None,
        );
    };
}

#[macro_export]
macro_rules! log_info {
    ($context:expr, $msg:expr) => {
        $crate::logging::enhancer::log_with_context(
            $crate::logging::UnifiedLogLevel::Info,
            $context,
            $msg.to_string(),
            None,
            None,
        );
    };
    ($context:expr, $msg:expr, $($arg:tt)*) => {
        $crate::logging::enhancer::log_with_context(
            $crate::logging::UnifiedLogLevel::Info,
            $context,
            format!($msg, $($arg)*),
            None,
            None,
        );
    };
}

#[macro_export]
macro_rules! log_warn {
    ($context:expr, $msg:expr) => {
        $crate::logging::enhancer::log_with_context(
            $crate::logging::UnifiedLogLevel::Warn,
            $context,
            $msg.to_string(),
            None,
            None,
        );
    };
    ($context:expr, $msg:expr, $($arg:tt)*) => {
        $crate::logging::enhancer::log_with_context(
            $crate::logging::UnifiedLogLevel::Warn,
            $context,
            format!($msg, $($arg)*),
            None,
            None,
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($context:expr, $msg:expr) => {
        $crate::logging::enhancer::log_with_context(
            $crate::logging::UnifiedLogLevel::Error,
            $context,
            $msg.to_string(),
            None,
            None,
        );
    };
    ($context:expr, $msg:expr, $($arg:tt)*) => {
        $crate::logging::enhancer::log_with_context(
            $crate::logging::UnifiedLogLevel::Error,
            $context,
            format!($msg, $($arg)*),
            None,
            None,
        );
    };
    ($context:expr, $msg:expr, $error:expr) => {
        $crate::logging::enhancer::log_with_context(
            $crate::logging::UnifiedLogLevel::Error,
            $context,
            $msg.to_string(),
            Some($error.to_string()),
            None,
        );
    };
}

/// パフォーマンス測定付きログマクロ
#[macro_export]
macro_rules! log_with_duration {
    ($level:expr, $context:expr, $msg:expr, $duration:expr) => {
        $crate::logging::enhancer::log_with_context(
            $level,
            $context,
            $msg.to_string(),
            None,
            Some($duration.as_millis() as u64),
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_log_level_ordering() {
        assert!(UnifiedLogLevel::Trace < UnifiedLogLevel::Debug);
        assert!(UnifiedLogLevel::Debug < UnifiedLogLevel::Info);
        assert!(UnifiedLogLevel::Info < UnifiedLogLevel::Warn);
        assert!(UnifiedLogLevel::Warn < UnifiedLogLevel::Error);
    }

    #[test]
    fn test_log_level_conversion() {
        assert_eq!(UnifiedLogLevel::Info.as_str(), "INFO");
        assert_eq!(
            UnifiedLogLevel::from_str("DEBUG"),
            Some(UnifiedLogLevel::Debug)
        );
        assert_eq!(UnifiedLogLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_log_context_creation() {
        let context = LogContext::new("process", "start")
            .with_entity_id("claude-001")
            .with_session_id("session-123")
            .with_metadata("cpu_usage", serde_json::json!(75.5));

        assert_eq!(context.component, "process");
        assert_eq!(context.operation, "start");
        assert_eq!(context.entity_id, Some("claude-001".to_string()));
        assert_eq!(context.session_id, Some("session-123".to_string()));
        assert!(context.metadata.contains_key("cpu_usage"));
    }
}
