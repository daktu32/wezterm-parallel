// Metrics collection implementation for system and process monitoring

use super::{
    SystemMetrics, ProcessMetrics, ProcessStatus, NetworkIoStats, MetricsConfig,
};
use log::{info, warn, debug};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::{interval, Interval};
use sysinfo::{System, SystemExt, ProcessExt, CpuExt, DiskExt, NetworkExt};

/// System metrics collector
pub struct MetricsCollector {
    /// System information instance
    system: System,
    
    /// Collection configuration
    config: MetricsConfig,
    
    /// Collection interval timer
    interval: Interval,
    
    /// Managed process PIDs
    managed_processes: HashMap<String, u32>,
    
    /// Previous network stats for rate calculation
    previous_network_stats: Option<NetworkIoStats>,
    
    /// Collection start time for uptime calculation
    start_time: SystemTime,
}

/// Process information for metrics collection
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub process_id: String,
    pub workspace: String,
    pub pid: u32,
    pub command_args: Vec<String>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsConfig) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        let interval = interval(Duration::from_secs(config.collection_interval));
        
        Self {
            system,
            config,
            interval,
            managed_processes: HashMap::new(),
            previous_network_stats: None,
            start_time: SystemTime::now(),
        }
    }
    
    /// Register a managed process for monitoring
    pub fn register_process(&mut self, process_info: ProcessInfo) {
        debug!("Registering process for metrics collection: {}", process_info.process_id);
        self.managed_processes.insert(process_info.process_id, process_info.pid);
    }
    
    /// Unregister a managed process
    pub fn unregister_process(&mut self, process_id: &str) {
        debug!("Unregistering process from metrics collection: {}", process_id);
        self.managed_processes.remove(process_id);
    }
    
    /// Collect system metrics
    pub fn collect_system_metrics(&mut self) -> Result<SystemMetrics, String> {
        if !self.config.collect_system_metrics {
            return Ok(SystemMetrics::new());
        }
        
        debug!("Collecting system metrics");
        
        // Refresh system information
        self.system.refresh_all();
        
        // Calculate CPU usage
        let cpu_usage = self.system.global_cpu_info().cpu_usage() as f64;
        
        // Get memory information
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let memory_percentage = if total_memory > 0 {
            (used_memory as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };
        
        // Get disk information
        let (disk_usage, total_disk, disk_percentage) = self.collect_disk_metrics();
        
        // Get load average
        let load_average = self.collect_load_average();
        
        // Get process count
        let process_count = self.system.processes().len() as u32;
        
        // Get network I/O statistics
        let network_io = self.collect_network_metrics();
        
        Ok(SystemMetrics {
            timestamp: Self::current_timestamp(),
            cpu_usage,
            memory_usage: used_memory,
            total_memory,
            memory_percentage,
            disk_usage,
            total_disk,
            disk_percentage,
            load_average,
            process_count,
            network_io,
        })
    }
    
    /// Collect metrics for all managed processes
    pub fn collect_process_metrics(&mut self) -> Result<Vec<ProcessMetrics>, String> {
        if !self.config.collect_process_metrics {
            return Ok(Vec::new());
        }
        
        debug!("Collecting process metrics for {} processes", self.managed_processes.len());
        
        let mut process_metrics = Vec::new();
        
        // Refresh process information
        self.system.refresh_processes();
        
        for (process_id, &pid) in &self.managed_processes {
            match self.collect_single_process_metrics(process_id, pid) {
                Ok(metrics) => process_metrics.push(metrics),
                Err(err) => {
                    warn!("Failed to collect metrics for process {}: {}", process_id, err);
                }
            }
        }
        
        Ok(process_metrics)
    }
    
    /// Collect metrics for a single process
    fn collect_single_process_metrics(&self, process_id: &str, pid: u32) -> Result<ProcessMetrics, String> {
        let process = self.system.process(sysinfo::Pid::from(pid as usize))
            .ok_or_else(|| format!("Process {} (PID: {}) not found", process_id, pid))?;
        
        // Basic process information
        let cpu_usage = process.cpu_usage() as f64;
        let memory_usage = process.memory();
        let memory_percentage = if self.system.total_memory() > 0 {
            (memory_usage as f64 / self.system.total_memory() as f64) * 100.0
        } else {
            0.0
        };
        
        // Process uptime calculation
        let uptime = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_sub(process.start_time());
        
        // Thread and file descriptor counts  
        let thread_count = 1; // sysinfo doesn't provide task count directly
        let fd_count = self.get_fd_count(pid).unwrap_or(0);
        
        // Determine process status
        let status = self.determine_process_status(cpu_usage, memory_percentage, &process);
        
        // Get command line arguments
        let command_args = process.cmd().to_vec();
        
        // Response time (would need integration with process monitoring)
        let response_time = self.measure_process_response_time(&process_id);
        
        Ok(ProcessMetrics {
            process_id: process_id.to_string(),
            workspace: self.get_process_workspace(process_id),
            timestamp: Self::current_timestamp(),
            status,
            cpu_usage,
            memory_usage,
            memory_percentage,
            uptime,
            thread_count,
            fd_count,
            last_activity: Self::current_timestamp(), // Would need proper activity tracking
            response_time,
            error_count: 0, // Would need error tracking integration
            command_args,
        })
    }
    
    /// Collect disk metrics
    fn collect_disk_metrics(&self) -> (u64, u64, f64) {
        let mut total_used = 0;
        let mut total_space = 0;
        
        for disk in self.system.disks() {
            total_used += disk.total_space() - disk.available_space();
            total_space += disk.total_space();
        }
        
        let disk_percentage = if total_space > 0 {
            (total_used as f64 / total_space as f64) * 100.0
        } else {
            0.0
        };
        
        (total_used, total_space, disk_percentage)
    }
    
    /// Collect load average (Unix systems only)
    fn collect_load_average(&self) -> [f64; 3] {
        #[cfg(unix)]
        {
            match self.system.load_average() {
                sysinfo::LoadAvg { one, five, fifteen } => [one, five, fifteen],
            }
        }
        
        #[cfg(not(unix))]
        {
            [0.0, 0.0, 0.0]
        }
    }
    
    /// Collect network I/O metrics
    fn collect_network_metrics(&mut self) -> NetworkIoStats {
        if !self.config.collect_network_metrics {
            return NetworkIoStats::default();
        }
        
        let mut total_rx = 0;
        let mut total_tx = 0;
        let mut total_rx_packets = 0;
        let mut total_tx_packets = 0;
        
        // Sum up all network interfaces
        for (_name, network) in self.system.networks() {
            total_rx += network.received();
            total_tx += network.transmitted();
            total_rx_packets += network.packets_received();
            total_tx_packets += network.packets_transmitted();
        }
        
        // Calculate rates if we have previous data
        let (rx_rate, tx_rate) = if let Some(ref prev) = self.previous_network_stats {
            let time_diff = self.config.collection_interval as f64;
            let rx_diff = total_rx.saturating_sub(prev.bytes_received);
            let tx_diff = total_tx.saturating_sub(prev.bytes_sent);
            
            (rx_diff as f64 / time_diff, tx_diff as f64 / time_diff)
        } else {
            (0.0, 0.0)
        };
        
        let stats = NetworkIoStats {
            bytes_received: total_rx,
            bytes_sent: total_tx,
            packets_received: total_rx_packets,
            packets_sent: total_tx_packets,
            rx_rate,
            tx_rate,
        };
        
        // Store for next rate calculation
        self.previous_network_stats = Some(stats.clone());
        
        stats
    }
    
    /// Get file descriptor count for a process
    fn get_fd_count(&self, pid: u32) -> Option<u32> {
        #[cfg(unix)]
        {
            use std::fs;
            
            match fs::read_dir(format!("/proc/{}/fd", pid)) {
                Ok(entries) => Some(entries.count() as u32),
                Err(_) => None,
            }
        }
        
        #[cfg(not(unix))]
        {
            None
        }
    }
    
    /// Determine process status based on metrics
    fn determine_process_status(&self, cpu_usage: f64, memory_percentage: f64, process: &sysinfo::Process) -> ProcessStatus {
        // Check if process is running
        if process.status() != sysinfo::ProcessStatus::Run {
            return match process.status() {
                sysinfo::ProcessStatus::Sleep => ProcessStatus::Idle,
                sysinfo::ProcessStatus::Stop => ProcessStatus::Stopped,
                sysinfo::ProcessStatus::Zombie => ProcessStatus::Failed,
                _ => ProcessStatus::Unresponsive,
            };
        }
        
        // Determine status based on resource usage
        if cpu_usage > 80.0 || memory_percentage > 80.0 {
            ProcessStatus::Busy
        } else if cpu_usage < 5.0 {
            ProcessStatus::Idle
        } else {
            ProcessStatus::Running
        }
    }
    
    /// Get workspace for a process (placeholder - would need integration)
    fn get_process_workspace(&self, process_id: &str) -> String {
        // This would need integration with workspace management
        // For now, extract from process_id if it follows a pattern
        if let Some(pos) = process_id.find('-') {
            process_id[..pos].to_string()
        } else {
            "default".to_string()
        }
    }
    
    /// Measure process response time (placeholder)
    fn measure_process_response_time(&self, _process_id: &str) -> Option<u64> {
        // This would need integration with process monitoring
        // For now, return None
        None
    }
    
    /// Wait for next collection interval
    pub async fn wait_for_next_collection(&mut self) {
        self.interval.tick().await;
    }
    
    /// Get current timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
    
    /// Update collection configuration
    pub fn update_config(&mut self, config: MetricsConfig) {
        info!("Updating metrics collection configuration");
        
        // Update interval if changed
        if config.collection_interval != self.config.collection_interval {
            self.interval = interval(Duration::from_secs(config.collection_interval));
        }
        
        self.config = config;
    }
    
    /// Get collection statistics
    pub fn get_collection_stats(&self) -> CollectionStats {
        CollectionStats {
            managed_processes_count: self.managed_processes.len(),
            collection_interval: self.config.collection_interval,
            uptime: self.start_time.elapsed().unwrap_or_default().as_secs(),
            config: self.config.clone(),
        }
    }
}

