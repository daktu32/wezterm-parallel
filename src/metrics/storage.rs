// Metrics storage and retrieval for historical data

use super::{FrameworkMetrics, ProcessMetrics, SystemMetrics, WorkspaceMetrics};
use log::{debug, info, warn};
use serde_json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;

/// Metrics storage for persistent historical data
pub struct MetricsStorage {
    /// Base directory for metrics storage
    base_path: PathBuf,

    /// In-memory cache for recent metrics
    cache: Arc<RwLock<MetricsCache>>,

    /// Maximum number of files to keep per metric type
    #[allow(dead_code)]
    max_files_per_type: usize,

    /// Maximum file size in bytes before rotation
    max_file_size: u64,
}

/// In-memory metrics cache
#[derive(Debug)]
struct MetricsCache {
    /// Recent system metrics
    system_metrics: Vec<SystemMetrics>,

    /// Recent process metrics by process ID
    process_metrics: HashMap<String, Vec<ProcessMetrics>>,

    /// Recent workspace metrics by workspace name
    workspace_metrics: HashMap<String, Vec<WorkspaceMetrics>>,

    /// Recent framework metrics
    framework_metrics: Vec<FrameworkMetrics>,
}

impl MetricsStorage {
    /// Create a new metrics storage instance
    pub async fn new(base_path: PathBuf) -> Result<Self, String> {
        // Create base directory if it doesn't exist
        fs::create_dir_all(&base_path)
            .await
            .map_err(|e| format!("Failed to create metrics directory: {e}"))?;

        let storage = Self {
            base_path,
            cache: Arc::new(RwLock::new(MetricsCache::new())),
            max_files_per_type: 24,          // Keep 24 hours of hourly files
            max_file_size: 10 * 1024 * 1024, // 10MB per file
        };

        // Load recent metrics into cache
        storage.load_recent_metrics().await?;

        Ok(storage)
    }

    /// Save system metrics
    pub async fn save_system_metrics(&self, metrics: &SystemMetrics) -> Result<(), String> {
        debug!("Saving system metrics");

        // Add to cache
        {
            let mut cache = self.cache.write().await;
            cache.system_metrics.push(metrics.clone());

            // Trim cache if needed
            if cache.system_metrics.len() > 1000 {
                cache.system_metrics.drain(0..500);
            }
        }

        // Write to file
        let file_path = self.get_metrics_file_path("system", metrics.timestamp);
        self.append_to_file(&file_path, metrics).await?;

        Ok(())
    }

    /// Save process metrics
    pub async fn save_process_metrics(&self, metrics: &[ProcessMetrics]) -> Result<(), String> {
        debug!("Saving {} process metrics", metrics.len());

        // Group by process ID for cache
        let mut by_process: HashMap<String, Vec<ProcessMetrics>> = HashMap::new();
        for metric in metrics {
            by_process
                .entry(metric.process_id.clone())
                .or_default()
                .push(metric.clone());
        }

        // Add to cache
        {
            let mut cache = self.cache.write().await;
            for (process_id, process_metrics) in by_process {
                let cached = cache
                    .process_metrics
                    .entry(process_id)
                    .or_insert_with(Vec::new);
                cached.extend(process_metrics);

                // Trim if needed
                if cached.len() > 1000 {
                    cached.drain(0..500);
                }
            }
        }

        // Write to file
        if let Some(first_metric) = metrics.first() {
            let file_path = self.get_metrics_file_path("process", first_metric.timestamp);
            self.append_batch_to_file(&file_path, metrics).await?;
        }

        Ok(())
    }

    /// Save workspace metrics
    pub async fn save_workspace_metrics(&self, metrics: &WorkspaceMetrics) -> Result<(), String> {
        debug!("Saving workspace metrics for {}", metrics.workspace_name);

        // Add to cache
        {
            let mut cache = self.cache.write().await;
            let cached = cache
                .workspace_metrics
                .entry(metrics.workspace_name.clone())
                .or_insert_with(Vec::new);
            cached.push(metrics.clone());

            // Trim if needed
            if cached.len() > 1000 {
                cached.drain(0..500);
            }
        }

        // Write to file
        let file_path = self.get_metrics_file_path("workspace", metrics.timestamp);
        self.append_to_file(&file_path, metrics).await?;

        Ok(())
    }

