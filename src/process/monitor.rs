// WezTerm Multi-Process Development Framework - Process Monitor

use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use tokio::sync::RwLock;
use tokio::time::sleep;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};

use crate::room::state::{ProcessInfo, ProcessStatus};
use super::manager::ProcessManager;

#[derive(Debug)]
pub struct ProcessMonitor {
    manager: ProcessManager,
    metrics: RwLock<HashMap<String, ProcessMetrics>>,
    config: MonitorConfig,
    alerts: RwLock<Vec<Alert>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MonitorConfig {
    pub monitor_interval_secs: u64,
    pub cpu_threshold_percent: f32,
    pub memory_threshold_mb: u64,
    pub response_timeout_secs: u64,
    pub restart_on_failure: bool,
    pub max_consecutive_failures: u32,
    pub alert_cooldown_secs: u64,
    pub enable_performance_logging: bool,
}

#[derive(Debug, Clone)]
pub struct ProcessMetrics {
    pub process_id: String,
    pub workspace: String,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: u64,
    pub response_time_ms: u64,
    pub uptime_secs: u64,
    pub restart_count: u32,
    pub last_heartbeat: SystemTime,
    pub is_responsive: bool,
    pub consecutive_failures: u32,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub id: String,
    pub process_id: String,
    pub alert_type: AlertType,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: SystemTime,
    pub acknowledged: bool,
}

#[derive(Debug, Clone)]
pub enum AlertType {
    HighCpuUsage,
    HighMemoryUsage,
    ProcessUnresponsive,
    ProcessCrashed,
    TooManyRestarts,
    PerformanceDegradation,
}

#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            monitor_interval_secs: 30,
            cpu_threshold_percent: 80.0,
            memory_threshold_mb: 512,
            response_timeout_secs: 5,
            restart_on_failure: true,
            max_consecutive_failures: 3,
            alert_cooldown_secs: 300, // 5 minutes
            enable_performance_logging: true,
        }
    }
}

impl ProcessMonitor {
    pub fn new(manager: ProcessManager, config: MonitorConfig) -> Self {
        Self {
            manager,
            metrics: RwLock::new(HashMap::new()),
            config,
            alerts: RwLock::new(Vec::new()),
        }
    }

    pub async fn start_monitoring(&self) {
        info!("Starting process monitoring with interval {}s", self.config.monitor_interval_secs);
        
        let monitor_interval = Duration::from_secs(self.config.monitor_interval_secs);
        
        loop {
            self.collect_metrics().await;
            self.check_alerts().await;
            self.cleanup_old_alerts().await;
            
            sleep(monitor_interval).await;
        }
    }

    async fn collect_metrics(&self) {
        let processes = self.manager.list_processes().await;
        let mut metrics = self.metrics.write().await;
        
        for process in &processes {
            let process_metrics = self.measure_process(&process).await;
            
            if self.config.enable_performance_logging {
                debug!("Process '{}' metrics: CPU: {:.1}%, Memory: {}MB, Responsive: {}", 
                       process_metrics.process_id,
                       process_metrics.cpu_usage_percent,
                       process_metrics.memory_usage_mb,
                       process_metrics.is_responsive);
            }
            
            metrics.insert(process.id.clone(), process_metrics);
        }
        
        // Remove metrics for processes that no longer exist
        let current_process_ids: std::collections::HashSet<_> = processes.iter().map(|p| &p.id).collect();
        metrics.retain(|id, _| current_process_ids.contains(id));
    }