/// Metrics collection statistics
#[derive(Debug, Clone)]
pub struct CollectionStats {
    pub managed_processes_count: usize,
    pub collection_interval: u64,
    pub uptime: u64,
    pub config: MetricsConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config.clone());
        
        assert_eq!(collector.config.collection_interval, config.collection_interval);
        assert!(collector.managed_processes.is_empty());
    }
    
    #[tokio::test]
    async fn test_process_registration() {
        let config = MetricsConfig::default();
        let mut collector = MetricsCollector::new(config);
        
        let process_info = ProcessInfo {
            process_id: "test-process".to_string(),
            workspace: "test-workspace".to_string(),
            pid: 1234,
            command_args: vec!["test".to_string()],
        };
        
        collector.register_process(process_info);
        assert_eq!(collector.managed_processes.len(), 1);
        assert_eq!(collector.managed_processes.get("test-process"), Some(&1234));
        
        collector.unregister_process("test-process");
        assert!(collector.managed_processes.is_empty());
    }
    
    #[tokio::test]
    async fn test_system_metrics_collection() {
        let config = MetricsConfig::default();
        let mut collector = MetricsCollector::new(config);
        
        let metrics = collector.collect_system_metrics().unwrap();
        
        assert!(metrics.timestamp > 0);
        assert!(metrics.total_memory > 0);
        assert!(metrics.memory_percentage >= 0.0);
        assert!(metrics.memory_percentage <= 100.0);
    }
}