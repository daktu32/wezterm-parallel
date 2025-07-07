// WezTerm Multi-Process Development Framework - Logging Strategy
// ログ戦略定義とコンポーネント別ログレベル管理

use super::{UnifiedLogLevel, LogContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// コンポーネント別ログ戦略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingStrategy {
    /// グローバルデフォルトレベル
    pub default_level: UnifiedLogLevel,
    /// コンポーネント別レベル設定
    pub component_levels: HashMap<String, UnifiedLogLevel>,
    /// 高頻度ログの制限設定
    pub rate_limits: HashMap<String, RateLimit>,
    /// パフォーマンス監視対象操作
    pub performance_targets: Vec<String>,
    /// 構造化ログの有効化
    pub structured_output: bool,
    /// 出力先設定
    pub outputs: Vec<LogOutput>,
}

/// レート制限設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// 制限対象のコンポーネント・操作
    pub target: String,
    /// 時間窓（秒）
    pub window_seconds: u64,
    /// 最大ログ数
    pub max_logs: u32,
    /// 制限時のサンプリング率 (0.0-1.0)
    pub sampling_rate: f64,
}

/// ログ出力先
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    /// 標準出力
    Stdout,
    /// 標準エラー出力
    Stderr,
    /// ファイル出力
    File {
        path: String,
        max_size_mb: u64,
        max_files: u32,
    },
    /// システムログ (syslog/journald)
    System,
    /// 構造化ログファイル (JSON)
    StructuredFile {
        path: String,
        max_size_mb: u64,
        max_files: u32,
    },
}

impl Default for LoggingStrategy {
    fn default() -> Self {
        let mut component_levels = HashMap::new();
        
        // コンポーネント別ログレベル戦略
        component_levels.insert("process".to_string(), UnifiedLogLevel::Info);    // プロセス管理
        component_levels.insert("room".to_string(), UnifiedLogLevel::Info);       // Room管理
        component_levels.insert("config".to_string(), UnifiedLogLevel::Info);     // 設定管理
        component_levels.insert("ipc".to_string(), UnifiedLogLevel::Debug);       // IPC通信
        component_levels.insert("sync".to_string(), UnifiedLogLevel::Debug);      // ファイル同期
        component_levels.insert("monitoring".to_string(), UnifiedLogLevel::Warn); // 監視システム
        component_levels.insert("performance".to_string(), UnifiedLogLevel::Info); // パフォーマンス
        component_levels.insert("error".to_string(), UnifiedLogLevel::Error);     // エラー処理
        component_levels.insert("task".to_string(), UnifiedLogLevel::Info);       // タスク管理
        component_levels.insert("dashboard".to_string(), UnifiedLogLevel::Warn);  // ダッシュボード

        let mut rate_limits = HashMap::new();
        
        // 高頻度操作のレート制限
        rate_limits.insert("heartbeat".to_string(), RateLimit {
            target: "process.heartbeat".to_string(),
            window_seconds: 60,
            max_logs: 5,
            sampling_rate: 0.1,
        });
        
        rate_limits.insert("file_watch".to_string(), RateLimit {
            target: "sync.file_watch".to_string(),
            window_seconds: 30,
            max_logs: 10,
            sampling_rate: 0.2,
        });

        Self {
            default_level: UnifiedLogLevel::Info,
            component_levels,
            rate_limits,
            performance_targets: vec![
                "process.start".to_string(),
                "process.stop".to_string(),
                "room.create".to_string(),
                "room.switch".to_string(),
                "config.load".to_string(),
                "sync.apply_change".to_string(),
                "task.execute".to_string(),
            ],
            structured_output: true,
            outputs: vec![
                LogOutput::Stdout,
                LogOutput::StructuredFile {
                    path: "logs/wezterm-parallel.json".to_string(),
                    max_size_mb: 100,
                    max_files: 10,
                },
            ],
        }
    }
}

impl LoggingStrategy {
    /// 特定のコンポーネント・操作のログレベルを取得
    pub fn get_log_level(&self, component: &str) -> UnifiedLogLevel {
        self.component_levels
            .get(component)
            .copied()
            .unwrap_or(self.default_level)
    }

    /// ログエントリがレート制限に引っかかるかチェック
    pub fn should_rate_limit(&self, context: &LogContext) -> bool {
        let target = format!("{}.{}", context.component, context.operation);
        
        for (_, limit) in &self.rate_limits {
            if limit.target == target || 
               limit.target == context.component || 
               limit.target == context.operation {
                // 実際のレート制限チェックはここで実装
                // 現在は簡略化のため常にfalseを返す
                return false;
            }
        }
        false
    }

    /// パフォーマンス測定対象かどうか
    pub fn should_measure_performance(&self, context: &LogContext) -> bool {
        let target = format!("{}.{}", context.component, context.operation);
        self.performance_targets.contains(&target)
    }

