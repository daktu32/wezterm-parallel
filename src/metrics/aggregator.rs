// Metrics aggregation and analysis for dashboard display

use super::{
    SystemMetrics, ProcessMetrics, WorkspaceMetrics, FrameworkMetrics, 
    PerformanceSummary, SystemHealthStatus, MetricsConfig,
};
use log::{info, warn, debug};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Metrics aggregator for real-time dashboard
pub struct MetricsAggregator {
    /// Configuration
    config: MetricsConfig,
    
    /// Historical system metrics
    system_history: RwLock<Vec<SystemMetrics>>,
    
    /// Historical process metrics by process ID
    process_history: RwLock<HashMap<String, Vec<ProcessMetrics>>>,
    
    /// Aggregated workspace metrics
    workspace_metrics: RwLock<HashMap<String, WorkspaceMetrics>>,
    
    /// Current framework metrics
    framework_metrics: RwLock<FrameworkMetrics>,
    
    /// Performance tracking
    performance_tracker: RwLock<PerformanceTracker>,
    
    /// Alert thresholds
    alert_thresholds: AlertThresholds,
}

/// Performance tracking for calculating statistics
#[derive(Debug)]
struct PerformanceTracker {
    /// Response time samples
    response_times: Vec<u64>,
    
    /// Request count
    total_requests: u64,
    
    /// Error count
    total_errors: u64,
    
    /// Start time for rate calculations
    start_time: SystemTime,
    
    /// Last reset time
    last_reset: SystemTime,
}

/// Alert threshold configuration
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// CPU usage threshold (percentage)
    pub cpu_threshold: f64,
    
    /// Memory usage threshold (percentage)
    pub memory_threshold: f64,
    
    /// Disk usage threshold (percentage)
    pub disk_threshold: f64,
    
    /// Response time threshold (milliseconds)
    pub response_time_threshold: u64,
    
    /// Error rate threshold (percentage)
    pub error_rate_threshold: f64,
    
    /// Process failure threshold (count)
    pub process_failure_threshold: u32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_threshold: 80.0,
            memory_threshold: 85.0,
            disk_threshold: 90.0,
            response_time_threshold: 5000,
            error_rate_threshold: 5.0,
            process_failure_threshold: 3,
        }
    }
}

