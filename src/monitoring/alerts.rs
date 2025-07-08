// WezTerm Multi-Process Development Framework - Alert System
// Provides intelligent alerting and notification capabilities

use super::{Alert, AlertSeverity, AlertThresholds, SystemMetrics};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info};

/// Alert manager for processing and dispatching alerts
pub struct AlertManager {
    /// Alert thresholds configuration
    thresholds: AlertThresholds,

    /// Active alerts
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,

    /// Alert history
    alert_history: Arc<RwLock<Vec<Alert>>>,

    /// Alert notification senders
    notification_senders: Vec<Box<dyn AlertNotificationSender + Send + Sync>>,

    /// Alert evaluation state
    evaluation_state: Arc<RwLock<AlertEvaluationState>>,
}

/// Alert evaluation state for tracking trends and preventing alert spam
#[derive(Debug, Default)]
struct AlertEvaluationState {
    /// CPU usage history for trend analysis
    cpu_history: Vec<f64>,

    /// Memory usage history
    memory_history: Vec<u64>,

    /// Error count tracking
    #[allow(dead_code)]
    error_counts: HashMap<String, u32>,

    /// Last alert timestamps to prevent spam
    last_alert_times: HashMap<String, u64>,

    /// Process restart tracking
    #[allow(dead_code)]
    process_restarts: HashMap<String, u32>,
}

/// Alert notification sender trait
pub trait AlertNotificationSender {
    /// Send alert notification (sync version)
    fn send_alert_sync(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>>;

    /// Get sender name
    fn name(&self) -> &str;
}

/// Console alert notification sender
pub struct ConsoleAlertSender;

/// Log file alert notification sender
pub struct LogAlertSender {
    log_path: String,
}

/// Webhook alert notification sender
pub struct WebhookAlertSender {
    webhook_url: String,
    #[allow(dead_code)]
    client: reqwest::Client,
}

impl AlertManager {
    /// Create new alert manager
    pub fn new(thresholds: AlertThresholds) -> Self {
        Self {
            thresholds,
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            notification_senders: Vec::new(),
            evaluation_state: Arc::new(RwLock::new(AlertEvaluationState::default())),
        }
    }

    /// Add notification sender
    pub fn add_notification_sender(
        &mut self,
        sender: Box<dyn AlertNotificationSender + Send + Sync>,
    ) {
        info!("Added alert notification sender: {}", sender.name());
        self.notification_senders.push(sender);
    }

    /// Start alert processing
    pub async fn start(
        &self,
        mut metrics_rx: mpsc::Receiver<SystemMetrics>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting alert manager");

        // Start alert evaluation task
        while let Some(metrics) = metrics_rx.recv().await {
            if let Err(e) = self.evaluate_metrics(&metrics).await {
                error!("Failed to evaluate metrics for alerts: {}", e);
            }
        }

        Ok(())
    }

    /// Evaluate metrics and generate alerts
    async fn evaluate_metrics(
        &self,
        metrics: &SystemMetrics,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug!(
            "Evaluating metrics for alerts at timestamp: {}",
            metrics.timestamp
        );

        // Update evaluation state
        self.update_evaluation_state(metrics).await;

        // Check system-level alerts
        self.check_system_alerts(metrics).await?;

        // Check process-level alerts
        self.check_process_alerts(metrics).await?;

        // Check for resolved alerts
        self.check_resolved_alerts(metrics).await?;

        Ok(())
    }

    /// Update evaluation state with current metrics
    async fn update_evaluation_state(&self, metrics: &SystemMetrics) {
        let mut state = self.evaluation_state.write().await;

        // Update CPU history
        state.cpu_history.push(metrics.cpu_usage);
        if state.cpu_history.len() > 60 {
            // Keep last 60 samples
            state.cpu_history.remove(0);
        }

        // Update memory history
        state.memory_history.push(metrics.memory_usage);
        if state.memory_history.len() > 60 {
            state.memory_history.remove(0);
        }
    }

