// WezTerm Multi-Process Development Framework - Metrics Collection System
// Handles system and process metrics collection for real-time dashboard

pub mod collector;
pub mod aggregator;
pub mod storage;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// System metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Timestamp when metrics were collected
    pub timestamp: u64,
    
    /// CPU usage percentage (0.0 - 100.0)
    pub cpu_usage: f64,
    
    /// Memory usage in bytes
    pub memory_usage: u64,
    
    /// Total system memory in bytes
    pub total_memory: u64,
    
    /// Memory usage percentage (0.0 - 100.0)
    pub memory_percentage: f64,
    
    /// Disk usage in bytes
    pub disk_usage: u64,
    
    /// Total disk space in bytes
    pub total_disk: u64,
    
    /// Disk usage percentage (0.0 - 100.0)
    pub disk_percentage: f64,
    
    /// Load average (1, 5, 15 minutes)
    pub load_average: [f64; 3],
    
    /// Number of active processes
    pub process_count: u32,
    
    /// Network I/O statistics
    pub network_io: NetworkIoStats,
}

/// Process metrics for a specific managed process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    /// Process ID
    pub process_id: String,
    
    /// Workspace name
    pub workspace: String,
    
    /// Timestamp when metrics were collected
    pub timestamp: u64,
    
    /// Process status
    pub status: ProcessStatus,
    
    /// CPU usage percentage (0.0 - 100.0)
    pub cpu_usage: f64,
    
    /// Memory usage in bytes
    pub memory_usage: u64,
    
    /// Memory usage percentage (0.0 - 100.0)
    pub memory_percentage: f64,
    
    /// Process uptime in seconds
    pub uptime: u64,
    
    /// Number of threads
    pub thread_count: u32,
    
    /// File descriptor count
    pub fd_count: u32,
    
    /// Last activity timestamp
    pub last_activity: u64,
    
    /// Response time in milliseconds
    pub response_time: Option<u64>,
    
    /// Error count since last reset
    pub error_count: u32,
    
    /// Command line arguments
    pub command_args: Vec<String>,
}

/// Process status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessStatus {
    /// Process is running normally
    Running,
    
    /// Process is idle (not responding to requests)
    Idle,
    
    /// Process is busy (high CPU/memory usage)
    Busy,
    
    /// Process is not responding
    Unresponsive,
    
    /// Process has crashed or exited unexpectedly
    Failed,
    
    /// Process is starting up
    Starting,
    
    /// Process is shutting down
    Stopping,
    
    /// Process is stopped
    Stopped,
}

/// Network I/O statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIoStats {
    /// Bytes received
    pub bytes_received: u64,
    
    /// Bytes sent
    pub bytes_sent: u64,
    
    /// Packets received
    pub packets_received: u64,
    
    /// Packets sent
    pub packets_sent: u64,
    
    /// Receive rate in bytes per second
    pub rx_rate: f64,
    
    /// Transmit rate in bytes per second
    pub tx_rate: f64,
}

/// Workspace metrics aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetrics {
    /// Workspace name
    pub workspace_name: String,
    
    /// Timestamp when metrics were aggregated
    pub timestamp: u64,
    
    /// Total number of processes in workspace
    pub total_processes: u32,
    
    /// Number of running processes
    pub running_processes: u32,
    
    /// Number of failed processes
    pub failed_processes: u32,
    
    /// Average CPU usage across all processes
    pub avg_cpu_usage: f64,
    
    /// Total memory usage across all processes
    pub total_memory_usage: u64,
    
    /// Overall workspace health score (0.0 - 100.0)
    pub health_score: f64,
    
    /// Last activity timestamp
    pub last_activity: u64,
    
    /// Process metrics for individual processes
    pub processes: HashMap<String, ProcessMetrics>,
}

/// Framework-wide metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkMetrics {
    /// Timestamp when metrics were aggregated
    pub timestamp: u64,
    
    /// System metrics
    pub system: SystemMetrics,
    
    /// Workspace metrics by name
    pub workspaces: HashMap<String, WorkspaceMetrics>,
    
    /// Total number of managed processes
    pub total_processes: u32,
    
    /// Total number of workspaces
    pub total_workspaces: u32,
    
    /// Framework uptime in seconds
    pub framework_uptime: u64,
    
    /// Overall system health status
    pub overall_status: SystemHealthStatus,
    
    /// Performance summary
    pub performance: PerformanceSummary,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SystemHealthStatus {
    /// All systems operating normally
    Healthy,
    
    /// Some issues detected, but system is functional
    Warning,
    
    /// Critical issues detected, degraded performance
    Critical,
    
    /// System is not responding or has failed
    Failed,
    
    /// System is starting up
    Starting,
    
    /// System is shutting down
    Stopping,
}

