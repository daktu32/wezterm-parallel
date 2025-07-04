use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::time::SystemTime;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration};
use tokio::task::JoinHandle;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use log::{debug, info, warn, error};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Claude Code専用のログ管理システム
#[derive(Debug)]
pub struct ClaudeLogger {
    /// ログ設定
    config: LogConfig,
    /// アクティブなログストリーム
    active_streams: RwLock<HashMap<String, LogStream>>,
    /// ログエントリ受信チャネル
    log_receiver: Option<mpsc::UnboundedReceiver<LogEntry>>,
    /// ログエントリ送信チャネル
    log_sender: mpsc::UnboundedSender<LogEntry>,
    /// ログ処理タスク
    processing_handle: Option<JoinHandle<()>>,
    /// ログローテーションタスク
    rotation_handle: Option<JoinHandle<()>>,
}

/// ログ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// ベースログディレクトリ
    pub base_dir: PathBuf,
    /// 最大ログファイルサイズ（MB）
    pub max_file_size_mb: u64,
    /// 保持するログファイル数
    pub max_files: u32,
    /// ログレベル
    pub log_level: LogLevel,
    /// ログフォーマット
    pub format: LogFormat,
    /// バッファサイズ
    pub buffer_size: usize,
    /// ローテーション間隔（時間）
    pub rotation_interval_hours: u64,
    /// 構造化ログの有効化
    pub enable_structured_logs: bool,
    /// デバッグ情報の有効化
    pub enable_debug_info: bool,
}

/// ログレベル
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// ログフォーマット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Plain,
    Json,
    Structured,
}

/// ログストリーム（プロセス固有）
#[derive(Debug)]
pub struct LogStream {
    pub process_id: String,
    pub workspace: String,
    pub file_path: PathBuf,
    pub writer: BufWriter<File>,
    pub entry_count: u64,
    pub current_size_bytes: u64,
    pub created_at: SystemTime,
    pub last_written: SystemTime,
}

/// ログエントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub process_id: String,
    pub workspace: String,
    pub level: LogLevel,
    pub source: LogSource,
    pub message: String,
    pub metadata: HashMap<String, String>,
    pub raw_output: Option<String>,
}

/// ログソース
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogSource {
    Stdout,
    Stderr,
    Internal,
    Debug,
    Health,
}

/// デバッグ情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    pub process_id: String,
    pub workspace: String,
    pub timestamp: DateTime<Utc>,
    pub debug_type: DebugType,
    pub data: HashMap<String, serde_json::Value>,
}

/// デバッグタイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugType {
    ProcessState,
    MemoryUsage,
    PerformanceMetrics,
    ErrorDiagnostics,
    CommandExecution,
    ResponseTime,
}

impl Default for LogConfig {
    fn default() -> Self {
        let mut base_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
        base_dir.push("wezterm-parallel");
        base_dir.push("logs");

        Self {
            base_dir,
            max_file_size_mb: 100,
            max_files: 10,
            log_level: LogLevel::Info,
            format: LogFormat::Structured,
            buffer_size: 8192,
            rotation_interval_hours: 24,
            enable_structured_logs: true,
            enable_debug_info: true,
        }
    }
}

impl ClaudeLogger {
    /// 新しいClaudeLoggerを作成
    pub fn new(config: Option<LogConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();
        
        // ログディレクトリを作成
        fs::create_dir_all(&config.base_dir)?;
        
        let (log_sender, log_receiver) = mpsc::unbounded_channel();
        
        Ok(Self {
            config,
            active_streams: RwLock::new(HashMap::new()),
            log_receiver: Some(log_receiver),
            log_sender,
            processing_handle: None,
            rotation_handle: None,
        })
    }

    /// ログ処理を開始
    pub async fn start(&mut self) -> Result<()> {
        // ログ処理タスクを開始
        if let Some(receiver) = self.log_receiver.take() {
            let logger = self.clone_for_task().await?;
            let processing_handle = tokio::spawn(async move {
                logger.process_log_entries(receiver).await;
            });
            self.processing_handle = Some(processing_handle);
        }

        // ログローテーションタスクを開始
        let logger = self.clone_for_task().await?;
        let rotation_interval = Duration::from_secs(self.config.rotation_interval_hours * 3600);
        let rotation_handle = tokio::spawn(async move {
            let mut interval = interval(rotation_interval);
            loop {
                interval.tick().await;
                if let Err(e) = logger.rotate_logs().await {
                    error!("Log rotation failed: {}", e);
                }
            }
        });
        self.rotation_handle = Some(rotation_handle);

        info!("Claude Logger started with config: {:?}", self.config);
        Ok(())
    }

