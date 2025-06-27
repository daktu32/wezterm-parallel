// WezTerm Multi-Process Development Framework - Metrics Collection
// Provides comprehensive system and process metrics collection

use super::{SystemMetrics, ProcessMetrics, NetworkIO, ProcessStatus};
use std::collections::HashMap;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

/// Metrics collector for system and process data
pub struct MetricsCollector {
    /// Process metrics cache
    process_cache: HashMap<u32, ProcessMetrics>,
    
    /// Network baseline for delta calculations
    network_baseline: Option<NetworkIO>,
    
    /// Last collection timestamp
    last_collection: Option<u64>,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            process_cache: HashMap::new(),
            network_baseline: None,
            last_collection: None,
        }
    }
    
    /// Collect comprehensive system metrics
    pub async fn collect_metrics(&mut self) -> Result<SystemMetrics, Box<dyn std::error::Error>> {
        let timestamp = current_timestamp();
        
        debug!("Collecting system metrics at timestamp: {}", timestamp);
        
        // Collect CPU metrics
        let cpu_usage = self.collect_cpu_usage().await?;
        
        // Collect memory metrics
        let (memory_usage, memory_available) = self.collect_memory_metrics().await?;
        
        // Collect disk metrics
        let (disk_usage, disk_available) = self.collect_disk_metrics().await?;
        
        // Collect network metrics
        let network_io = self.collect_network_metrics().await?;
        
        // Collect process metrics
        let process_metrics = self.collect_process_metrics().await?;
        
        let metrics = SystemMetrics {
            timestamp,
            cpu_usage,
            memory_usage,
            memory_available,
            disk_usage,
            disk_available,
            network_io,
            process_metrics: process_metrics.clone(),
        };
        
        self.last_collection = Some(timestamp);
        
        debug!("Collected metrics: CPU={:.2}%, Memory={:.2}MB, Processes={}", 
               cpu_usage, memory_usage as f64 / 1024.0 / 1024.0, process_metrics.len());
        
        Ok(metrics)
    }
    
    /// Collect CPU usage percentage
    async fn collect_cpu_usage(&self) -> Result<f64, Box<dyn std::error::Error>> {
        // Platform-specific CPU usage collection
        #[cfg(target_os = "macos")]
        {
            self.collect_cpu_usage_macos().await
        }
        
        #[cfg(target_os = "linux")]
        {
            self.collect_cpu_usage_linux().await
        }
        
        #[cfg(target_os = "windows")]
        {
            self.collect_cpu_usage_windows().await
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            warn!("CPU usage collection not implemented for this platform");
            Ok(0.0)
        }
    }
    
    #[cfg(target_os = "macos")]
    async fn collect_cpu_usage_macos(&self) -> Result<f64, Box<dyn std::error::Error>> {
        let output = Command::new("top")
            .args(&["-l", "1", "-s", "0", "-n", "0"])
            .output()?;
        
        let output_str = String::from_utf8(output.stdout)?;
        
        // Parse CPU usage from top output
        for line in output_str.lines() {
            if line.contains("CPU usage:") {
                // Example: "CPU usage: 12.34% user, 5.67% sys, 81.99% idle"
                if let Some(idle_start) = line.find("% idle") {
                    if let Some(comma_pos) = line[..idle_start].rfind(',') {
                        let idle_str = line[comma_pos + 1..idle_start].trim();
                        if let Ok(idle_percentage) = idle_str.parse::<f64>() {
                            return Ok(100.0 - idle_percentage);
                        }
                    }
                }
            }
        }
        
        warn!("Failed to parse CPU usage from top output");
        Ok(0.0)
    }
    
    #[cfg(target_os = "linux")]
    async fn collect_cpu_usage_linux(&self) -> Result<f64, Box<dyn std::error::Error>> {
        let stat_content = tokio::fs::read_to_string("/proc/stat").await?;
        
        if let Some(first_line) = stat_content.lines().next() {
            let fields: Vec<&str> = first_line.split_whitespace().collect();
            if fields.len() >= 8 && fields[0] == "cpu" {
                let user: u64 = fields[1].parse()?;
                let nice: u64 = fields[2].parse()?;
                let system: u64 = fields[3].parse()?;
                let idle: u64 = fields[4].parse()?;
                let iowait: u64 = fields[5].parse()?;
                let irq: u64 = fields[6].parse()?;
                let softirq: u64 = fields[7].parse()?;
                
                let total = user + nice + system + idle + iowait + irq + softirq;
                let usage = if total > 0 {
                    ((total - idle) as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                
                return Ok(usage);
            }
        }
        
        warn!("Failed to parse CPU usage from /proc/stat");
        Ok(0.0)
    }
    
    #[cfg(target_os = "windows")]
    async fn collect_cpu_usage_windows(&self) -> Result<f64, Box<dyn std::error::Error>> {
        let output = Command::new("wmic")
            .args(&["cpu", "get", "loadpercentage", "/value"])
            .output()?;
        
        let output_str = String::from_utf8(output.stdout)?;
        
        for line in output_str.lines() {
            if line.starts_with("LoadPercentage=") {
                let percentage_str = line.trim_start_matches("LoadPercentage=");
                if let Ok(percentage) = percentage_str.parse::<f64>() {
                    return Ok(percentage);
                }
            }
        }
        
        warn!("Failed to parse CPU usage from wmic output");
        Ok(0.0)
    }
    
    /// Collect memory metrics
    async fn collect_memory_metrics(&self) -> Result<(u64, u64), Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        {
            self.collect_memory_metrics_macos().await
        }
        
        #[cfg(target_os = "linux")]
        {
            self.collect_memory_metrics_linux().await
        }
        
        #[cfg(target_os = "windows")]
        {
            self.collect_memory_metrics_windows().await
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            warn!("Memory metrics collection not implemented for this platform");
            Ok((0, 0))
        }
    }
    
    #[cfg(target_os = "macos")]
    async fn collect_memory_metrics_macos(&self) -> Result<(u64, u64), Box<dyn std::error::Error>> {
        let output = Command::new("vm_stat").output()?;
        let output_str = String::from_utf8(output.stdout)?;
        
        let mut page_size = 4096u64; // Default page size
        let mut pages_free = 0u64;
        let mut pages_wired = 0u64;
        let mut pages_active = 0u64;
        let mut pages_inactive = 0u64;
        
        for line in output_str.lines() {
            if line.contains("page size of") {
                if let Some(size_start) = line.find("page size of ") {
                    let size_part = &line[size_start + 13..];
                    if let Some(size_end) = size_part.find(" ") {
                        if let Ok(size) = size_part[..size_end].parse::<u64>() {
                            page_size = size;
                        }
                    }
                }
            } else if line.contains("Pages free:") {
                pages_free = self.extract_pages_from_line(line);
            } else if line.contains("Pages wired down:") {
                pages_wired = self.extract_pages_from_line(line);
            } else if line.contains("Pages active:") {
                pages_active = self.extract_pages_from_line(line);
            } else if line.contains("Pages inactive:") {
                pages_inactive = self.extract_pages_from_line(line);
            }
        }
        
        let used_memory = (pages_wired + pages_active + pages_inactive) * page_size;
        let available_memory = pages_free * page_size;
        
        Ok((used_memory, available_memory))
    }
    
    #[cfg(target_os = "linux")]
    async fn collect_memory_metrics_linux(&self) -> Result<(u64, u64), Box<dyn std::error::Error>> {
        let meminfo_content = tokio::fs::read_to_string("/proc/meminfo").await?;
        
        let mut mem_total = 0u64;
        let mut mem_available = 0u64;
        
        for line in meminfo_content.lines() {
            if line.starts_with("MemTotal:") {
                mem_total = self.extract_memory_from_line(line)?;
            } else if line.starts_with("MemAvailable:") {
                mem_available = self.extract_memory_from_line(line)?;
            }
        }
        
        let used_memory = mem_total - mem_available;
        Ok((used_memory * 1024, mem_available * 1024)) // Convert KB to bytes
    }
    
    #[cfg(target_os = "windows")]
    async fn collect_memory_metrics_windows(&self) -> Result<(u64, u64), Box<dyn std::error::Error>> {
        let output = Command::new("wmic")
            .args(&["OS", "get", "TotalVisibleMemorySize,FreePhysicalMemory", "/value"])
            .output()?;
        
        let output_str = String::from_utf8(output.stdout)?;
        
        let mut total_memory = 0u64;
        let mut free_memory = 0u64;
        
        for line in output_str.lines() {
            if line.starts_with("TotalVisibleMemorySize=") {
                let value_str = line.trim_start_matches("TotalVisibleMemorySize=");
                total_memory = value_str.parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes
            } else if line.starts_with("FreePhysicalMemory=") {
                let value_str = line.trim_start_matches("FreePhysicalMemory=");
                free_memory = value_str.parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes
            }
        }
        
        let used_memory = total_memory - free_memory;
        Ok((used_memory, free_memory))
    }
    
    /// Collect disk metrics
    async fn collect_disk_metrics(&self) -> Result<(u64, u64), Box<dyn std::error::Error>> {
        #[cfg(unix)]
        {
            let output = Command::new("df")
                .args(&["-h", "."])
                .output()?;
            
            let output_str = String::from_utf8(output.stdout)?;
            
            // Parse df output to get disk usage
            for line in output_str.lines().skip(1) {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 4 {
                    // Fields: Filesystem, Size, Used, Available, Use%, Mounted on
                    let used = self.parse_disk_size(fields[2])?;
                    let available = self.parse_disk_size(fields[3])?;
                    return Ok((used, available));
                }
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("dir")
                .args(&["/-c"])
                .output()?;
            
            // Parse Windows dir output for disk usage
            // This is a simplified implementation
            return Ok((0, 0));
        }
        
        warn!("Failed to collect disk metrics");
        Ok((0, 0))
    }
    
    /// Collect network I/O metrics
    async fn collect_network_metrics(&self) -> Result<NetworkIO, Box<dyn std::error::Error>> {
        #[cfg(target_os = "linux")]
        {
            let net_dev_content = tokio::fs::read_to_string("/proc/net/dev").await?;
            
            let mut total_rx_bytes = 0u64;
            let mut total_tx_bytes = 0u64;
            let mut total_rx_packets = 0u64;
            let mut total_tx_packets = 0u64;
            
            for line in net_dev_content.lines().skip(2) {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 10 {
                    // Skip loopback interface
                    if !fields[0].starts_with("lo:") {
                        total_rx_bytes += fields[1].parse::<u64>().unwrap_or(0);
                        total_rx_packets += fields[2].parse::<u64>().unwrap_or(0);
                        total_tx_bytes += fields[9].parse::<u64>().unwrap_or(0);
                        total_tx_packets += fields[10].parse::<u64>().unwrap_or(0);
                    }
                }
            }
            
            return Ok(NetworkIO {
                bytes_received: total_rx_bytes,
                bytes_sent: total_tx_bytes,
                packets_received: total_rx_packets,
                packets_sent: total_tx_packets,
            });
        }
        
        // For non-Linux platforms, return empty metrics
        Ok(NetworkIO {
            bytes_received: 0,
            bytes_sent: 0,
            packets_received: 0,
            packets_sent: 0,
        })
    }
    
    /// Collect process-specific metrics
    async fn collect_process_metrics(&mut self) -> Result<HashMap<String, ProcessMetrics>, Box<dyn std::error::Error>> {
        let mut process_metrics = HashMap::new();
        
        // Get list of all wezterm-parallel related processes
        let processes = self.find_related_processes().await?;
        
        for process_info in processes {
            let metrics = self.collect_single_process_metrics(&process_info).await?;
            process_metrics.insert(process_info.name.clone(), metrics);
        }
        
        Ok(process_metrics)
    }
    
    /// Find processes related to wezterm-parallel
    async fn find_related_processes(&self) -> Result<Vec<ProcessInfo>, Box<dyn std::error::Error>> {
        let mut processes = Vec::new();
        
        #[cfg(unix)]
        {
            let output = Command::new("ps")
                .args(&["aux"])
                .output()?;
            
            let output_str = String::from_utf8(output.stdout)?;
            
            for line in output_str.lines().skip(1) {
                if line.contains("wezterm-parallel") || line.contains("claude") {
                    if let Some(process_info) = self.parse_ps_line(line) {
                        processes.push(process_info);
                    }
                }
            }
        }
        
        Ok(processes)
    }
    
    /// Parse ps command output line
    #[cfg(unix)]
    fn parse_ps_line(&self, line: &str) -> Option<ProcessInfo> {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 11 {
            let pid = fields[1].parse::<u32>().ok()?;
            let cpu = fields[2].parse::<f64>().unwrap_or(0.0);
            let mem = fields[3].parse::<f64>().unwrap_or(0.0);
            let command = fields[10..].join(" ");
            
            Some(ProcessInfo {
                pid,
                name: Self::extract_process_name(&command),
                cpu_usage: cpu,
                memory_usage: (mem * 1024.0 * 1024.0) as u64, // Rough estimation
                command,
            })
        } else {
            None
        }
    }
    
    /// Extract process name from command
    fn extract_process_name(command: &str) -> String {
        if let Some(last_slash) = command.rfind('/') {
            command[last_slash + 1..].split_whitespace().next().unwrap_or("unknown").to_string()
        } else {
            command.split_whitespace().next().unwrap_or("unknown").to_string()
        }
    }
    
    /// Collect metrics for a single process
    async fn collect_single_process_metrics(&mut self, process_info: &ProcessInfo) -> Result<ProcessMetrics, Box<dyn std::error::Error>> {
        let current_time = current_timestamp();
        
        // Get or create cached process metrics
        let cached_metrics = self.process_cache.get(&process_info.pid);
        let restart_count = cached_metrics.map(|m| m.restart_count).unwrap_or(0);
        let start_time = cached_metrics.map(|m| current_time - m.uptime).unwrap_or(current_time);
        
        let metrics = ProcessMetrics {
            pid: process_info.pid,
            name: process_info.name.clone(),
            cpu_usage: process_info.cpu_usage,
            memory_usage: process_info.memory_usage,
            thread_count: self.get_thread_count(process_info.pid).await.unwrap_or(1),
            fd_count: self.get_fd_count(process_info.pid).await.unwrap_or(0),
            uptime: current_time - start_time,
            status: ProcessStatus::Running,
            restart_count,
        };
        
        // Update cache
        self.process_cache.insert(process_info.pid, metrics.clone());
        
        Ok(metrics)
    }
    
    /// Get thread count for a process
    async fn get_thread_count(&self, _pid: u32) -> Result<u32, Box<dyn std::error::Error>> {
        #[cfg(target_os = "linux")]
        {
            let stat_path = format!("/proc/{}/stat", pid);
            if let Ok(stat_content) = tokio::fs::read_to_string(&stat_path).await {
                let fields: Vec<&str> = stat_content.split_whitespace().collect();
                if fields.len() > 19 {
                    return Ok(fields[19].parse::<u32>().unwrap_or(1));
                }
            }
        }
        
        Ok(1) // Default to 1 thread
    }
    
    /// Get file descriptor count for a process
    async fn get_fd_count(&self, _pid: u32) -> Result<u32, Box<dyn std::error::Error>> {
        #[cfg(target_os = "linux")]
        {
            let fd_dir = format!("/proc/{}/fd", pid);
            if let Ok(mut entries) = tokio::fs::read_dir(&fd_dir).await {
                let mut count = 0;
                while let Ok(Some(_)) = entries.next_entry().await {
                    count += 1;
                }
                return Ok(count);
            }
        }
        
        Ok(0) // Default to 0 if unable to read
    }
    
    /// Helper function to extract pages from vm_stat line
    fn extract_pages_from_line(&self, line: &str) -> u64 {
        if let Some(colon_pos) = line.find(':') {
            let number_part = &line[colon_pos + 1..];
            let number_str = number_part.trim().trim_end_matches('.');
            number_str.parse::<u64>().unwrap_or(0)
        } else {
            0
        }
    }
    
    /// Helper function to extract memory from /proc/meminfo line
    fn extract_memory_from_line(&self, line: &str) -> Result<u64, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            Ok(parts[1].parse::<u64>()?)
        } else {
            Ok(0)
        }
    }
    
    /// Parse disk size from df output
    fn parse_disk_size(&self, size_str: &str) -> Result<u64, Box<dyn std::error::Error>> {
        let size_str = size_str.to_uppercase();
        let multiplier = if size_str.ends_with('K') {
            1024
        } else if size_str.ends_with('M') {
            1024 * 1024
        } else if size_str.ends_with('G') {
            1024 * 1024 * 1024
        } else if size_str.ends_with('T') {
            1024u64 * 1024 * 1024 * 1024
        } else {
            1
        };
        
        let number_part = if multiplier > 1 {
            &size_str[..size_str.len() - 1]
        } else {
            &size_str
        };
        
        let number: f64 = number_part.parse()?;
        Ok((number * multiplier as f64) as u64)
    }
}

/// Process information for metrics collection
#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_usage: f64,
    memory_usage: u64,
    command: String,
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert!(collector.process_cache.is_empty());
        assert!(collector.network_baseline.is_none());
    }
    
    #[tokio::test]
    async fn test_disk_size_parsing() {
        let collector = MetricsCollector::new();
        
        assert_eq!(collector.parse_disk_size("1024").unwrap(), 1024);
        assert_eq!(collector.parse_disk_size("1K").unwrap(), 1024);
        assert_eq!(collector.parse_disk_size("1M").unwrap(), 1024 * 1024);
        assert_eq!(collector.parse_disk_size("1G").unwrap(), 1024 * 1024 * 1024);
    }
}