/// Performance summary metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Average response time across all processes in milliseconds
    pub avg_response_time: f64,
    
    /// 95th percentile response time in milliseconds
    pub p95_response_time: f64,
    
    /// 99th percentile response time in milliseconds
    pub p99_response_time: f64,
    
    /// Total number of requests processed
    pub total_requests: u64,
    
    /// Requests per second rate
    pub requests_per_second: f64,
    
    /// Error rate percentage (0.0 - 100.0)
    pub error_rate: f64,
    
    /// Total errors count
    pub total_errors: u64,
}

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    
    /// Collection interval in seconds
    pub collection_interval: u64,
    
    /// Maximum number of historical data points to keep
    pub max_history_points: usize,
    
    /// Enable system metrics collection
    pub collect_system_metrics: bool,
    
    /// Enable process metrics collection
    pub collect_process_metrics: bool,
    
    /// Enable network metrics collection
    pub collect_network_metrics: bool,
    
    /// Metrics retention period in hours
    pub retention_hours: u64,
    
    /// Enable performance profiling
    pub enable_profiling: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: 5,
            max_history_points: 1000,
            collect_system_metrics: true,
            collect_process_metrics: true,
            collect_network_metrics: true,
            retention_hours: 24,
            enable_profiling: false,
        }
    }
}

impl SystemMetrics {
    /// Create a new empty system metrics instance
    pub fn new() -> Self {
        Self {
            timestamp: Self::current_timestamp(),
            cpu_usage: 0.0,
            memory_usage: 0,
            total_memory: 0,
            memory_percentage: 0.0,
            disk_usage: 0,
            total_disk: 0,
            disk_percentage: 0.0,
            load_average: [0.0, 0.0, 0.0],
            process_count: 0,
            network_io: NetworkIoStats::default(),
        }
    }
    
    /// Get current timestamp in seconds since Unix epoch
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

impl Default for NetworkIoStats {
    fn default() -> Self {
        Self {
            bytes_received: 0,
            bytes_sent: 0,
            packets_received: 0,
            packets_sent: 0,
            rx_rate: 0.0,
            tx_rate: 0.0,
        }
    }
}

impl ProcessMetrics {
    /// Create new process metrics instance
    pub fn new(process_id: String, workspace: String) -> Self {
        Self {
            process_id,
            workspace,
            timestamp: SystemMetrics::current_timestamp(),
            status: ProcessStatus::Starting,
            cpu_usage: 0.0,
            memory_usage: 0,
            memory_percentage: 0.0,
            uptime: 0,
            thread_count: 0,
            fd_count: 0,
            last_activity: SystemMetrics::current_timestamp(),
            response_time: None,
            error_count: 0,
            command_args: Vec::new(),
        }
    }
    
    /// Check if process is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, ProcessStatus::Running | ProcessStatus::Idle)
            && self.cpu_usage < 90.0
            && self.memory_percentage < 90.0
            && self.response_time.map_or(true, |rt| rt < 5000) // 5 second threshold
    }
    
    /// Calculate health score (0.0 - 100.0)
    pub fn health_score(&self) -> f64 {
        let mut score = 100.0;
        
        // Deduct points based on status
        match self.status {
            ProcessStatus::Running => {}
            ProcessStatus::Idle => score -= 5.0,
            ProcessStatus::Busy => score -= 15.0,
            ProcessStatus::Unresponsive => score -= 50.0,
            ProcessStatus::Failed => return 0.0,
            ProcessStatus::Starting => score -= 20.0,
            ProcessStatus::Stopping => score -= 30.0,
            ProcessStatus::Stopped => return 0.0,
        }
        
        // Deduct points for high resource usage
        if self.cpu_usage > 80.0 {
            score -= (self.cpu_usage - 80.0) * 2.0;
        }
        
        if self.memory_percentage > 80.0 {
            score -= (self.memory_percentage - 80.0) * 2.0;
        }
        
        // Deduct points for slow response times
        if let Some(rt) = self.response_time {
            if rt > 1000 { // 1 second
                score -= ((rt - 1000) as f64 / 100.0).min(30.0);
            }
        }
        
        // Deduct points for errors
        if self.error_count > 0 {
            score -= (self.error_count as f64 * 2.0).min(40.0);
        }
        
        score.max(0.0)
    }
}

