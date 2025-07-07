// WezTerm Multi-Process Development Framework - Performance Metrics
// パフォーマンスメトリクス収集・分析

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::logging::LogContext;
use crate::{log_debug, log_info};

/// パフォーマンスメトリクス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: u64,
    pub cpu_usage: f64,
    pub memory_usage: usize,
    pub memory_peak: usize,
    pub task_count: usize,
    pub response_time: Duration,
    pub throughput: f64,
    pub error_rate: f64,
    pub gc_count: u32,
    pub gc_duration: Duration,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            cpu_usage: 0.0,
            memory_usage: 0,
            memory_peak: 0,
            task_count: 0,
            response_time: Duration::from_millis(0),
            throughput: 0.0,
            error_rate: 0.0,
            gc_count: 0,
            gc_duration: Duration::from_millis(0),
        }
    }
}

/// メトリクス収集器
pub struct MetricsCollector {
    metrics_history: Arc<RwLock<VecDeque<PerformanceMetrics>>>,
    max_history_size: usize,
    collection_interval: Duration,
    current_metrics: Arc<RwLock<PerformanceMetrics>>,
    collection_handle: Option<tokio::task::JoinHandle<()>>,
    
    // 実行時統計
    response_times: Arc<RwLock<VecDeque<Duration>>>,
    error_counts: Arc<RwLock<HashMap<String, u32>>>,
    operation_counts: Arc<RwLock<HashMap<String, u64>>>,
}

impl MetricsCollector {
    pub fn new(max_history_size: usize, collection_interval: Duration) -> Self {
        Self {
            metrics_history: Arc::new(RwLock::new(VecDeque::new())),
            max_history_size,
            collection_interval,
            current_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            collection_handle: None,
            response_times: Arc::new(RwLock::new(VecDeque::new())),
            error_counts: Arc::new(RwLock::new(HashMap::new())),
            operation_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// メトリクス収集を開始
    pub fn start_collection(&mut self) {
        let metrics_history = Arc::clone(&self.metrics_history);
        let current_metrics = Arc::clone(&self.current_metrics);
        let max_history_size = self.max_history_size;
        let collection_interval = self.collection_interval;
        
        self.collection_handle = Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(collection_interval);
            
            loop {
                interval.tick().await;
                
                let metrics = {
                    let current = current_metrics.read().await;
                    current.clone()
                };
                
                {
                    let mut history = metrics_history.write().await;
                    history.push_back(metrics);
                    
                    // 履歴サイズ制限
                    while history.len() > max_history_size {
                        history.pop_front();
                    }
                }
                
                let collection_context = LogContext::new("performance", "metrics_collection");
                log_debug!(collection_context, "パフォーマンスメトリクス収集完了");
            }
        }));
        