    /// Save framework metrics
    pub async fn save_framework_metrics(&self, metrics: &FrameworkMetrics) -> Result<(), String> {
        debug!("Saving framework metrics");

        // Add to cache
        {
            let mut cache = self.cache.write().await;
            cache.framework_metrics.push(metrics.clone());

            // Trim cache if needed
            if cache.framework_metrics.len() > 1000 {
                cache.framework_metrics.drain(0..500);
            }
        }

        // Write to file
        let file_path = self.get_metrics_file_path("framework", metrics.timestamp);
        self.append_to_file(&file_path, metrics).await?;

        Ok(())
    }

    /// Get recent system metrics
    pub async fn get_recent_system_metrics(&self, limit: usize) -> Vec<SystemMetrics> {
        let cache = self.cache.read().await;
        let metrics = &cache.system_metrics;

        if metrics.len() <= limit {
            metrics.clone()
        } else {
            metrics[metrics.len() - limit..].to_vec()
        }
    }

    /// Get recent process metrics
    pub async fn get_recent_process_metrics(
        &self,
        process_id: &str,
        limit: usize,
    ) -> Vec<ProcessMetrics> {
        let cache = self.cache.read().await;

        if let Some(metrics) = cache.process_metrics.get(process_id) {
            if metrics.len() <= limit {
                metrics.clone()
            } else {
                metrics[metrics.len() - limit..].to_vec()
            }
        } else {
            Vec::new()
        }
    }

    /// Get recent workspace metrics
    pub async fn get_recent_workspace_metrics(
        &self,
        workspace_name: &str,
        limit: usize,
    ) -> Vec<WorkspaceMetrics> {
        let cache = self.cache.read().await;

        if let Some(metrics) = cache.workspace_metrics.get(workspace_name) {
            if metrics.len() <= limit {
                metrics.clone()
            } else {
                metrics[metrics.len() - limit..].to_vec()
            }
        } else {
            Vec::new()
        }
    }

    /// Get recent framework metrics
    pub async fn get_recent_framework_metrics(&self, limit: usize) -> Vec<FrameworkMetrics> {
        let cache = self.cache.read().await;
        let metrics = &cache.framework_metrics;

        if metrics.len() <= limit {
            metrics.clone()
        } else {
            metrics[metrics.len() - limit..].to_vec()
        }
    }

    /// Load metrics for a specific time range
    pub async fn load_metrics_range(
        &self,
        metric_type: &str,
        start_time: u64,
        end_time: u64,
    ) -> Result<Vec<serde_json::Value>, String> {
        let mut all_metrics = Vec::new();

        // Calculate hourly file paths to check
        let start_hour = start_time / 3600;
        let end_hour = end_time / 3600;

        for hour in start_hour..=end_hour {
            let timestamp = hour * 3600;
            let file_path = self.get_metrics_file_path(metric_type, timestamp);

            if file_path.exists() {
                match self.load_file(&file_path).await {
                    Ok(metrics) => {
                        // Filter metrics within time range
                        let filtered: Vec<serde_json::Value> = metrics
                            .into_iter()
                            .filter(|m| {
                                if let Some(ts) = m.get("timestamp").and_then(|v| v.as_u64()) {
                                    ts >= start_time && ts <= end_time
                                } else {
                                    false
                                }
                            })
                            .collect();
                        all_metrics.extend(filtered);
                    }
                    Err(e) => warn!("Failed to load metrics from {}: {}", file_path.display(), e),
                }
            }
        }

        Ok(all_metrics)
    }

