// WezTerm Multi-Process Development Framework - Monitoring Module
// Provides comprehensive monitoring, logging, and alerting capabilities

pub mod alerts;
pub mod analytics;
pub mod health;
pub mod logger;
pub mod metrics;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Monitoring system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable monitoring system
    pub enabled: bool,

    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,

    /// Log format (json, pretty, compact)
    pub log_format: LogFormat,

    /// Log output destination
    pub log_output: LogOutput,

    /// Metrics collection interval in seconds
    pub metrics_interval: u64,

    /// Health check interval in seconds
    pub health_check_interval: u64,

    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,

    /// Enable log rotation
    pub log_rotation: bool,

    /// Maximum log file size in MB
    pub max_log_size_mb: u64,

    /// Number of log files to retain
    pub log_retention_count: u32,
}

/// Log format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    /// JSON structured format
    Json,
    /// Human-readable pretty format
    Pretty,
    /// Compact single-line format
    Compact,
}

/// Log output destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    /// Console output
    Console,
    /// File output
    File { path: String },
    /// Both console and file
    Both { path: String },
    /// Syslog integration
    Syslog,
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// CPU usage threshold (percentage)
    pub cpu_usage: f64,

    /// Memory usage threshold (percentage)
    pub memory_usage: f64,

    /// Disk usage threshold (percentage)
    pub disk_usage: f64,

    /// Process restart count threshold
    pub restart_count: u32,

    /// Error rate threshold (per minute)
    pub error_rate: u32,

    /// Response time threshold (milliseconds)
    pub response_time_ms: u64,
}

/// System metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Timestamp of the metrics
    pub timestamp: u64,

    /// CPU usage percentage
    pub cpu_usage: f64,

    /// Memory usage in bytes
    pub memory_usage: u64,

    /// Available memory in bytes
    pub memory_available: u64,

    /// Disk usage in bytes
    pub disk_usage: u64,

    /// Available disk space in bytes
    pub disk_available: u64,

    /// Network I/O statistics
    pub network_io: NetworkIO,

    /// Process-specific metrics
    pub process_metrics: HashMap<String, ProcessMetrics>,
}

/// Network I/O statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIO {
    /// Bytes received
    pub bytes_received: u64,

    /// Bytes sent
    pub bytes_sent: u64,

    /// Packets received
    pub packets_received: u64,

    /// Packets sent
    pub packets_sent: u64,
}

/// Process-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    /// Process ID
    pub pid: u32,

    /// Process name
    pub name: String,

    /// CPU usage percentage
    pub cpu_usage: f64,

    /// Memory usage in bytes
    pub memory_usage: u64,

    /// Number of threads
    pub thread_count: u32,

    /// File descriptor count
    pub fd_count: u32,

    /// Process uptime in seconds
    pub uptime: u64,

    /// Process status
    pub status: ProcessStatus,

    /// Number of restarts
    pub restart_count: u32,
}

/// Process status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessStatus {
    /// Process is running normally
    Running,
    /// Process is starting up
    Starting,
    /// Process is stopping
    Stopping,
    /// Process has stopped
    Stopped,
    /// Process crashed or failed
    Failed,
    /// Process is being restarted
    Restarting,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Error alert
    Error,
    /// Critical alert requiring immediate attention
    Critical,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "INFO"),
            AlertSeverity::Warning => write!(f, "WARNING"),
            AlertSeverity::Error => write!(f, "ERROR"),
            AlertSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Alert notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique alert ID
    pub id: String,

    /// Alert severity
    pub severity: AlertSeverity,

    /// Alert category
    pub category: String,

    /// Alert message
    pub message: String,

    /// Affected component
    pub component: Option<String>,

    /// Alert timestamp
    pub timestamp: u64,

    /// Additional alert data
    pub data: HashMap<String, serde_json::Value>,

    /// Whether alert is resolved
    pub resolved: bool,

    /// Resolution timestamp
    pub resolved_at: Option<u64>,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Check timestamp
    pub timestamp: u64,

    /// Overall system health
    pub overall_status: HealthStatus,

    /// Component health checks
    pub components: HashMap<String, ComponentHealth>,

    /// Health check duration in milliseconds
    pub check_duration_ms: u64,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System has minor issues
    Degraded,
    /// System has significant issues
    Unhealthy,
    /// Health check failed
    Unknown,
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component status
    pub status: HealthStatus,

    /// Component check message
    pub message: String,

    /// Last successful check timestamp
    pub last_success: Option<u64>,

    /// Number of consecutive failures
    pub failure_count: u32,

    /// Component response time in milliseconds
    pub response_time_ms: u64,
}