impl MetricsAggregator {
    /// Create a new metrics aggregator
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            config,
            system_history: RwLock::new(Vec::new()),
            process_history: RwLock::new(HashMap::new()),
            workspace_metrics: RwLock::new(HashMap::new()),
            framework_metrics: RwLock::new(FrameworkMetrics::new()),
            performance_tracker: RwLock::new(PerformanceTracker::new()),
            alert_thresholds: AlertThresholds::default(),
        }
    }
    
    /// Add system metrics to aggregation
    pub async fn add_system_metrics(&self, metrics: SystemMetrics) {
        debug!("Adding system metrics to aggregation");
        
        let mut history = self.system_history.write().await;
        history.push(metrics.clone());
        
        // Trim history to max points
        if history.len() > self.config.max_history_points {
            history.drain(0..history.len() - self.config.max_history_points);
        }
        
        // Update framework metrics
        let mut framework = self.framework_metrics.write().await;
        framework.system = metrics;
        framework.timestamp = Self::current_timestamp();
    }
    
    /// Add process metrics to aggregation
    pub async fn add_process_metrics(&self, metrics: Vec<ProcessMetrics>) {
        debug!("Adding {} process metrics to aggregation", metrics.len());
        
        let mut process_history = self.process_history.write().await;
        let mut workspace_metrics = self.workspace_metrics.write().await;
        
        // Group processes by workspace
        let mut workspace_processes: HashMap<String, Vec<ProcessMetrics>> = HashMap::new();
        
        for metric in metrics {
            // Add to process history
            let process_id = metric.process_id.clone();
            let workspace = metric.workspace.clone();
            
            process_history
                .entry(process_id)
                .or_insert_with(Vec::new)
                .push(metric.clone());
            
            // Trim process history
            if let Some(history) = process_history.get_mut(&metric.process_id) {
                if history.len() > self.config.max_history_points {
                    history.drain(0..history.len() - self.config.max_history_points);
                }
            }
            
            // Group by workspace
            workspace_processes
                .entry(workspace)
                .or_insert_with(Vec::new)
                .push(metric);
        }
        
        // Update workspace metrics
        for (workspace_name, processes) in workspace_processes {
            let mut workspace_metric = workspace_metrics
                .entry(workspace_name.clone())
                .or_insert_with(|| WorkspaceMetrics::new(workspace_name));
            
            workspace_metric.update_from_processes(processes);
        }
        
        // Update framework metrics
        let mut framework = self.framework_metrics.write().await;
        let workspace_list: Vec<WorkspaceMetrics> = workspace_metrics.values().cloned().collect();
        framework.update_from_workspaces(workspace_list);
    }
    
    /// Get current framework metrics
    pub async fn get_framework_metrics(&self) -> FrameworkMetrics {
        self.framework_metrics.read().await.clone()
    }
    
    /// Get workspace metrics
    pub async fn get_workspace_metrics(&self, workspace_name: &str) -> Option<WorkspaceMetrics> {
        self.workspace_metrics.read().await.get(workspace_name).cloned()
    }
    
    /// Get all workspace metrics
    pub async fn get_all_workspace_metrics(&self) -> HashMap<String, WorkspaceMetrics> {
        self.workspace_metrics.read().await.clone()
    }
    
    /// Get system metrics history
    pub async fn get_system_history(&self, limit: Option<usize>) -> Vec<SystemMetrics> {
        let history = self.system_history.read().await;
        
        match limit {
            Some(n) => {
                let start = if history.len() > n { history.len() - n } else { 0 };
                history[start..].to_vec()
            }
            None => history.clone(),
        }
    }
    
    /// Get process metrics history
    pub async fn get_process_history(&self, process_id: &str, limit: Option<usize>) -> Vec<ProcessMetrics> {
        let history = self.process_history.read().await;
        
        if let Some(process_history) = history.get(process_id) {
            match limit {
                Some(n) => {
                    let start = if process_history.len() > n { process_history.len() - n } else { 0 };
                    process_history[start..].to_vec()
                }
                None => process_history.clone(),
            }
        } else {
            Vec::new()
        }
    }
    
    /// Calculate performance summary
    pub async fn calculate_performance_summary(&self) -> PerformanceSummary {
        let tracker = self.performance_tracker.read().await;
        tracker.calculate_summary()
    }
    
    /// Add performance data point
    pub async fn add_performance_data(&self, response_time: Option<u64>, is_error: bool) {
        let mut tracker = self.performance_tracker.write().await;
        
        tracker.total_requests += 1;
        
        if is_error {
            tracker.total_errors += 1;
        }
        
        if let Some(rt) = response_time {
            tracker.response_times.push(rt);
            
            // Trim response times to prevent memory bloat
            if tracker.response_times.len() > 1000 {
                tracker.response_times.drain(0..500);
            }
        }
    }
    
    /// Reset performance tracking
    pub async fn reset_performance_tracking(&self) {
        let mut tracker = self.performance_tracker.write().await;
        tracker.reset();
    }
    
    /// Get aggregated statistics
    pub async fn get_aggregation_stats(&self) -> AggregationStats {
        let system_history = self.system_history.read().await;
        let process_history = self.process_history.read().await;
        let workspace_metrics = self.workspace_metrics.read().await;
        
        AggregationStats {
            system_history_points: system_history.len(),
            process_history_points: process_history.values().map(|v| v.len()).sum(),
            workspace_count: workspace_metrics.len(),
            total_processes: workspace_metrics.values().map(|w| w.total_processes).sum(),
            memory_usage_mb: self.estimate_memory_usage(&*system_history, &*process_history),
        }
    }
    
    /// Check for alerts
    pub async fn check_alerts(&self) -> Vec<Alert> {
        let mut alerts = Vec::new();
        
        // Check system alerts
        if let Some(latest_system) = self.system_history.read().await.last() {
            alerts.extend(self.check_system_alerts(latest_system));
        }
        
        // Check workspace alerts
        for workspace in self.workspace_metrics.read().await.values() {
            alerts.extend(self.check_workspace_alerts(workspace));
        }
        
        // Check performance alerts
        let performance = self.calculate_performance_summary().await;
        alerts.extend(self.check_performance_alerts(&performance));
        
        alerts
    }
    
    /// Check system-level alerts
    fn check_system_alerts(&self, metrics: &SystemMetrics) -> Vec<Alert> {
        let mut alerts = Vec::new();
        
        if metrics.cpu_usage > self.alert_thresholds.cpu_threshold {
            alerts.push(Alert {
                severity: AlertSeverity::Warning,
                category: AlertCategory::System,
                message: format!("High CPU usage: {:.1}%", metrics.cpu_usage),
                timestamp: metrics.timestamp,
                details: Some(format!("CPU usage is above threshold of {:.1}%", self.alert_thresholds.cpu_threshold)),
            });
        }
        
        if metrics.memory_percentage > self.alert_thresholds.memory_threshold {
            alerts.push(Alert {
                severity: AlertSeverity::Warning,
                category: AlertCategory::System,
                message: format!("High memory usage: {:.1}%", metrics.memory_percentage),
                timestamp: metrics.timestamp,
                details: Some(format!("Memory usage is above threshold of {:.1}%", self.alert_thresholds.memory_threshold)),
            });
        }
        
        if metrics.disk_percentage > self.alert_thresholds.disk_threshold {
            alerts.push(Alert {
                severity: AlertSeverity::Critical,
                category: AlertCategory::System,
                message: format!("High disk usage: {:.1}%", metrics.disk_percentage),
                timestamp: metrics.timestamp,
                details: Some(format!("Disk usage is above threshold of {:.1}%", self.alert_thresholds.disk_threshold)),
            });
        }
        
        alerts
    }
    
    /// Check workspace-level alerts
    fn check_workspace_alerts(&self, workspace: &WorkspaceMetrics) -> Vec<Alert> {
        let mut alerts = Vec::new();
        
        if workspace.failed_processes >= self.alert_thresholds.process_failure_threshold {
            alerts.push(Alert {
                severity: AlertSeverity::Critical,
                category: AlertCategory::Process,
                message: format!("Multiple process failures in workspace {}: {}", workspace.workspace_name, workspace.failed_processes),
                timestamp: workspace.timestamp,
                details: Some(format!("{} processes have failed", workspace.failed_processes)),
            });
        }
        
        if workspace.health_score < 50.0 {
            alerts.push(Alert {
                severity: AlertSeverity::Warning,
                category: AlertCategory::Workspace,
                message: format!("Low workspace health: {} ({:.1}%)", workspace.workspace_name, workspace.health_score),
                timestamp: workspace.timestamp,
                details: Some("Workspace health is below 50%".to_string()),
            });
        }
        
        alerts
    }
    
    /// Check performance-level alerts
    fn check_performance_alerts(&self, performance: &PerformanceSummary) -> Vec<Alert> {
        let mut alerts = Vec::new();
        
        if performance.avg_response_time > self.alert_thresholds.response_time_threshold as f64 {
            alerts.push(Alert {
                severity: AlertSeverity::Warning,
                category: AlertCategory::Performance,
                message: format!("High response time: {:.0}ms", performance.avg_response_time),
                timestamp: Self::current_timestamp(),
                details: Some(format!("Average response time is above threshold of {}ms", self.alert_thresholds.response_time_threshold)),
            });
        }
        
        if performance.error_rate > self.alert_thresholds.error_rate_threshold {
            alerts.push(Alert {
                severity: AlertSeverity::Critical,
                category: AlertCategory::Performance,
                message: format!("High error rate: {:.1}%", performance.error_rate),
                timestamp: Self::current_timestamp(),
                details: Some(format!("Error rate is above threshold of {:.1}%", self.alert_thresholds.error_rate_threshold)),
            });
        }
        
        alerts
    }
    
    /// Estimate memory usage of stored metrics
    fn estimate_memory_usage(&self, system_history: &[SystemMetrics], process_history: &HashMap<String, Vec<ProcessMetrics>>) -> usize {
        let system_size = system_history.len() * std::mem::size_of::<SystemMetrics>();
        let process_size = process_history.values()
            .map(|v| v.len() * std::mem::size_of::<ProcessMetrics>())
            .sum::<usize>();
        
        (system_size + process_size) / (1024 * 1024) // Convert to MB
    }
    
    /// Get current timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
    
    /// Update configuration
    pub async fn update_config(&self, config: MetricsConfig) {
        info!("Updating metrics aggregator configuration");
        // Note: self.config is not mutable in this design
        // In practice, you might want to make it RwLock<MetricsConfig>
    }
    
    /// Clean up old metrics data
    pub async fn cleanup_old_data(&self) {
        let cutoff_time = Self::current_timestamp().saturating_sub(self.config.retention_hours * 3600);
        
        // Clean system history
        let mut system_history = self.system_history.write().await;
        system_history.retain(|m| m.timestamp > cutoff_time);
        
        // Clean process history
        let mut process_history = self.process_history.write().await;
        for history in process_history.values_mut() {
            history.retain(|m| m.timestamp > cutoff_time);
        }
        
        // Remove empty process histories
        process_history.retain(|_, history| !history.is_empty());
        
        info!("Cleaned up metrics data older than {} hours", self.config.retention_hours);
    }
}

