// WezTerm Multi-Process Development Framework - Log Formatter
// 構造化ログフォーマット定義とカスタムフォーマッター

use super::{UnifiedLogEntry, UnifiedLogLevel};
use serde_json::{json, Value};
use std::collections::HashMap;

/// ログフォーマッター
pub trait LogFormatter {
    fn format(&self, entry: &UnifiedLogEntry) -> String;
}

/// 人間が読みやすい形式のフォーマッター
pub struct HumanReadableFormatter {
    show_timestamp: bool,
    show_level: bool,
    show_component: bool,
    show_location: bool,
    show_metadata: bool,
    color_enabled: bool,
}

impl Default for HumanReadableFormatter {
    fn default() -> Self {
        Self {
            show_timestamp: true,
            show_level: true,
            show_component: true,
            show_location: false,
            show_metadata: true,
            color_enabled: true,
        }
    }
}

impl HumanReadableFormatter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compact() -> Self {
        Self {
            show_timestamp: false,
            show_level: true,
            show_component: true,
            show_location: false,
            show_metadata: false,
            color_enabled: true,
        }
    }

    pub fn verbose() -> Self {
        Self {
            show_timestamp: true,
            show_level: true,
            show_component: true,
            show_location: true,
            show_metadata: true,
            color_enabled: true,
        }
    }

    fn get_level_color(&self, level: UnifiedLogLevel) -> &'static str {
        if !self.color_enabled {
            return "";
        }
        
        match level {
            UnifiedLogLevel::Trace => "\x1b[36m", // Cyan
            UnifiedLogLevel::Debug => "\x1b[34m", // Blue
            UnifiedLogLevel::Info => "\x1b[32m",  // Green
            UnifiedLogLevel::Warn => "\x1b[33m",  // Yellow
            UnifiedLogLevel::Error => "\x1b[31m", // Red
        }
    }

    fn reset_color(&self) -> &'static str {
        if self.color_enabled { "\x1b[0m" } else { "" }
    }
}

impl LogFormatter for HumanReadableFormatter {
    fn format(&self, entry: &UnifiedLogEntry) -> String {
        let mut parts = Vec::new();

        // タイムスタンプ
        if self.show_timestamp {
            let timestamp = entry.timestamp
                .parse::<chrono::DateTime<chrono::Utc>>()
                .map(|dt| dt.format("%H:%M:%S%.3f").to_string())
                .unwrap_or_else(|_| entry.timestamp.clone());
            parts.push(format!("{}", timestamp));
        }

        // ログレベル
        if self.show_level {
            let level_color = self.get_level_color(entry.level);
            let reset = self.reset_color();
            parts.push(format!("{}[{}]{}", level_color, entry.level.as_str(), reset));
        }

        // コンポーネント・操作
        if self.show_component {
            let component_info = if let Some(entity_id) = &entry.context.entity_id {
                format!("[{}:{}:{}]", entry.context.component, entry.context.operation, entity_id)
            } else {
                format!("[{}:{}]", entry.context.component, entry.context.operation)
            };
            parts.push(component_info);
        }

        // セッションID
        if let Some(session_id) = &entry.context.session_id {
            parts.push(format!("({})", session_id));
        }

        // メッセージ
        parts.push(entry.message.clone());

        // 実行時間
        if let Some(duration) = entry.duration_ms {
            let duration_color = if duration > 1000 {
                self.get_level_color(UnifiedLogLevel::Warn)
            } else if duration > 100 {
                self.get_level_color(UnifiedLogLevel::Info)
            } else {
                ""
            };
            parts.push(format!("{}({}ms){}", duration_color, duration, self.reset_color()));
        }

        // エラー情報
        if let Some(error) = &entry.error {
            let error_color = self.get_level_color(UnifiedLogLevel::Error);
            let reset = self.reset_color();
            parts.push(format!("{}ERROR: {}{}", error_color, error, reset));
        }

        // メタデータ
        if self.show_metadata && !entry.context.metadata.is_empty() {
            let metadata_str = entry.context.metadata
                .iter()
                .map(|(k, v)| format!("{}={}", k, format_metadata_value(v)))
                .collect::<Vec<_>>()
                .join(" ");
            parts.push(format!("[{}]", metadata_str));
        }

        // ファイル・行番号
        if self.show_location {
            if let Some(location) = &entry.location {
                parts.push(format!("@{}", location));
            }
        }

        parts.join(" ")
    }
}