    /// 開発環境用設定
    pub fn development() -> Self {
        let mut strategy = Self::default();
        strategy.default_level = UnifiedLogLevel::Debug;
        
        // 開発時はより詳細なログを出力
        strategy.component_levels.insert("process".to_string(), UnifiedLogLevel::Debug);
        strategy.component_levels.insert("room".to_string(), UnifiedLogLevel::Debug);
        strategy.component_levels.insert("config".to_string(), UnifiedLogLevel::Debug);
        strategy.component_levels.insert("ipc".to_string(), UnifiedLogLevel::Trace);
        
        strategy.outputs = vec![
            LogOutput::Stdout,
            LogOutput::File {
                path: "logs/dev.log".to_string(),
                max_size_mb: 50,
                max_files: 5,
            },
            LogOutput::StructuredFile {
                path: "logs/dev-structured.json".to_string(),
                max_size_mb: 50,
                max_files: 5,
            },
        ];
        
        strategy
    }

    /// プロダクション環境用設定
    pub fn production() -> Self {
        let mut strategy = Self::default();
        strategy.default_level = UnifiedLogLevel::Warn;
        
        // プロダクションでは重要なログのみ
        strategy.component_levels.insert("process".to_string(), UnifiedLogLevel::Info);
        strategy.component_levels.insert("room".to_string(), UnifiedLogLevel::Info);
        strategy.component_levels.insert("config".to_string(), UnifiedLogLevel::Warn);
        strategy.component_levels.insert("error".to_string(), UnifiedLogLevel::Error);
        
        strategy.outputs = vec![
            LogOutput::System,
            LogOutput::StructuredFile {
                path: "/var/log/wezterm-parallel/app.json".to_string(),
                max_size_mb: 200,
                max_files: 20,
            },
        ];
        
        strategy
    }

    /// デバッグ専用設定
    pub fn debug() -> Self {
        let mut strategy = Self::default();
        strategy.default_level = UnifiedLogLevel::Trace;
        
        // すべてのコンポーネントでTRACEレベル
        for component in ["process", "room", "config", "ipc", "sync", "monitoring", "task"] {
            strategy.component_levels.insert(component.to_string(), UnifiedLogLevel::Trace);
        }
        
        // レート制限を緩和
        strategy.rate_limits.clear();
        
        // パフォーマンス測定を全操作で有効化
        strategy.performance_targets = vec![
            "process.*".to_string(),
            "room.*".to_string(),
            "config.*".to_string(),
            "ipc.*".to_string(),
            "sync.*".to_string(),
            "task.*".to_string(),
        ];
        
        strategy.outputs = vec![
            LogOutput::Stdout,
            LogOutput::StructuredFile {
                path: "logs/debug-trace.json".to_string(),
                max_size_mb: 500,
                max_files: 3,
            },
        ];
        
        strategy
    }
}

/// ログ戦略の管理
pub struct StrategyManager {
    current_strategy: LoggingStrategy,
}

impl StrategyManager {
    pub fn new(strategy: LoggingStrategy) -> Self {
        Self {
            current_strategy: strategy,
        }
    }

    pub fn get_strategy(&self) -> &LoggingStrategy {
        &self.current_strategy
    }

    pub fn update_strategy(&mut self, strategy: LoggingStrategy) {
        self.current_strategy = strategy;
    }

    /// 環境変数から戦略を選択
    pub fn from_environment() -> Self {
        let strategy = match std::env::var("WEZTERM_LOG_MODE").as_deref() {
            Ok("development") | Ok("dev") => LoggingStrategy::development(),
            Ok("production") | Ok("prod") => LoggingStrategy::production(),
            Ok("debug") => LoggingStrategy::debug(),
            _ => LoggingStrategy::default(),
        };
        
        Self::new(strategy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_strategy() {
        let strategy = LoggingStrategy::default();
        assert_eq!(strategy.default_level, UnifiedLogLevel::Info);
        assert_eq!(strategy.get_log_level("process"), UnifiedLogLevel::Info);
        assert_eq!(strategy.get_log_level("unknown"), UnifiedLogLevel::Info);
    }

    #[test]
    fn test_development_strategy() {
        let strategy = LoggingStrategy::development();
        assert_eq!(strategy.default_level, UnifiedLogLevel::Debug);
        assert_eq!(strategy.get_log_level("ipc"), UnifiedLogLevel::Trace);
    }

    #[test]
    fn test_production_strategy() {
        let strategy = LoggingStrategy::production();
        assert_eq!(strategy.default_level, UnifiedLogLevel::Warn);
        assert_eq!(strategy.get_log_level("error"), UnifiedLogLevel::Error);
    }

    #[test]
    fn test_performance_measurement() {
        let strategy = LoggingStrategy::default();
        let context = LogContext::new("process", "start");
        assert!(strategy.should_measure_performance(&context));
        
        let context2 = LogContext::new("process", "heartbeat");
        assert!(!strategy.should_measure_performance(&context2));
    }
}