    /// ログ処理を停止
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.processing_handle.take() {
            handle.abort();
        }
        
        if let Some(handle) = self.rotation_handle.take() {
            handle.abort();
        }

        // 残りのログをフラッシュ
        self.flush_all_streams().await?;
        
        info!("Claude Logger stopped");
        Ok(())
    }

    /// プロセスのログストリームを開始
    pub async fn start_logging_process(&self, process_id: String, workspace: String) -> Result<()> {
        let log_file_path = self.get_log_file_path(&process_id, &workspace);
        
        // ディレクトリを作成
        if let Some(parent) = log_file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // ファイルを開く
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)?;
        
        let writer = BufWriter::with_capacity(self.config.buffer_size, file);
        
        let stream = LogStream {
            process_id: process_id.clone(),
            workspace: workspace.clone(),
            file_path: log_file_path,
            writer,
            entry_count: 0,
            current_size_bytes: 0,
            created_at: SystemTime::now(),
            last_written: SystemTime::now(),
        };

        {
            let mut streams = self.active_streams.write().await;
            streams.insert(process_id.clone(), stream);
        }

        info!("Started logging for process '{}' in workspace '{}'", process_id, workspace);
        Ok(())
    }

    /// プロセスのログストリームを停止
    pub async fn stop_logging_process(&self, process_id: &str) -> Result<()> {
        let mut streams = self.active_streams.write().await;
        if let Some(mut stream) = streams.remove(process_id) {
            stream.writer.flush()?;
            info!("Stopped logging for process '{}'", process_id);
        }
        Ok(())
    }

    /// ログエントリを送信
    pub fn log(&self, entry: LogEntry) -> Result<()> {
        self.log_sender.send(entry)
            .map_err(|e| format!("Failed to send log entry: {}", e).into())
    }

    /// Claude Code出力をログ
    pub fn log_claude_output(&self, process_id: String, workspace: String, line: String, is_stderr: bool) -> Result<()> {
        let entry = LogEntry {
            timestamp: Utc::now(),
            process_id,
            workspace,
            level: if is_stderr { LogLevel::Error } else { LogLevel::Info },
            source: if is_stderr { LogSource::Stderr } else { LogSource::Stdout },
            message: line.clone(),
            metadata: HashMap::new(),
            raw_output: Some(line),
        };
        
        self.log(entry)
    }

    /// デバッグ情報をログ
    pub fn log_debug_info(&self, debug_info: DebugInfo) -> Result<()> {
        if !self.config.enable_debug_info {
            return Ok(());
        }

        let entry = LogEntry {
            timestamp: debug_info.timestamp,
            process_id: debug_info.process_id,
            workspace: debug_info.workspace,
            level: LogLevel::Debug,
            source: LogSource::Debug,
            message: format!("Debug: {:?}", debug_info.debug_type),
            metadata: debug_info.data.iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect(),
            raw_output: None,
        };

        self.log(entry)
    }

    /// ヘルス情報をログ
    pub fn log_health_info(&self, process_id: String, workspace: String, health_data: HashMap<String, String>) -> Result<()> {
        let entry = LogEntry {
            timestamp: Utc::now(),
            process_id,
            workspace,
            level: LogLevel::Info,
            source: LogSource::Health,
            message: "Health check data".to_string(),
            metadata: health_data,
            raw_output: None,
        };

        self.log(entry)
    }

    /// ログファイルパスを取得
    fn get_log_file_path(&self, process_id: &str, workspace: &str) -> PathBuf {
        let mut path = self.config.base_dir.clone();
        path.push(workspace);
        path.push(format!("{}.log", process_id));
        path
    }

    /// タスク用のCloneを作成
    async fn clone_for_task(&self) -> Result<Self> {
        let config = self.config.clone();
        let active_streams = RwLock::new(HashMap::new());
        let (log_sender, _) = mpsc::unbounded_channel();
        
        Ok(Self {
            config,
            active_streams,
            log_receiver: None,
            log_sender,
            processing_handle: None,
            rotation_handle: None,
        })
    }

    /// ログエントリを処理
    async fn process_log_entries(&self, mut receiver: mpsc::UnboundedReceiver<LogEntry>) {
        while let Some(entry) = receiver.recv().await {
            if let Err(e) = self.write_log_entry(&entry).await {
                error!("Failed to write log entry: {}", e);
            }
        }
    }

    /// ログエントリを書き込み
    async fn write_log_entry(&self, entry: &LogEntry) -> Result<()> {
        // ログレベルフィルタリング
        if entry.level < self.config.log_level {
            return Ok(());
        }

        let formatted = self.format_log_entry(entry)?;
        
        {
            let mut streams = self.active_streams.write().await;
            if let Some(stream) = streams.get_mut(&entry.process_id) {
                stream.writer.write_all(formatted.as_bytes())?;
                stream.writer.write_all(b"\n")?;
                stream.entry_count += 1;
                stream.current_size_bytes += formatted.len() as u64 + 1;
                stream.last_written = SystemTime::now();

                // ファイルサイズチェック
                if stream.current_size_bytes > self.config.max_file_size_mb * 1024 * 1024 {
                    self.rotate_stream(&entry.process_id).await?;
                }
            }
        }

        Ok(())
    }

    /// ログエントリをフォーマット
    fn format_log_entry(&self, entry: &LogEntry) -> Result<String> {
        match self.config.format {
            LogFormat::Plain => {
                Ok(format!(
                    "[{}] [{}] [{}] [{}] {}",
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                    self.level_to_string(&entry.level),
                    entry.process_id,
                    self.source_to_string(&entry.source),
                    entry.message
                ))
            }
            LogFormat::Json => {
                serde_json::to_string(entry)
                    .map_err(|e| format!("JSON serialization failed: {}", e).into())
            }
            LogFormat::Structured => {
                let mut output = format!(
                    "[{}] [{}] [{}:{}] [{}] {}",
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                    self.level_to_string(&entry.level),
                    entry.workspace,
                    entry.process_id,
                    self.source_to_string(&entry.source),
                    entry.message
                );

                if !entry.metadata.is_empty() {
                    output.push_str(" | ");
                    for (key, value) in &entry.metadata {
                        output.push_str(&format!("{}={} ", key, value));
                    }
                }

                Ok(output)
            }
        }
    }

    /// ログレベルを文字列に変換
    fn level_to_string(&self, level: &LogLevel) -> &str {
        match level {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    /// ログソースを文字列に変換
    fn source_to_string(&self, source: &LogSource) -> &str {
        match source {
            LogSource::Stdout => "STDOUT",
            LogSource::Stderr => "STDERR",
            LogSource::Internal => "INTERNAL",
            LogSource::Debug => "DEBUG",
            LogSource::Health => "HEALTH",
        }
    }

    /// 特定のストリームをローテート
    async fn rotate_stream(&self, process_id: &str) -> Result<()> {
        let mut streams = self.active_streams.write().await;
        if let Some(stream) = streams.get_mut(process_id) {
            // 現在のファイルをフラッシュ
            stream.writer.flush()?;

            // ローテートファイル名を生成
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
            let rotated_path = stream.file_path.with_extension(format!("log.{}", timestamp));

            // ファイルをリネーム
            fs::rename(&stream.file_path, &rotated_path)?;

            // 新しいファイルを開く
            let new_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&stream.file_path)?;
            
            stream.writer = BufWriter::with_capacity(self.config.buffer_size, new_file);
            stream.current_size_bytes = 0;
            stream.entry_count = 0;

            info!("Rotated log file for process '{}': {:?}", process_id, rotated_path);

            // 古いファイルを削除
            self.cleanup_old_files(&stream.file_path).await?;
        }
        Ok(())
    }

    /// すべてのログをローテート
    async fn rotate_logs(&self) -> Result<()> {
        let process_ids: Vec<String> = {
            let streams = self.active_streams.read().await;
            streams.keys().cloned().collect()
        };

        for process_id in process_ids {
            if let Err(e) = self.rotate_stream(&process_id).await {
                error!("Failed to rotate log for process '{}': {}", process_id, e);
            }
        }

        Ok(())
    }

    /// 古いログファイルを削除
    async fn cleanup_old_files(&self, log_file_path: &Path) -> Result<()> {
        let dir = log_file_path.parent().unwrap();
        let file_stem = log_file_path.file_stem().unwrap().to_string_lossy();
        
        let mut log_files = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();
                if name_str.starts_with(&*file_stem) && name_str.contains(".log.") {
                    log_files.push((path, entry.metadata()?.modified()?));
                }
            }
        }

        // 作成時間でソート
        log_files.sort_by(|a, b| b.1.cmp(&a.1));

        // 保持数を超えたファイルを削除
        if log_files.len() > self.config.max_files as usize {
            for (path, _) in log_files.iter().skip(self.config.max_files as usize) {
                if let Err(e) = fs::remove_file(path) {
                    warn!("Failed to remove old log file {:?}: {}", path, e);
                } else {
                    debug!("Removed old log file: {:?}", path);
                }
            }
        }

        Ok(())
    }

    /// すべてのストリームをフラッシュ
    async fn flush_all_streams(&self) -> Result<()> {
        let mut streams = self.active_streams.write().await;
        for stream in streams.values_mut() {
            stream.writer.flush()?;
        }
        Ok(())
    }

    /// ログ統計を取得
    pub async fn get_log_statistics(&self) -> HashMap<String, LogStatistics> {
        let streams = self.active_streams.read().await;
        streams.iter()
            .map(|(process_id, stream)| {
                (process_id.clone(), LogStatistics {
                    process_id: stream.process_id.clone(),
                    workspace: stream.workspace.clone(),
                    entry_count: stream.entry_count,
                    file_size_bytes: stream.current_size_bytes,
                    created_at: stream.created_at,
                    last_written: stream.last_written,
                })
            })
            .collect()
    }
}