        let start_context = LogContext::new("performance", "metrics_start")
            .with_metadata("collection_interval_ms", serde_json::json!(collection_interval.as_millis()));
        log_info!(start_context, "メトリクス収集開始: 間隔={:?}", collection_interval);
    }

    /// メトリクス収集を停止
    pub fn stop_collection(&mut self) {
        if let Some(handle) = self.collection_handle.take() {
            handle.abort();
            let stop_context = LogContext::new("performance", "metrics_stop");
            log_info!(stop_context, "メトリクス収集停止");
        }
    }

    /// CPU使用率を更新
    pub async fn update_cpu_usage(&self, usage: f64) {
        let mut metrics = self.current_metrics.write().await;
        metrics.cpu_usage = usage;
        metrics.timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    }

    /// メモリ使用量を更新
    pub async fn update_memory_usage(&self, usage: usize, peak: usize) {
        let mut metrics = self.current_metrics.write().await;
        metrics.memory_usage = usage;
        metrics.memory_peak = peak;
    }

    /// タスク数を更新
    pub async fn update_task_count(&self, count: usize) {
        let mut metrics = self.current_metrics.write().await;
        metrics.task_count = count;
    }

    /// 応答時間を記録
    pub async fn record_response_time(&self, duration: Duration) {
        // 応答時間履歴を更新
        {
            let mut response_times = self.response_times.write().await;
            response_times.push_back(duration);
            
            // 最新100件のみ保持
            while response_times.len() > 100 {
                response_times.pop_front();
            }
        }
        
        // 平均応答時間を計算
        let avg_response_time = {
            let response_times = self.response_times.read().await;
            if response_times.is_empty() {
                Duration::from_millis(0)
            } else {
                let total_nanos: u64 = response_times.iter().map(|d| d.as_nanos() as u64).sum();
                Duration::from_nanos(total_nanos / response_times.len() as u64)
            }
        };
        
        let mut metrics = self.current_metrics.write().await;
        metrics.response_time = avg_response_time;
    }

    /// エラーを記録
    pub async fn record_error(&self, error_type: &str) {
        // エラーカウントを更新（ロックを早期に解放）
        {
            let mut error_counts = self.error_counts.write().await;
            *error_counts.entry(error_type.to_string()).or_insert(0) += 1;
        } // ここでロックが解放される
        
        // エラー率を計算
        self.calculate_error_rate().await;
    }

    /// 操作を記録
    pub async fn record_operation(&self, operation_type: &str) {
        // 操作カウントを更新（ロックを早期に解放）
        {
            let mut operation_counts = self.operation_counts.write().await;
            *operation_counts.entry(operation_type.to_string()).or_insert(0) += 1;
        } // ここでロックが解放される
        
        // スループットを計算
        self.calculate_throughput().await;
    }

    /// ガベージコレクション情報を更新
    pub async fn update_gc_info(&self, count: u32, duration: Duration) {
        let mut metrics = self.current_metrics.write().await;
        metrics.gc_count = count;
        metrics.gc_duration = duration;
    }

    /// エラー率を計算
    async fn calculate_error_rate(&self) {
        let (total_errors, total_operations) = {
            let error_counts = self.error_counts.read().await;
            let operation_counts = self.operation_counts.read().await;
            
            let total_errors: u32 = error_counts.values().sum();
            let total_operations: u64 = operation_counts.values().sum();
            
            (total_errors, total_operations)
        };
        
        let error_rate = if total_operations > 0 {
            (total_errors as f64 / total_operations as f64) * 100.0
        } else {
            0.0
        };
        
        let mut metrics = self.current_metrics.write().await;
        metrics.error_rate = error_rate;
    }

    /// スループットを計算
    async fn calculate_throughput(&self) {
        let total_operations = {
            let operation_counts = self.operation_counts.read().await;
            operation_counts.values().sum::<u64>()
        };
        
        // 1秒あたりの操作数として計算（簡略化）
        let throughput = total_operations as f64 / 60.0; // 過去1分間の平均
        
        let mut metrics = self.current_metrics.write().await;
        metrics.throughput = throughput;
    }

    /// 現在のメトリクスを取得
    pub async fn get_current_metrics(&self) -> PerformanceMetrics {
        let metrics = self.current_metrics.read().await;
        metrics.clone()
    }

    /// メトリクス履歴を取得
    pub async fn get_metrics_history(&self) -> Vec<PerformanceMetrics> {
        let history = self.metrics_history.read().await;
        history.iter().cloned().collect()
    }

    /// 統計サマリーを生成
    pub async fn generate_summary(&self) -> MetricsSummary {
        let history = self.metrics_history.read().await;
        
        if history.is_empty() {
            return MetricsSummary::default();
        }
        
        let cpu_values: Vec<f64> = history.iter().map(|m| m.cpu_usage).collect();
        let memory_values: Vec<usize> = history.iter().map(|m| m.memory_usage).collect();
        let response_times: Vec<Duration> = history.iter().map(|m| m.response_time).collect();
        
        MetricsSummary {
            sample_count: history.len(),
            avg_cpu_usage: cpu_values.iter().sum::<f64>() / cpu_values.len() as f64,
            max_cpu_usage: cpu_values.iter().fold(0.0, |a, &b| a.max(b)),
            min_cpu_usage: cpu_values.iter().fold(100.0, |a, &b| a.min(b)),
            avg_memory_usage: memory_values.iter().sum::<usize>() / memory_values.len(),
            max_memory_usage: *memory_values.iter().max().unwrap_or(&0),
            min_memory_usage: *memory_values.iter().min().unwrap_or(&0),
            avg_response_time: Duration::from_nanos(
                response_times.iter().map(|d| d.as_nanos() as u64).sum::<u64>() / response_times.len() as u64
            ),
            max_response_time: response_times.iter().max().copied().unwrap_or_default(),
            min_response_time: response_times.iter().min().copied().unwrap_or_default(),
            total_gc_count: history.iter().map(|m| m.gc_count).max().unwrap_or(0),
            avg_throughput: history.iter().map(|m| m.throughput).sum::<f64>() / history.len() as f64,
            avg_error_rate: history.iter().map(|m| m.error_rate).sum::<f64>() / history.len() as f64,
        }
    }

    /// パフォーマンスアラートをチェック
    pub async fn check_performance_alerts(&self) -> Vec<PerformanceAlert> {
        let current = self.current_metrics.read().await;
        let mut alerts = Vec::new();
        
        // CPU使用率アラート
        if current.cpu_usage > 90.0 {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::CpuHigh,
                severity: AlertSeverity::Critical,
                message: format!("CPU使用率が危険レベル: {:.1}%", current.cpu_usage),
                value: current.cpu_usage,
                threshold: 90.0,
            });
        } else if current.cpu_usage > 80.0 {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::CpuHigh,
                severity: AlertSeverity::Warning,
                message: format!("CPU使用率が高い: {:.1}%", current.cpu_usage),
                value: current.cpu_usage,
                threshold: 80.0,
            });
        }
        
        // メモリ使用量アラート
        let memory_mb = current.memory_usage / 1024 / 1024;
        if memory_mb > 1024 {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::MemoryHigh,
                severity: AlertSeverity::Critical,
                message: format!("メモリ使用量が危険レベル: {}MB", memory_mb),
                value: memory_mb as f64,
                threshold: 1024.0,
            });
        } else if memory_mb > 512 {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::MemoryHigh,
                severity: AlertSeverity::Warning,
                message: format!("メモリ使用量が高い: {}MB", memory_mb),
                value: memory_mb as f64,
                threshold: 512.0,
            });
        }
        
        // 応答時間アラート
        if current.response_time > Duration::from_millis(1000) {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::ResponseTimeSlow,
                severity: AlertSeverity::Critical,
                message: format!("応答時間が遅い: {:?}", current.response_time),
                value: current.response_time.as_millis() as f64,
                threshold: 1000.0,
            });
        } else if current.response_time > Duration::from_millis(500) {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::ResponseTimeSlow,
                severity: AlertSeverity::Warning,
                message: format!("応答時間が長い: {:?}", current.response_time),
                value: current.response_time.as_millis() as f64,
                threshold: 500.0,
            });
        }
        
        // エラー率アラート
        if current.error_rate > 10.0 {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::ErrorRateHigh,
                severity: AlertSeverity::Critical,
                message: format!("エラー率が高い: {:.1}%", current.error_rate),
                value: current.error_rate,
                threshold: 10.0,
            });
        } else if current.error_rate > 5.0 {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::ErrorRateHigh,
                severity: AlertSeverity::Warning,
                message: format!("エラー率が上昇: {:.1}%", current.error_rate),
                value: current.error_rate,
                threshold: 5.0,
            });
        }
        
        alerts
    }

    /// 詳細レポートを生成
    pub async fn generate_detailed_report(&self) -> String {
        let current = self.current_metrics.read().await;
        let summary = self.generate_summary().await;
        let alerts = self.check_performance_alerts().await;
        
        let mut report = String::new();
        report.push_str("=== 詳細パフォーマンスレポート ===\n\n");
        
        // 現在の状況
        report.push_str("【現在の状況】\n");
        report.push_str(&format!("CPU使用率: {:.1}%\n", current.cpu_usage));
        report.push_str(&format!("メモリ使用量: {}MB (ピーク: {}MB)\n", 
                                current.memory_usage / 1024 / 1024, 
                                current.memory_peak / 1024 / 1024));
        report.push_str(&format!("アクティブタスク: {}\n", current.task_count));
        report.push_str(&format!("応答時間: {:?}\n", current.response_time));
        report.push_str(&format!("スループット: {:.1} ops/min\n", current.throughput));
        report.push_str(&format!("エラー率: {:.1}%\n", current.error_rate));
        report.push_str(&format!("GC実行回数: {} (合計時間: {:?})\n\n", current.gc_count, current.gc_duration));
        
        // 統計サマリー
        report.push_str("【統計サマリー】\n");
        report.push_str(&format!("サンプル数: {}\n", summary.sample_count));
        report.push_str(&format!("CPU使用率: 平均={:.1}%, 最大={:.1}%, 最小={:.1}%\n", 
                                summary.avg_cpu_usage, summary.max_cpu_usage, summary.min_cpu_usage));
        report.push_str(&format!("メモリ使用量: 平均={}MB, 最大={}MB, 最小={}MB\n", 
                                summary.avg_memory_usage / 1024 / 1024, 
                                summary.max_memory_usage / 1024 / 1024, 
                                summary.min_memory_usage / 1024 / 1024));
        report.push_str(&format!("応答時間: 平均={:?}, 最大={:?}, 最小={:?}\n", 
                                summary.avg_response_time, summary.max_response_time, summary.min_response_time));
        report.push_str(&format!("平均スループット: {:.1} ops/min\n", summary.avg_throughput));
        report.push_str(&format!("平均エラー率: {:.1}%\n\n", summary.avg_error_rate));
        
        // アラート
        if !alerts.is_empty() {
            report.push_str("【アラート】\n");
            for alert in alerts {
                let severity_str = match alert.severity {
                    AlertSeverity::Critical => "🔴 CRITICAL",
                    AlertSeverity::Warning => "🟡 WARNING",
                    AlertSeverity::Info => "🔵 INFO",
                };
                report.push_str(&format!("{}: {}\n", severity_str, alert.message));
            }
        } else {
            report.push_str("【アラート】\n✅ 問題なし\n");
        }
        
        report
    }
}