impl PerformanceTracker {
    fn new() -> Self {
        let now = SystemTime::now();
        Self {
            response_times: Vec::new(),
            total_requests: 0,
            total_errors: 0,
            start_time: now,
            last_reset: now,
        }
    }
    
    fn calculate_summary(&self) -> PerformanceSummary {
        let elapsed = self.start_time.elapsed().unwrap_or_default().as_secs_f64();
        let requests_per_second = if elapsed > 0.0 {
            self.total_requests as f64 / elapsed
        } else {
            0.0
        };
        
        let error_rate = if self.total_requests > 0 {
            (self.total_errors as f64 / self.total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        let (avg_response_time, p95_response_time, p99_response_time) = if self.response_times.is_empty() {
            (0.0, 0.0, 0.0)
        } else {
            let mut sorted_times = self.response_times.clone();
            sorted_times.sort_unstable();
            
            let avg = sorted_times.iter().sum::<u64>() as f64 / sorted_times.len() as f64;
            let p95_idx = (sorted_times.len() as f64 * 0.95) as usize;
            let p99_idx = (sorted_times.len() as f64 * 0.99) as usize;
            
            let p95 = sorted_times.get(p95_idx.saturating_sub(1)).copied().unwrap_or(0) as f64;
            let p99 = sorted_times.get(p99_idx.saturating_sub(1)).copied().unwrap_or(0) as f64;
            
            (avg, p95, p99)
        };
        
        PerformanceSummary {
            avg_response_time,
            p95_response_time,
            p99_response_time,
            total_requests: self.total_requests,
            requests_per_second,
            error_rate,
            total_errors: self.total_errors,
        }
    }
    
    fn reset(&mut self) {
        self.response_times.clear();
        self.total_requests = 0;
        self.total_errors = 0;
        self.start_time = SystemTime::now();
        self.last_reset = SystemTime::now();
    }
}

/// Aggregation statistics
#[derive(Debug, Clone)]
pub struct AggregationStats {
    pub system_history_points: usize,
    pub process_history_points: usize,
    pub workspace_count: usize,
    pub total_processes: u32,
    pub memory_usage_mb: usize,
}

/// Alert information
#[derive(Debug, Clone)]
pub struct Alert {
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub message: String,
    pub timestamp: u64,
    pub details: Option<String>,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert categories
#[derive(Debug, Clone, PartialEq)]
pub enum AlertCategory {
    System,
    Process,
    Workspace,
    Performance,
    Network,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_metrics_aggregation() {
        let config = MetricsConfig::default();
        let aggregator = MetricsAggregator::new(config);
        
        let system_metrics = SystemMetrics::new();
        aggregator.add_system_metrics(system_metrics).await;
        
        let framework_metrics = aggregator.get_framework_metrics().await;
        assert!(framework_metrics.timestamp > 0);
    }
    
    #[tokio::test]
    async fn test_performance_tracking() {
        let config = MetricsConfig::default();
        let aggregator = MetricsAggregator::new(config);
        
        aggregator.add_performance_data(Some(100), false).await;
        aggregator.add_performance_data(Some(200), true).await;
        
        let summary = aggregator.calculate_performance_summary().await;
        assert_eq!(summary.total_requests, 2);
        assert_eq!(summary.total_errors, 1);
        assert!(summary.avg_response_time > 0.0);
    }
}