    /// Check system-level alerts
    async fn check_system_alerts(
        &self,
        metrics: &SystemMetrics,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // CPU usage alert
        if metrics.cpu_usage > self.thresholds.cpu_usage {
            self.create_alert_if_needed(
                "system_cpu_high",
                AlertSeverity::Warning,
                "System",
                format!(
                    "High CPU usage: {:.2}% (threshold: {:.2}%)",
                    metrics.cpu_usage, self.thresholds.cpu_usage
                ),
                Some("system"),
                metrics.timestamp,
                serde_json::json!({
                    "cpu_usage": metrics.cpu_usage,
                    "threshold": self.thresholds.cpu_usage
                }),
            )
            .await?;
        }

        // Memory usage alert
        let memory_usage_percentage = if metrics.memory_usage + metrics.memory_available > 0 {
            (metrics.memory_usage as f64 / (metrics.memory_usage + metrics.memory_available) as f64)
                * 100.0
        } else {
            0.0
        };

        if memory_usage_percentage > self.thresholds.memory_usage {
            self.create_alert_if_needed(
                "system_memory_high",
                AlertSeverity::Warning,
                "System",
                format!(
                    "High memory usage: {:.2}% (threshold: {:.2}%)",
                    memory_usage_percentage, self.thresholds.memory_usage
                ),
                Some("system"),
                metrics.timestamp,
                serde_json::json!({
                    "memory_usage_percentage": memory_usage_percentage,
                    "memory_usage_bytes": metrics.memory_usage,
                    "threshold": self.thresholds.memory_usage
                }),
            )
            .await?;
        }

        // Disk usage alert
        let disk_usage_percentage = if metrics.disk_usage + metrics.disk_available > 0 {
            (metrics.disk_usage as f64 / (metrics.disk_usage + metrics.disk_available) as f64)
                * 100.0
        } else {
            0.0
        };

        if disk_usage_percentage > self.thresholds.disk_usage {
            self.create_alert_if_needed(
                "system_disk_high",
                AlertSeverity::Critical,
                "System",
                format!(
                    "High disk usage: {:.2}% (threshold: {:.2}%)",
                    disk_usage_percentage, self.thresholds.disk_usage
                ),
                Some("system"),
                metrics.timestamp,
                serde_json::json!({
                    "disk_usage_percentage": disk_usage_percentage,
                    "disk_usage_bytes": metrics.disk_usage,
                    "threshold": self.thresholds.disk_usage
                }),
            )
            .await?;
        }

        Ok(())
    }