    async fn measure_process(&self, process: &ProcessInfo) -> ProcessMetrics {
        let now = SystemTime::now();
        let uptime = now.duration_since(process.started_at).unwrap_or_default().as_secs();
        
        // Get existing metrics for historical data
        let existing_metrics = {
            let metrics = self.metrics.read().await;
            metrics.get(&process.id).cloned()
        };

        // TODO: Implement actual system metrics collection
        // For now, we'll simulate metrics
        let (cpu_usage, memory_usage, response_time, is_responsive) = 
            self.simulate_process_metrics(process, &existing_metrics).await;

        ProcessMetrics {
            process_id: process.id.clone(),
            workspace: process.workspace.clone(),
            cpu_usage_percent: cpu_usage,
            memory_usage_mb: memory_usage,
            response_time_ms: response_time,
            uptime_secs: uptime,
            restart_count: process.restart_count,
            last_heartbeat: process.last_heartbeat,
            is_responsive,
            consecutive_failures: existing_metrics.as_ref().map(|m| {
                if is_responsive { 0 } else { m.consecutive_failures + 1 }
            }).unwrap_or(0),
            total_requests: existing_metrics.as_ref().map(|m| m.total_requests + 1).unwrap_or(1),
            successful_requests: existing_metrics.as_ref().map(|m| {
                if is_responsive { m.successful_requests + 1 } else { m.successful_requests }
            }).unwrap_or(if is_responsive { 1 } else { 0 }),
            failed_requests: existing_metrics.as_ref().map(|m| {
                if !is_responsive { m.failed_requests + 1 } else { m.failed_requests }
            }).unwrap_or(if !is_responsive { 1 } else { 0 }),
        }
    }

    async fn simulate_process_metrics(
        &self,
        process: &ProcessInfo,
        existing: &Option<ProcessMetrics>,
    ) -> (f32, u64, u64, bool) {
        // Simulate metrics based on process status
        let (base_cpu, base_memory, base_response) = match process.status {
            ProcessStatus::Running => (25.0, 128, 100),
            ProcessStatus::Busy => (60.0, 256, 200),
            ProcessStatus::Idle => (5.0, 64, 50),
            ProcessStatus::Starting => (15.0, 96, 150),
            ProcessStatus::Failed => (0.0, 32, 5000),
            ProcessStatus::Stopped => (0.0, 0, 0),
            ProcessStatus::Stopping => (10.0, 48, 300),
            ProcessStatus::Restarting => (20.0, 112, 250),
        };

        // Add some variance based on existing metrics
        let cpu_usage = if let Some(existing) = existing {
            (existing.cpu_usage_percent * 0.7 + base_cpu * 0.3).min(100.0)
        } else {
            base_cpu
        };

        let memory_usage = if let Some(existing) = existing {
            ((existing.memory_usage_mb as f32 * 0.8 + base_memory as f32 * 0.2) as u64).max(32)
        } else {
            base_memory
        };

        let response_time = base_response;
        let is_responsive = matches!(process.status, 
            ProcessStatus::Running | ProcessStatus::Busy | ProcessStatus::Idle) 
            && response_time < (self.config.response_timeout_secs * 1000);

        (cpu_usage, memory_usage, response_time, is_responsive)
    }

    async fn check_alerts(&self) {
        let metrics = self.metrics.read().await;
        
        for (_process_id, process_metrics) in metrics.iter() {
            self.check_cpu_alert(process_metrics).await;
            self.check_memory_alert(process_metrics).await;
            self.check_responsiveness_alert(process_metrics).await;
            self.check_restart_alert(process_metrics).await;
        }
    }

    async fn check_cpu_alert(&self, metrics: &ProcessMetrics) {
        if metrics.cpu_usage_percent > self.config.cpu_threshold_percent {
            if !self.has_recent_alert(&metrics.process_id, &AlertType::HighCpuUsage).await {
                let alert = Alert {
                    id: format!("cpu-{}-{}", metrics.process_id, chrono::Utc::now().timestamp()),
                    process_id: metrics.process_id.clone(),
                    alert_type: AlertType::HighCpuUsage,
                    message: format!("Process '{}' CPU usage: {:.1}% (threshold: {:.1}%)", 
                                   metrics.process_id, metrics.cpu_usage_percent, self.config.cpu_threshold_percent),
                    severity: if metrics.cpu_usage_percent > 95.0 { 
                        AlertSeverity::Critical 
                    } else { 
                        AlertSeverity::Warning 
                    },
                    timestamp: SystemTime::now(),
                    acknowledged: false,
                };
                
                self.add_alert(alert).await;
            }
        }
    }