    /// Clean up old metrics files
    pub async fn cleanup_old_files(&self, retention_hours: u64) -> Result<(), String> {
        info!("Cleaning up metrics files older than {retention_hours} hours");

        let cutoff_time = SystemMetrics::current_timestamp().saturating_sub(retention_hours * 3600);

        for metric_type in &["system", "process", "workspace", "framework"] {
            let type_dir = self.base_path.join(metric_type);

            if type_dir.exists() {
                let mut entries = fs::read_dir(&type_dir)
                    .await
                    .map_err(|e| format!("Failed to read directory: {e}"))?;

                while let Some(entry) = entries
                    .next_entry()
                    .await
                    .map_err(|e| format!("Failed to read directory entry: {e}"))?
                {
                    let path = entry.path();
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        // Parse timestamp from filename (format: metrics_TIMESTAMP.json)
                        if let Some(timestamp_str) = file_name
                            .strip_prefix("metrics_")
                            .and_then(|s| s.strip_suffix(".json"))
                        {
                            if let Ok(timestamp) = timestamp_str.parse::<u64>() {
                                if timestamp < cutoff_time {
                                    match fs::remove_file(&path).await {
                                        Ok(_) => {
                                            debug!("Removed old metrics file: {}", path.display())
                                        }
                                        Err(e) => {
                                            warn!("Failed to remove file {}: {}", path.display(), e)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get metrics file path
    fn get_metrics_file_path(&self, metric_type: &str, timestamp: u64) -> PathBuf {
        // Group by hour for efficient storage
        let hour_timestamp = (timestamp / 3600) * 3600;

        self.base_path
            .join(metric_type)
            .join(format!("metrics_{hour_timestamp}.json"))
    }

    /// Append metrics to file
    async fn append_to_file<T: serde::Serialize>(
        &self,
        path: &Path,
        data: &T,
    ) -> Result<(), String> {
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }

        // Serialize data
        let json_line =
            serde_json::to_string(data).map_err(|e| format!("Failed to serialize data: {e}"))?;

        // Append to file with newline
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
            .map_err(|e| format!("Failed to open file: {e}"))?;

        file.write_all(format!("{json_line}\n").as_bytes())
            .await
            .map_err(|e| format!("Failed to write to file: {e}"))?;

        // Check file size and rotate if needed
        let metadata = file
            .metadata()
            .await
            .map_err(|e| format!("Failed to get file metadata: {e}"))?;

        if metadata.len() > self.max_file_size {
            self.rotate_file(path).await?;
        }

        Ok(())
    }

    /// Append batch of metrics to file
    async fn append_batch_to_file<T: serde::Serialize>(
        &self,
        path: &Path,
        data: &[T],
    ) -> Result<(), String> {
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }

        // Serialize all data
        let mut json_lines = String::new();
        for item in data {
            let line = serde_json::to_string(item)
                .map_err(|e| format!("Failed to serialize data: {e}"))?;
            json_lines.push_str(&line);
            json_lines.push('\n');
        }

        // Append to file
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
            .map_err(|e| format!("Failed to open file: {e}"))?;

        file.write_all(json_lines.as_bytes())
            .await
            .map_err(|e| format!("Failed to write to file: {e}"))?;

        Ok(())
    }

    /// Load metrics from file
    async fn load_file(&self, path: &Path) -> Result<Vec<serde_json::Value>, String> {
        let mut file = fs::File::open(path)
            .await
            .map_err(|e| format!("Failed to open file: {e}"))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(|e| format!("Failed to read file: {e}"))?;

        let mut metrics = Vec::new();
        for line in contents.lines() {
            if !line.trim().is_empty() {
                match serde_json::from_str(line) {
                    Ok(value) => metrics.push(value),
                    Err(e) => warn!("Failed to parse line: {e}"),
                }
            }
        }

        Ok(metrics)
    }

    /// Rotate file when it gets too large
    async fn rotate_file(&self, path: &Path) -> Result<(), String> {
        let new_path = path.with_extension("json.old");

        fs::rename(path, &new_path)
            .await
            .map_err(|e| format!("Failed to rotate file: {e}"))?;

        info!(
            "Rotated large metrics file: {} -> {}",
            path.display(),
            new_path.display()
        );

        Ok(())
    }

    /// Load recent metrics into cache on startup
    async fn load_recent_metrics(&self) -> Result<(), String> {
        info!("Loading recent metrics into cache");

        let current_time = SystemMetrics::current_timestamp();
        let one_hour_ago = current_time.saturating_sub(3600);

        // Load each metric type
        for metric_type in &["system", "process", "workspace", "framework"] {
            match self
                .load_metrics_range(metric_type, one_hour_ago, current_time)
                .await
            {
                Ok(metrics) => {
                    debug!("Loaded {} {} metrics", metrics.len(), metric_type);
                    // Could deserialize and add to cache here if needed
                }
                Err(e) => warn!("Failed to load {metric_type} metrics: {e}"),
            }
        }

        Ok(())
    }

    /// Get storage statistics
    pub async fn get_storage_stats(&self) -> Result<StorageStats, String> {
        let mut total_files = 0;
        let mut total_size = 0;
        let mut metrics_by_type = HashMap::new();

        for metric_type in &["system", "process", "workspace", "framework"] {
            let type_dir = self.base_path.join(metric_type);
            let mut type_files = 0;
            let mut type_size = 0;

            if type_dir.exists() {
                let mut entries = fs::read_dir(&type_dir)
                    .await
                    .map_err(|e| format!("Failed to read directory: {e}"))?;

                while let Some(entry) = entries
                    .next_entry()
                    .await
                    .map_err(|e| format!("Failed to read directory entry: {e}"))?
                {
                    let metadata = entry
                        .metadata()
                        .await
                        .map_err(|e| format!("Failed to get metadata: {e}"))?;

                    if metadata.is_file() {
                        type_files += 1;
                        type_size += metadata.len();
                    }
                }
            }

            metrics_by_type.insert(
                metric_type.to_string(),
                TypeStats {
                    file_count: type_files,
                    total_size: type_size,
                },
            );

            total_files += type_files;
            total_size += type_size;
        }

        Ok(StorageStats {
            total_files,
            total_size,
            metrics_by_type,
            cache_size: self.estimate_cache_size().await,
        })
    }

    /// Estimate cache memory usage
    async fn estimate_cache_size(&self) -> usize {
        let cache = self.cache.read().await;

        let system_size = cache.system_metrics.len() * std::mem::size_of::<SystemMetrics>();
        let process_size = cache
            .process_metrics
            .values()
            .map(|v| v.len() * std::mem::size_of::<ProcessMetrics>())
            .sum::<usize>();
        let workspace_size = cache
            .workspace_metrics
            .values()
            .map(|v| v.len() * std::mem::size_of::<WorkspaceMetrics>())
            .sum::<usize>();
        let framework_size =
            cache.framework_metrics.len() * std::mem::size_of::<FrameworkMetrics>();

        system_size + process_size + workspace_size + framework_size
    }
}

impl MetricsCache {
    fn new() -> Self {
        Self {
            system_metrics: Vec::new(),
            process_metrics: HashMap::new(),
            workspace_metrics: HashMap::new(),
            framework_metrics: Vec::new(),
        }
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_files: u64,
    pub total_size: u64,
    pub metrics_by_type: HashMap<String, TypeStats>,
    pub cache_size: usize,
}

/// Per-type storage statistics
#[derive(Debug, Clone)]
pub struct TypeStats {
    pub file_count: u64,
    pub total_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_metrics_storage() {
        let temp_dir = tempdir().unwrap();
        let storage = MetricsStorage::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        // Test saving system metrics
        let system_metrics = SystemMetrics::new();
        storage.save_system_metrics(&system_metrics).await.unwrap();

        // Test retrieving metrics
        let recent = storage.get_recent_system_metrics(10).await;
        assert_eq!(recent.len(), 1);
    }

    #[tokio::test]
    async fn test_cache_trimming() {
        let temp_dir = tempdir().unwrap();
        let storage = MetricsStorage::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        // Add many metrics to trigger trimming
        for i in 0..1500 {
            let mut metrics = SystemMetrics::new();
            metrics.timestamp = i;
            storage.save_system_metrics(&metrics).await.unwrap();
        }

        // Cache should be trimmed
        let recent = storage.get_recent_system_metrics(2000).await;
        assert!(recent.len() < 1500);
        assert!(recent.len() >= 500);
    }
}