/// JSON構造化フォーマッター
pub struct JsonFormatter {
    pretty: bool,
    #[allow(dead_code)]
    include_empty_fields: bool,
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self {
            pretty: false,
            include_empty_fields: false,
        }
    }
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pretty() -> Self {
        Self {
            pretty: true,
            include_empty_fields: false,
        }
    }

    pub fn compact() -> Self {
        Self {
            pretty: false,
            include_empty_fields: false,
        }
    }
}

impl LogFormatter for JsonFormatter {
    fn format(&self, entry: &UnifiedLogEntry) -> String {
        let mut json_obj = json!({
            "timestamp": entry.timestamp,
            "level": entry.level.as_str(),
            "component": entry.context.component,
            "operation": entry.context.operation,
            "message": entry.message
        });

        // オプションフィールドを追加
        if let Some(entity_id) = &entry.context.entity_id {
            json_obj["entity_id"] = json!(entity_id);
        }

        if let Some(session_id) = &entry.context.session_id {
            json_obj["session_id"] = json!(session_id);
        }

        if let Some(error) = &entry.error {
            json_obj["error"] = json!(error);
        }

        if let Some(duration) = entry.duration_ms {
            json_obj["duration_ms"] = json!(duration);
        }

        if let Some(location) = &entry.location {
            json_obj["location"] = json!(location);
        }

        if !entry.context.metadata.is_empty() {
            json_obj["metadata"] = json!(entry.context.metadata);
        }

        if self.pretty {
            serde_json::to_string_pretty(&json_obj)
                .unwrap_or_else(|_| format!("{:?}", entry))
        } else {
            serde_json::to_string(&json_obj)
                .unwrap_or_else(|_| format!("{:?}", entry))
        }
    }
}

/// LogFmt形式のフォーマッター（Heroku/Twelve-Factor App style）
pub struct LogFmtFormatter;

impl LogFormatter for LogFmtFormatter {
    fn format(&self, entry: &UnifiedLogEntry) -> String {
        let mut parts = Vec::new();

        // 基本フィールド
        parts.push(format!("timestamp={}", entry.timestamp));
        parts.push(format!("level={}", entry.level.as_str()));
        parts.push(format!("component={}", entry.context.component));
        parts.push(format!("operation={}", entry.context.operation));
        
        if let Some(entity_id) = &entry.context.entity_id {
            parts.push(format!("entity_id={}", quote_if_needed(entity_id)));
        }

        if let Some(session_id) = &entry.context.session_id {
            parts.push(format!("session_id={}", quote_if_needed(session_id)));
        }

        parts.push(format!("message={}", quote_if_needed(&entry.message)));

        if let Some(duration) = entry.duration_ms {
            parts.push(format!("duration_ms={}", duration));
        }

        if let Some(error) = &entry.error {
            parts.push(format!("error={}", quote_if_needed(error)));
        }

        // メタデータ
        for (key, value) in &entry.context.metadata {
            parts.push(format!("{}={}", key, quote_if_needed(&format_metadata_value(value))));
        }

        if let Some(location) = &entry.location {
            parts.push(format!("location={}", quote_if_needed(location)));
        }

        parts.join(" ")
    }
}

/// カスタムフォーマッター（ユーザー定義）
pub struct CustomFormatter {
    template: String,
    field_extractors: HashMap<String, Box<dyn Fn(&UnifiedLogEntry) -> String + Send + Sync>>,
}

impl CustomFormatter {
    pub fn new(template: &str) -> Self {
        let mut extractors: HashMap<String, Box<dyn Fn(&UnifiedLogEntry) -> String + Send + Sync>> = HashMap::new();
        
        // 基本フィールドのエクストラクター
        extractors.insert("timestamp".to_string(), Box::new(|e| e.timestamp.clone()));
        extractors.insert("level".to_string(), Box::new(|e| e.level.as_str().to_string()));
        extractors.insert("component".to_string(), Box::new(|e| e.context.component.clone()));
        extractors.insert("operation".to_string(), Box::new(|e| e.context.operation.clone()));
        extractors.insert("message".to_string(), Box::new(|e| e.message.clone()));
        extractors.insert("entity_id".to_string(), Box::new(|e| e.context.entity_id.clone().unwrap_or_default()));
        extractors.insert("session_id".to_string(), Box::new(|e| e.context.session_id.clone().unwrap_or_default()));
        extractors.insert("error".to_string(), Box::new(|e| e.error.clone().unwrap_or_default()));
        extractors.insert("duration".to_string(), Box::new(|e| e.duration_ms.map(|d| d.to_string()).unwrap_or_default()));
        extractors.insert("location".to_string(), Box::new(|e| e.location.clone().unwrap_or_default()));

        Self {
            template: template.to_string(),
            field_extractors: extractors,
        }
    }