    async fn check_memory_alert(&self, metrics: &ProcessMetrics) {
        if metrics.memory_usage_mb > self.config.memory_threshold_mb {
            if !self.has_recent_alert(&metrics.process_id, &AlertType::HighMemoryUsage).await {
                let alert = Alert {
                    id: format!("mem-{}-{}", metrics.process_id, chrono::Utc::now().timestamp()),
                    process_id: metrics.process_id.clone(),
                    alert_type: AlertType::HighMemoryUsage,
                    message: format!("Process '{}' memory usage: {}MB (threshold: {}MB)", 
                                   metrics.process_id, metrics.memory_usage_mb, self.config.memory_threshold_mb),
                    severity: if metrics.memory_usage_mb > self.config.memory_threshold_mb * 2 { 
                        AlertSeverity::Critical 
                    } else { 
                        AlertSeverity::Warning 
                    },
                    timestamp: SystemTime::now(),
                    acknowledged: false,
                };
                
                self.add_alert(alert).await;
            }
        }
    }

    async fn check_responsiveness_alert(&self, metrics: &ProcessMetrics) {
        if !metrics.is_responsive {
            if !self.has_recent_alert(&metrics.process_id, &AlertType::ProcessUnresponsive).await {
                let alert = Alert {
                    id: format!("unresponsive-{}-{}", metrics.process_id, chrono::Utc::now().timestamp()),
                    process_id: metrics.process_id.clone(),
                    alert_type: AlertType::ProcessUnresponsive,
                    message: format!("Process '{}' is unresponsive (response time: {}ms)", 
                                   metrics.process_id, metrics.response_time_ms),
                    severity: AlertSeverity::Critical,
                    timestamp: SystemTime::now(),
                    acknowledged: false,
                };
                
                self.add_alert(alert).await;
                
                // Auto-restart if configured
                if self.config.restart_on_failure && 
                   metrics.consecutive_failures >= self.config.max_consecutive_failures {
                    warn!("Process '{}' has {} consecutive failures, attempting restart", 
                          metrics.process_id, metrics.consecutive_failures);
                    
                    if let Err(e) = self.manager.restart_process(&metrics.process_id).await {
                        error!("Failed to restart unresponsive process '{}': {}", metrics.process_id, e);
                    }
                }
            }
        }
    }

    async fn check_restart_alert(&self, metrics: &ProcessMetrics) {
        if metrics.restart_count > 5 { // Alert if more than 5 restarts
            if !self.has_recent_alert(&metrics.process_id, &AlertType::TooManyRestarts).await {
                let alert = Alert {
                    id: format!("restarts-{}-{}", metrics.process_id, chrono::Utc::now().timestamp()),
                    process_id: metrics.process_id.clone(),
                    alert_type: AlertType::TooManyRestarts,
                    message: format!("Process '{}' has been restarted {} times", 
                                   metrics.process_id, metrics.restart_count),
                    severity: AlertSeverity::Warning,
                    timestamp: SystemTime::now(),
                    acknowledged: false,
                };
                
                self.add_alert(alert).await;
            }
        }
    }

    async fn has_recent_alert(&self, process_id: &str, alert_type: &AlertType) -> bool {
        let alerts = self.alerts.read().await;
        let cooldown = Duration::from_secs(self.config.alert_cooldown_secs);
        let cutoff = SystemTime::now() - cooldown;
        
        alerts.iter().any(|alert| {
            alert.process_id == process_id &&
            std::mem::discriminant(&alert.alert_type) == std::mem::discriminant(alert_type) &&
            alert.timestamp > cutoff
        })
    }

    async fn add_alert(&self, alert: Alert) {
        match alert.severity {
            AlertSeverity::Critical => error!("CRITICAL: {}", alert.message),
            AlertSeverity::Warning => warn!("WARNING: {}", alert.message),
            AlertSeverity::Info => info!("INFO: {}", alert.message),
        }
        
        let mut alerts = self.alerts.write().await;
        alerts.push(alert);
    }

    async fn cleanup_old_alerts(&self) {
        let mut alerts = self.alerts.write().await;
        let cutoff = SystemTime::now() - Duration::from_secs(3600 * 24); // 24 hours
        
        let initial_count = alerts.len();
        alerts.retain(|alert| alert.timestamp > cutoff);
        
        let removed_count = initial_count - alerts.len();
        if removed_count > 0 {
            debug!("Cleaned up {} old alerts", removed_count);
        }
    }