    /// Check process-level alerts
    async fn check_process_alerts(
        &self,
        metrics: &SystemMetrics,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (process_name, process_metrics) in &metrics.process_metrics {
            // Process restart count alert
            if process_metrics.restart_count > self.thresholds.restart_count {
                self.create_alert_if_needed(
                    &format!("process_restart_{process_name}"),
                    AlertSeverity::Error,
                    "Process",
                    format!(
                        "Process {} has restarted {} times (threshold: {})",
                        process_name, process_metrics.restart_count, self.thresholds.restart_count
                    ),
                    Some(process_name),
                    metrics.timestamp,
                    serde_json::json!({
                        "process_name": process_name,
                        "restart_count": process_metrics.restart_count,
                        "threshold": self.thresholds.restart_count
                    }),
                )
                .await?;
            }

            // Process failure alert
            if process_metrics.status == super::ProcessStatus::Failed {
                self.create_alert_if_needed(
                    &format!("process_failed_{process_name}"),
                    AlertSeverity::Critical,
                    "Process",
                    format!("Process {process_name} has failed"),
                    Some(process_name),
                    metrics.timestamp,
                    serde_json::json!({
                        "process_name": process_name,
                        "pid": process_metrics.pid,
                        "status": "failed"
                    }),
                )
                .await?;
            }

            // High process CPU usage
            if process_metrics.cpu_usage > 80.0 {
                // Process-specific threshold
                self.create_alert_if_needed(
                    &format!("process_cpu_high_{process_name}"),
                    AlertSeverity::Warning,
                    "Process",
                    format!(
                        "Process {} high CPU usage: {:.2}%",
                        process_name, process_metrics.cpu_usage
                    ),
                    Some(process_name),
                    metrics.timestamp,
                    serde_json::json!({
                        "process_name": process_name,
                        "cpu_usage": process_metrics.cpu_usage,
                        "pid": process_metrics.pid
                    }),
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Check for resolved alerts
    async fn check_resolved_alerts(
        &self,
        metrics: &SystemMetrics,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut active_alerts = self.active_alerts.write().await;
        let mut resolved_alerts = Vec::new();

        for (alert_id, alert) in active_alerts.iter() {
            let mut should_resolve = false;

            match alert.category.as_str() {
                "System" => {
                    if alert_id == "system_cpu_high"
                        && metrics.cpu_usage <= self.thresholds.cpu_usage * 0.9
                    {
                        should_resolve = true;
                    } else if alert_id == "system_memory_high" {
                        let memory_usage_percentage =
                            if metrics.memory_usage + metrics.memory_available > 0 {
                                (metrics.memory_usage as f64
                                    / (metrics.memory_usage + metrics.memory_available) as f64)
                                    * 100.0
                            } else {
                                0.0
                            };
                        if memory_usage_percentage <= self.thresholds.memory_usage * 0.9 {
                            should_resolve = true;
                        }
                    } else if alert_id == "system_disk_high" {
                        let disk_usage_percentage =
                            if metrics.disk_usage + metrics.disk_available > 0 {
                                (metrics.disk_usage as f64
                                    / (metrics.disk_usage + metrics.disk_available) as f64)
                                    * 100.0
                            } else {
                                0.0
                            };
                        if disk_usage_percentage <= self.thresholds.disk_usage * 0.9 {
                            should_resolve = true;
                        }
                    }
                }
                "Process" => {
                    // Check if process is now healthy
                    for (process_name, process_metrics) in &metrics.process_metrics {
                        if alert_id.contains(process_name)
                            && ((alert_id.contains("failed")
                                && process_metrics.status == super::ProcessStatus::Running)
                                || (alert_id.contains("cpu_high")
                                    && process_metrics.cpu_usage <= 70.0))
                        {
                            should_resolve = true;
                        }
                    }
                }
                _ => {}
            }

            if should_resolve {
                resolved_alerts.push(alert_id.clone());
            }
        }

        // Resolve alerts
        for alert_id in resolved_alerts {
            if let Some(mut alert) = active_alerts.remove(&alert_id) {
                alert.resolved = true;
                alert.resolved_at = Some(metrics.timestamp);

                info!("Resolved alert: {}", alert.message);

                // Add to history
                let mut history = self.alert_history.write().await;
                history.push(alert.clone());

                // Send resolution notification
                self.send_alert_notification(&alert).await;
            }
        }

        Ok(())
    }

    /// Create alert if needed (prevents spam)
    #[allow(clippy::too_many_arguments)]
    async fn create_alert_if_needed(
        &self,
        alert_id: &str,
        severity: AlertSeverity,
        category: &str,
        message: String,
        component: Option<&str>,
        timestamp: u64,
        data: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.evaluation_state.write().await;

        // Check if we recently sent this alert (prevent spam)
        if let Some(&last_time) = state.last_alert_times.get(alert_id) {
            if timestamp - last_time < 300 {
                // 5 minutes cooldown
                return Ok(());
            }
        }

        // Check if alert already exists
        let active_alerts = self.active_alerts.read().await;
        if active_alerts.contains_key(alert_id) {
            return Ok(());
        }
        drop(active_alerts);

        // Create new alert
        let alert = Alert {
            id: alert_id.to_string(),
            severity,
            category: category.to_string(),
            message,
            component: component.map(|s| s.to_string()),
            timestamp,
            data: {
                let mut map = HashMap::new();
                if let serde_json::Value::Object(obj) = data {
                    for (k, v) in obj {
                        map.insert(k, v);
                    }
                }
                map
            },
            resolved: false,
            resolved_at: None,
        };

        info!("Created alert: {} - {}", alert.severity, alert.message);

        // Add to active alerts
        let mut active_alerts = self.active_alerts.write().await;
        active_alerts.insert(alert_id.to_string(), alert.clone());
        drop(active_alerts);

        // Update last alert time
        state
            .last_alert_times
            .insert(alert_id.to_string(), timestamp);
        drop(state);

        // Add to history
        let mut history = self.alert_history.write().await;
        history.push(alert.clone());

        // Limit history size
        if history.len() > 1000 {
            history.drain(0..100);
        }
        drop(history);

        // Send notification
        self.send_alert_notification(&alert).await;

        Ok(())
    }

    /// Send alert notification through all configured senders
    async fn send_alert_notification(&self, alert: &Alert) {
        for sender in &self.notification_senders {
            if let Err(e) = sender.send_alert_sync(alert) {
                error!(
                    "Failed to send alert notification via {}: {}",
                    sender.name(),
                    e
                );
            }
        }
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.active_alerts.read().await;
        alerts.values().cloned().collect()
    }

    /// Get alert history
    pub async fn get_alert_history(&self, limit: Option<usize>) -> Vec<Alert> {
        let history = self.alert_history.read().await;
        let limit = limit.unwrap_or(history.len());
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Manually resolve alert
    pub async fn resolve_alert(&self, alert_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut active_alerts = self.active_alerts.write().await;
        if let Some(mut alert) = active_alerts.remove(alert_id) {
            alert.resolved = true;
            alert.resolved_at = Some(crate::monitoring::utils::current_timestamp());

            info!("Manually resolved alert: {}", alert.message);

            let mut history = self.alert_history.write().await;
            history.push(alert.clone());

            self.send_alert_notification(&alert).await;
        }

        Ok(())
    }
}

impl AlertNotificationSender for ConsoleAlertSender {
    fn send_alert_sync(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
        let status = if alert.resolved { "RESOLVED" } else { "ACTIVE" };
        let severity_icon = match alert.severity {
            AlertSeverity::Info => "ℹ️",
            AlertSeverity::Warning => "⚠️",
            AlertSeverity::Error => "❌",
            AlertSeverity::Critical => "🚨",
        };

        println!(
            "{} [{}] {} - {}",
            severity_icon, status, alert.category, alert.message
        );

        Ok(())
    }

    fn name(&self) -> &str {
        "console"
    }
}

impl LogAlertSender {
    pub fn new(log_path: String) -> Self {
        Self { log_path }
    }
}

impl AlertNotificationSender for LogAlertSender {
    fn send_alert_sync(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
        let alert_json = serde_json::to_string(alert)?;
        std::fs::write(&self.log_path, format!("{alert_json}\n"))?;
        Ok(())
    }

    fn name(&self) -> &str {
        "log_file"
    }
}

impl WebhookAlertSender {
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url,
            client: reqwest::Client::new(),
        }
    }
}

impl AlertNotificationSender for WebhookAlertSender {
    fn send_alert_sync(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
        // For sync implementation, we'll skip the actual HTTP call
        // In a real implementation, you'd use a blocking HTTP client
        tracing::info!(
            "Would send webhook alert to {}: {}",
            self.webhook_url,
            alert.message
        );
        Ok(())
    }

    fn name(&self) -> &str {
        "webhook"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_manager_creation() {
        let thresholds = AlertThresholds::default();
        let manager = AlertManager::new(thresholds);

        let active_alerts = manager.get_active_alerts().await;
        assert!(active_alerts.is_empty());
    }

    #[tokio::test]
    async fn test_console_alert_sender() {
        let sender = ConsoleAlertSender;
        let alert = Alert {
            id: "test".to_string(),
            severity: AlertSeverity::Warning,
            category: "Test".to_string(),
            message: "Test alert".to_string(),
            component: None,
            timestamp: 1234567890,
            data: HashMap::new(),
            resolved: false,
            resolved_at: None,
        };

        assert!(sender.send_alert_sync(&alert).is_ok());
    }

    #[tokio::test]
    async fn test_alert_spam_prevention() {
        let thresholds = AlertThresholds::default();
        let manager = AlertManager::new(thresholds);

        let timestamp = 1234567890;

        // Create first alert
        manager
            .create_alert_if_needed(
                "test_alert",
                AlertSeverity::Warning,
                "Test",
                "Test message".to_string(),
                None,
                timestamp,
                serde_json::json!({}),
            )
            .await
            .unwrap();

        // Try to create same alert immediately (should be prevented)
        manager
            .create_alert_if_needed(
                "test_alert",
                AlertSeverity::Warning,
                "Test",
                "Test message".to_string(),
                None,
                timestamp + 60, // 1 minute later
                serde_json::json!({}),
            )
            .await
            .unwrap();

        let active_alerts = manager.get_active_alerts().await;
        assert_eq!(active_alerts.len(), 1); // Should still be only 1 alert
    }
}