    /// 使用例のテンプレート
    pub fn simple() -> Self {
        Self::new("{timestamp} [{level}] {component}: {message}")
    }

    pub fn detailed() -> Self {
        Self::new("{timestamp} [{level}] [{component}:{operation}] {entity_id} {message} {duration}ms")
    }
}

impl LogFormatter for CustomFormatter {
    fn format(&self, entry: &UnifiedLogEntry) -> String {
        let mut result = self.template.clone();
        
        for (field, extractor) in &self.field_extractors {
            let placeholder = format!("{{{}}}", field);
            if result.contains(&placeholder) {
                let value = extractor(entry);
                result = result.replace(&placeholder, &value);
            }
        }
        
        result
    }
}

// ヘルパー関数

fn format_metadata_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}

fn quote_if_needed(s: &str) -> String {
    if s.contains(' ') || s.contains('=') || s.contains('"') {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

/// フォーマッター選択
pub enum FormatterType {
    HumanReadable(HumanReadableFormatter),
    Json(JsonFormatter),
    LogFmt(LogFmtFormatter),
    Custom(CustomFormatter),
}

impl FormatterType {
    pub fn format(&self, entry: &UnifiedLogEntry) -> String {
        match self {
            FormatterType::HumanReadable(f) => f.format(entry),
            FormatterType::Json(f) => f.format(entry),
            FormatterType::LogFmt(f) => f.format(entry),
            FormatterType::Custom(f) => f.format(entry),
        }
    }

    /// 環境変数から選択
    pub fn from_environment() -> Self {
        match std::env::var("WEZTERM_LOG_FORMAT").as_deref() {
            Ok("json") => FormatterType::Json(JsonFormatter::new()),
            Ok("json-pretty") => FormatterType::Json(JsonFormatter::pretty()),
            Ok("logfmt") => FormatterType::LogFmt(LogFmtFormatter),
            Ok("compact") => FormatterType::HumanReadable(HumanReadableFormatter::compact()),
            Ok("verbose") => FormatterType::HumanReadable(HumanReadableFormatter::verbose()),
            _ => FormatterType::HumanReadable(HumanReadableFormatter::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::{LogContext, UnifiedLogLevel};

    fn create_test_entry() -> UnifiedLogEntry {
        let context = LogContext::new("test", "operation")
            .with_entity_id("test-123")
            .with_session_id("session-456")
            .with_metadata("cpu", serde_json::json!(75.5))
            .with_metadata("memory", serde_json::json!("512MB"));

        UnifiedLogEntry {
            timestamp: "2025-01-01T12:00:00.123Z".to_string(),
            level: UnifiedLogLevel::Info,
            context,
            message: "Test message with spaces".to_string(),
            error: Some("Test error".to_string()),
            duration_ms: Some(150),
            location: Some("test.rs:42".to_string()),
        }
    }

    #[test]
    fn test_human_readable_formatter() {
        let formatter = HumanReadableFormatter::new();
        let entry = create_test_entry();
        let formatted = formatter.format(&entry);
        
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("test:operation"));
        assert!(formatted.contains("test-123"));
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("150ms"));
        assert!(formatted.contains("ERROR: Test error"));
    }

    #[test]
    fn test_json_formatter() {
        let formatter = JsonFormatter::new();
        let entry = create_test_entry();
        let formatted = formatter.format(&entry);
        
        let parsed: Value = serde_json::from_str(&formatted).unwrap();
        assert_eq!(parsed["level"], "INFO");
        assert_eq!(parsed["component"], "test");
        assert_eq!(parsed["operation"], "operation");
        assert_eq!(parsed["entity_id"], "test-123");
        assert_eq!(parsed["duration_ms"], 150);
    }

    #[test]
    fn test_logfmt_formatter() {
        let formatter = LogFmtFormatter;
        let entry = create_test_entry();
        let formatted = formatter.format(&entry);
        
        assert!(formatted.contains("level=INFO"));
        assert!(formatted.contains("component=test"));
        assert!(formatted.contains("operation=operation"));
        assert!(formatted.contains("entity_id=test-123"));
        assert!(formatted.contains("message=\"Test message with spaces\""));
        assert!(formatted.contains("duration_ms=150"));
    }

    #[test]
    fn test_custom_formatter() {
        let formatter = CustomFormatter::new("{level} - {component}: {message}");
        let entry = create_test_entry();
        let formatted = formatter.format(&entry);
        
        assert_eq!(formatted, "INFO - test: Test message with spaces");
    }
}