/// ログ統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStatistics {
    pub process_id: String,
    pub workspace: String,
    pub entry_count: u64,
    pub file_size_bytes: u64,
    pub created_at: SystemTime,
    pub last_written: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_logger() -> (ClaudeLogger, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = LogConfig {
            base_dir: temp_dir.path().to_path_buf(),
            max_file_size_mb: 1,
            max_files: 3,
            ..Default::default()
        };
        
        let logger = ClaudeLogger::new(Some(config)).unwrap();
        (logger, temp_dir)
    }

    #[tokio::test]
    async fn test_claude_logger_creation() {
        let (logger, _temp_dir) = create_test_logger().await;
        assert_eq!(logger.config.max_files, 3);
        assert_eq!(logger.config.max_file_size_mb, 1);
    }

    #[tokio::test]
    async fn test_start_logging_process() {
        let (logger, _temp_dir) = create_test_logger().await;
        
        let result = logger.start_logging_process(
            "test-process".to_string(),
            "test-workspace".to_string()
        ).await;
        
        assert!(result.is_ok());
        
        let streams = logger.active_streams.read().await;
        assert!(streams.contains_key("test-process"));
    }

    #[tokio::test]
    async fn test_log_entry_formatting() {
        let (logger, _temp_dir) = create_test_logger().await;
        
        let entry = LogEntry {
            timestamp: Utc::now(),
            process_id: "test-process".to_string(),
            workspace: "test-workspace".to_string(),
            level: LogLevel::Info,
            source: LogSource::Stdout,
            message: "Test message".to_string(),
            metadata: HashMap::new(),
            raw_output: None,
        };

        let formatted = logger.format_log_entry(&entry).unwrap();
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("test-process"));
        assert!(formatted.contains("Test message"));
    }

    #[tokio::test]
    async fn test_log_claude_output() {
        let (logger, _temp_dir) = create_test_logger().await;
        
        logger.start_logging_process(
            "test-process".to_string(),
            "test-workspace".to_string()
        ).await.unwrap();

        let result = logger.log_claude_output(
            "test-process".to_string(),
            "test-workspace".to_string(),
            "Claude output line".to_string(),
            false
        );

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_log_statistics() {
        let (logger, _temp_dir) = create_test_logger().await;
        
        logger.start_logging_process(
            "test-process".to_string(),
            "test-workspace".to_string()
        ).await.unwrap();

        let stats = logger.get_log_statistics().await;
        assert!(stats.contains_key("test-process"));
        
        let process_stats = &stats["test-process"];
        assert_eq!(process_stats.process_id, "test-process");
        assert_eq!(process_stats.workspace, "test-workspace");
    }

    #[tokio::test]
    async fn test_debug_info_logging() {
        let (logger, _temp_dir) = create_test_logger().await;
        
        let mut debug_data = HashMap::new();
        debug_data.insert("memory_mb".to_string(), serde_json::Value::Number(512.into()));
        debug_data.insert("cpu_percent".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(75.5).unwrap()));

        let debug_info = DebugInfo {
            process_id: "test-process".to_string(),
            workspace: "test-workspace".to_string(),
            timestamp: Utc::now(),
            debug_type: DebugType::MemoryUsage,
            data: debug_data,
        };

        let result = logger.log_debug_info(debug_info);
        assert!(result.is_ok());
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }
}