    pub async fn get_process_metrics(&self, process_id: &str) -> Option<ProcessMetrics> {
        let metrics = self.metrics.read().await;
        metrics.get(process_id).cloned()
    }

    pub async fn get_all_metrics(&self) -> HashMap<String, ProcessMetrics> {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.iter()
            .filter(|alert| !alert.acknowledged)
            .cloned()
            .collect()
    }

    pub async fn acknowledge_alert(&self, alert_id: &str) -> Result<(), String> {
        let mut alerts = self.alerts.write().await;
        
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            info!("Alert '{}' acknowledged", alert_id);
            Ok(())
        } else {
            Err(format!("Alert '{}' not found", alert_id))
        }
    }

    pub async fn get_system_health(&self) -> SystemHealth {
        let metrics = self.metrics.read().await;
        let alerts = self.alerts.read().await;
        
        let total_processes = metrics.len();
        let responsive_processes = metrics.values()
            .filter(|m| m.is_responsive)
            .count();
        
        let avg_cpu = if total_processes > 0 {
            metrics.values().map(|m| m.cpu_usage_percent).sum::<f32>() / total_processes as f32
        } else {
            0.0
        };
        
        let total_memory = metrics.values().map(|m| m.memory_usage_mb).sum::<u64>();
        
        let critical_alerts = alerts.iter()
            .filter(|a| !a.acknowledged && matches!(a.severity, AlertSeverity::Critical))
            .count();
        
        let warning_alerts = alerts.iter()
            .filter(|a| !a.acknowledged && matches!(a.severity, AlertSeverity::Warning))
            .count();

        SystemHealth {
            total_processes,
            responsive_processes,
            avg_cpu_usage: avg_cpu,
            total_memory_usage: total_memory,
            critical_alerts,
            warning_alerts,
            overall_status: if critical_alerts > 0 {
                HealthStatus::Critical
            } else if warning_alerts > 0 || responsive_processes < total_processes {
                HealthStatus::Warning
            } else {
                HealthStatus::Healthy
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub total_processes: usize,
    pub responsive_processes: usize,
    pub avg_cpu_usage: f32,
    pub total_memory_usage: u64,
    pub critical_alerts: usize,
    pub warning_alerts: usize,
    pub overall_status: HealthStatus,
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

// Add chrono for timestamp generation
mod chrono {
    pub struct Utc;
    
    impl Utc {
        pub fn now() -> Self {
            Self
        }
        
        pub fn timestamp(&self) -> i64 {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::manager::ProcessConfig;

    #[tokio::test]
    async fn test_process_monitor_creation() {
        let process_config = ProcessConfig::default();
        let (manager, _receiver) = ProcessManager::new(process_config);
        let monitor_config = MonitorConfig::default();
        
        let monitor = ProcessMonitor::new(manager, monitor_config);
        
        let health = monitor.get_system_health().await;
        assert_eq!(health.total_processes, 0);
        assert!(matches!(health.overall_status, HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_alert_system() {
        let process_config = ProcessConfig::default();
        let (manager, _receiver) = ProcessManager::new(process_config);
        let monitor_config = MonitorConfig::default();
        
        let monitor = ProcessMonitor::new(manager, monitor_config);
        
        let alert = Alert {
            id: "test-alert".to_string(),
            process_id: "test-process".to_string(),
            alert_type: AlertType::HighCpuUsage,
            message: "Test alert".to_string(),
            severity: AlertSeverity::Warning,
            timestamp: SystemTime::now(),
            acknowledged: false,
        };
        
        monitor.add_alert(alert).await;
        
        let active_alerts = monitor.get_active_alerts().await;
        assert_eq!(active_alerts.len(), 1);
        
        let result = monitor.acknowledge_alert("test-alert").await;
        assert!(result.is_ok());
        
        let active_alerts = monitor.get_active_alerts().await;
        assert_eq!(active_alerts.len(), 0);
    }
}