/// メトリクスサマリー
#[derive(Debug, Clone, Default)]
pub struct MetricsSummary {
    pub sample_count: usize,
    pub avg_cpu_usage: f64,
    pub max_cpu_usage: f64,
    pub min_cpu_usage: f64,
    pub avg_memory_usage: usize,
    pub max_memory_usage: usize,
    pub min_memory_usage: usize,
    pub avg_response_time: Duration,
    pub max_response_time: Duration,
    pub min_response_time: Duration,
    pub total_gc_count: u32,
    pub avg_throughput: f64,
    pub avg_error_rate: f64,
}

/// パフォーマンスアラート
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertType {
    CpuHigh,
    MemoryHigh,
    ResponseTimeSlow,
    ErrorRateHigh,
    ThroughputLow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new(10, Duration::from_millis(50));
        
        collector.update_cpu_usage(50.0).await;
        collector.update_memory_usage(1024 * 1024 * 100, 1024 * 1024 * 150).await; // 100MB, 150MB peak
        collector.update_task_count(5).await;
        
        let metrics = collector.get_current_metrics().await;
        assert_eq!(metrics.cpu_usage, 50.0);
        assert_eq!(metrics.memory_usage, 1024 * 1024 * 100);
        assert_eq!(metrics.task_count, 5);
    }

    #[tokio::test]
    async fn test_response_time_recording() {
        let collector = MetricsCollector::new(10, Duration::from_millis(50));
        
        collector.record_response_time(Duration::from_millis(100)).await;
        collector.record_response_time(Duration::from_millis(200)).await;
        
        let metrics = collector.get_current_metrics().await;
        assert_eq!(metrics.response_time, Duration::from_millis(150)); // 平均
    }

    #[tokio::test]
    async fn test_error_recording() {
        let collector = MetricsCollector::new(10, Duration::from_millis(50));
        
        collector.record_operation("test_op").await;
        collector.record_operation("test_op").await;
        collector.record_error("test_error").await;
        
        let metrics = collector.get_current_metrics().await;
        assert_eq!(metrics.error_rate, 50.0); // 1 error / 2 operations * 100
    }

    #[tokio::test]
    async fn test_performance_alerts() {
        let collector = MetricsCollector::new(10, Duration::from_millis(50));
        
        // 高CPU使用率をシミュレート
        collector.update_cpu_usage(95.0).await;
        
        let alerts = collector.check_performance_alerts().await;
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].alert_type, AlertType::CpuHigh);
        assert_eq!(alerts[0].severity, AlertSeverity::Critical);
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let mut collector = MetricsCollector::new(3, Duration::from_millis(10));
        
        collector.start_collection();
        sleep(Duration::from_millis(50)).await;
        collector.stop_collection();
        
        let history = collector.get_metrics_history().await;
        assert!(!history.is_empty());
    }
}