impl WorkspaceMetrics {
    /// Create new workspace metrics
    pub fn new(workspace_name: String) -> Self {
        Self {
            workspace_name,
            timestamp: SystemMetrics::current_timestamp(),
            total_processes: 0,
            running_processes: 0,
            failed_processes: 0,
            avg_cpu_usage: 0.0,
            total_memory_usage: 0,
            health_score: 100.0,
            last_activity: SystemMetrics::current_timestamp(),
            processes: HashMap::new(),
        }
    }
    
    /// Update workspace metrics from process metrics
    pub fn update_from_processes(&mut self, process_metrics: Vec<ProcessMetrics>) {
        self.timestamp = SystemMetrics::current_timestamp();
        self.total_processes = process_metrics.len() as u32;
        self.running_processes = 0;
        self.failed_processes = 0;
        self.avg_cpu_usage = 0.0;
        self.total_memory_usage = 0;
        
        let mut total_cpu = 0.0;
        let mut health_scores = Vec::new();
        let mut latest_activity = 0;
        
        for metrics in process_metrics {
            // Update counters
            match metrics.status {
                ProcessStatus::Running | ProcessStatus::Idle | ProcessStatus::Busy => {
                    self.running_processes += 1;
                }
                ProcessStatus::Failed => {
                    self.failed_processes += 1;
                }
                _ => {}
            }
            
            // Accumulate resource usage
            total_cpu += metrics.cpu_usage;
            self.total_memory_usage += metrics.memory_usage;
            
            // Track health and activity
            health_scores.push(metrics.health_score());
            latest_activity = latest_activity.max(metrics.last_activity);
            
            // Store individual process metrics
            self.processes.insert(metrics.process_id.clone(), metrics);
        }
        
        // Calculate averages
        if self.total_processes > 0 {
            self.avg_cpu_usage = total_cpu / self.total_processes as f64;
            
            // Calculate overall health score
            self.health_score = if health_scores.is_empty() {
                100.0
            } else {
                health_scores.iter().sum::<f64>() / health_scores.len() as f64
            };
        }
        
        self.last_activity = latest_activity;
    }
}

impl FrameworkMetrics {
    /// Create new framework metrics
    pub fn new() -> Self {
        Self {
            timestamp: SystemMetrics::current_timestamp(),
            system: SystemMetrics::new(),
            workspaces: HashMap::new(),
            total_processes: 0,
            total_workspaces: 0,
            framework_uptime: 0,
            overall_status: SystemHealthStatus::Starting,
            performance: PerformanceSummary::default(),
        }
    }
    
    /// Update framework metrics from workspace metrics
    pub fn update_from_workspaces(&mut self, workspace_metrics: Vec<WorkspaceMetrics>) {
        self.timestamp = SystemMetrics::current_timestamp();
        self.total_workspaces = workspace_metrics.len() as u32;
        self.total_processes = 0;
        
        let mut workspace_health_scores = Vec::new();
        
        for workspace in workspace_metrics {
            self.total_processes += workspace.total_processes;
            workspace_health_scores.push(workspace.health_score);
            self.workspaces.insert(workspace.workspace_name.clone(), workspace);
        }
        
        // Determine overall system health
        if workspace_health_scores.is_empty() {
            self.overall_status = SystemHealthStatus::Healthy;
        } else {
            let avg_health = workspace_health_scores.iter().sum::<f64>() / workspace_health_scores.len() as f64;
            
            self.overall_status = if avg_health >= 90.0 {
                SystemHealthStatus::Healthy
            } else if avg_health >= 70.0 {
                SystemHealthStatus::Warning
            } else if avg_health >= 30.0 {
                SystemHealthStatus::Critical
            } else {
                SystemHealthStatus::Failed
            };
        }
    }
}

impl Default for PerformanceSummary {
    fn default() -> Self {
        Self {
            avg_response_time: 0.0,
            p95_response_time: 0.0,
            p99_response_time: 0.0,
            total_requests: 0,
            requests_per_second: 0.0,
            error_rate: 0.0,
            total_errors: 0,
        }
    }
}