/// Monitoring system manager
pub struct MonitoringManager {
    /// Configuration
    config: MonitoringConfig,

    /// Current system metrics
    current_metrics: Arc<RwLock<Option<SystemMetrics>>>,

    /// Active alerts
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,

    /// Health check results
    health_status: Arc<RwLock<Option<HealthCheck>>>,

    /// Metrics history for analytics
    #[allow(dead_code)]
    metrics_history: Arc<RwLock<Vec<SystemMetrics>>>,

    /// Alert history
    alert_history: Arc<RwLock<Vec<Alert>>>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: "info".to_string(),
            log_format: LogFormat::Json,
            log_output: LogOutput::Console,
            metrics_interval: 30,
            health_check_interval: 60,
            alert_thresholds: AlertThresholds::default(),
            log_rotation: true,
            max_log_size_mb: 100,
            log_retention_count: 10,
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_usage: 80.0,
            memory_usage: 85.0,
            disk_usage: 90.0,
            restart_count: 5,
            error_rate: 10,
            response_time_ms: 5000,
        }
    }
}

impl MonitoringManager {
    /// Create new monitoring manager
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            current_metrics: Arc::new(RwLock::new(None)),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            health_status: Arc::new(RwLock::new(None)),
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start monitoring system
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.enabled {
            tracing::info!("Monitoring system is disabled");
            return Ok(());
        }

        tracing::info!("Starting monitoring system");

        // Initialize logger
        self.initialize_logger().await?;

        // Start metrics collection
        self.start_metrics_collection().await?;

        // Start health checks
        self.start_health_checks().await?;

        // Start alert processing
        self.start_alert_processing().await?;

        tracing::info!("Monitoring system started successfully");
        Ok(())
    }

    /// Initialize logging system
    async fn initialize_logger(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize structured logging based on configuration
        // This will be implemented in logger.rs
        Ok(())
    }

    /// Start metrics collection task
    async fn start_metrics_collection(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Start background task for metrics collection
        // This will be implemented in metrics.rs
        Ok(())
    }

    /// Start health check task
    async fn start_health_checks(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Start background health check task
        // This will be implemented in health.rs
        Ok(())
    }

    /// Start alert processing task
    async fn start_alert_processing(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Start alert processing and notification
        // This will be implemented in alerts.rs
        Ok(())
    }

    /// Get current system metrics
    pub async fn get_current_metrics(&self) -> Option<SystemMetrics> {
        let metrics = self.current_metrics.read().await;
        metrics.clone()
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.active_alerts.read().await;
        alerts.values().cloned().collect()
    }

    /// Get health status
    pub async fn get_health_status(&self) -> Option<HealthCheck> {
        let health = self.health_status.read().await;
        health.clone()
    }

    /// Create manual alert
    pub async fn create_alert(&self, alert: Alert) {
        let mut alerts = self.active_alerts.write().await;
        let mut history = self.alert_history.write().await;

        alerts.insert(alert.id.clone(), alert.clone());
        history.push(alert);

        // Limit history size
        if history.len() > 1000 {
            history.drain(0..100);
        }
    }

    /// Resolve alert
    pub async fn resolve_alert(&self, alert_id: &str) {
        let mut alerts = self.active_alerts.write().await;
        if let Some(mut alert) = alerts.remove(alert_id) {
            alert.resolved = true;
            alert.resolved_at = Some(utils::current_timestamp());

            let mut history = self.alert_history.write().await;
            history.push(alert);
        }
    }
}

/// Utility functions
pub mod utils {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| {
                log::warn!("System time error in monitoring, using fallback timestamp");
                std::time::Duration::from_secs(0)
            })
            .as_secs()
    }
}

// Re-export public types from submodules
pub use alerts::{AlertManager, AlertNotificationSender, ConsoleAlertSender};
pub use analytics::{AnalyticsManager, AnalyticsReport};
pub use health::HealthCheckManager;
pub use logger::{LogEntry, LogStats, LoggingManager};
pub use metrics::MetricsCollector;
