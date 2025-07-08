// WezTerm Multi-Process Development Framework - Performance Optimization
// パフォーマンス最適化モジュール

pub mod async_opt;
pub mod memory;
pub mod metrics;
pub mod startup;

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// パフォーマンス設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 遅延初期化を有効にする
    pub lazy_initialization: bool,
    /// プリロードするモジュール数の制限
    pub max_preload_modules: usize,
    /// メモリプール初期サイズ
    pub initial_memory_pool_size: usize,
    /// 非同期タスクプール初期サイズ
    pub async_task_pool_size: usize,
    /// ガベージコレクション間隔（秒）
    pub gc_interval_secs: u64,
    /// CPU使用率制限（%）
    pub cpu_limit_percent: f64,
    /// メモリ使用量制限（MB）
    pub memory_limit_mb: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            lazy_initialization: true,
            max_preload_modules: 5,
            initial_memory_pool_size: 1024 * 1024, // 1MB
            async_task_pool_size: 4,
            gc_interval_secs: 300, // 5分
            cpu_limit_percent: 80.0,
            memory_limit_mb: 512,
        }
    }
}

/// パフォーマンスメトリクス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub startup_time: Duration,
    pub memory_usage: usize,
    pub cpu_usage: f64,
    pub active_tasks: usize,
    pub peak_memory: usize,
    pub gc_runs: u32,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            startup_time: Duration::from_secs(0),
            memory_usage: 0,
            cpu_usage: 0.0,
            active_tasks: 0,
            peak_memory: 0,
            gc_runs: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}

/// パフォーマンス最適化マネージャー
pub struct PerformanceManager {
    config: PerformanceConfig,
    metrics: PerformanceMetrics,
    start_time: Instant,
    last_gc: Instant,
    memory_pool: Vec<Vec<u8>>,
    cache: std::collections::HashMap<String, Vec<u8>>,
}

impl PerformanceManager {
    pub fn new(config: PerformanceConfig) -> Self {
        let start_time = Instant::now();
        info!("パフォーマンス最適化マネージャーを初期化中...");

        // メモリプールを事前確保
        let mut memory_pool = Vec::with_capacity(16);
        for _ in 0..8 {
            memory_pool.push(Vec::with_capacity(config.initial_memory_pool_size / 8));
        }

        Self {
            config,
            metrics: PerformanceMetrics::default(),
            start_time,
            last_gc: start_time,
            memory_pool,
            cache: std::collections::HashMap::new(),
        }
    }

    /// 起動完了を記録
    pub fn record_startup_complete(&mut self) {
        self.metrics.startup_time = self.start_time.elapsed();
        info!("起動完了: {:?}", self.metrics.startup_time);
    }

    /// メモリ使用量を更新
    pub fn update_memory_usage(&mut self, usage: usize) {
        self.metrics.memory_usage = usage;
        if usage > self.metrics.peak_memory {
            self.metrics.peak_memory = usage;
        }

        // メモリ制限チェック
        let limit_bytes = self.config.memory_limit_mb * 1024 * 1024;
        if usage > limit_bytes {
            warn!(
                "メモリ使用量が制限を超過: {}MB > {}MB",
                usage / 1024 / 1024,
                self.config.memory_limit_mb
            );
            self.trigger_gc();
        }
    }

    /// CPU使用率を更新
    pub fn update_cpu_usage(&mut self, usage: f64) {
        self.metrics.cpu_usage = usage;

        if usage > self.config.cpu_limit_percent {
            warn!(
                "CPU使用率が制限を超過: {:.1}% > {:.1}%",
                usage, self.config.cpu_limit_percent
            );
        }
    }

    /// アクティブタスク数を更新
    pub fn update_active_tasks(&mut self, count: usize) {
        self.metrics.active_tasks = count;
    }

    /// ガベージコレクションを実行
    pub fn trigger_gc(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_gc).as_secs() < 60 {
            // 1分以内のGC実行を制限
            return;
        }

        debug!("ガベージコレクションを実行中...");

        // キャッシュクリーンアップ
        let cache_size_before = self.cache.len();
        self.cache.retain(|_, v| v.capacity() <= 1024); // 1KB以下のみ保持
        let cache_size_after = self.cache.len();

        // メモリプールリセット
        for buffer in &mut self.memory_pool {
            buffer.clear();
            buffer.shrink_to_fit();
        }

        self.metrics.gc_runs += 1;
        self.last_gc = now;

        info!(
            "GC完了: キャッシュ {}→{} エントリ",
            cache_size_before, cache_size_after
        );
    }

    /// 定期的なガベージコレクション
    pub fn periodic_gc(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_gc).as_secs() >= self.config.gc_interval_secs {
            self.trigger_gc();
        }
    }

    /// キャッシュからデータを取得
    pub fn get_cached(&mut self, key: &str) -> Option<Vec<u8>> {
        if let Some(data) = self.cache.get(key) {
            self.metrics.cache_hits += 1;
            Some(data.clone())
        } else {
            self.metrics.cache_misses += 1;
            None
        }
    }

    /// データをキャッシュに保存
    pub fn cache_data(&mut self, key: String, data: Vec<u8>) {
        // キャッシュサイズ制限
        if self.cache.len() >= 100 {
            // 最も古いエントリを削除
            if let Some(oldest_key) = self.cache.keys().next().cloned() {
                self.cache.remove(&oldest_key);
            }
        }

        self.cache.insert(key, data);
    }

    /// メモリプールからバッファを取得
    pub fn get_buffer(&mut self, size: usize) -> Vec<u8> {
        for buffer in &mut self.memory_pool {
            if buffer.is_empty() {
                buffer.reserve(size);
                return std::mem::take(buffer);
            }
        }

        // プールが空の場合は新規作成
        Vec::with_capacity(size)
    }

    /// バッファをメモリプールに返却
    pub fn return_buffer(&mut self, mut buffer: Vec<u8>) {
        buffer.clear();

        if buffer.capacity() <= self.config.initial_memory_pool_size {
            for slot in &mut self.memory_pool {
                if slot.is_empty() {
                    *slot = buffer;
                    return;
                }
            }
        }

        // プールが満杯またはバッファが大きすぎる場合は破棄
        drop(buffer);
    }

    /// パフォーマンス統計を取得
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    /// パフォーマンスレポートを生成
    pub fn generate_report(&self) -> String {
        format!(
            "=== パフォーマンスレポート ===\n\
            起動時間: {:?}\n\
            メモリ使用量: {}MB (ピーク: {}MB)\n\
            CPU使用率: {:.1}%\n\
            アクティブタスク: {}\n\
            GC実行回数: {}\n\
            キャッシュヒット率: {:.1}%\n\
            メモリプール使用中: {}/{}",
            self.metrics.startup_time,
            self.metrics.memory_usage / 1024 / 1024,
            self.metrics.peak_memory / 1024 / 1024,
            self.metrics.cpu_usage,
            self.metrics.active_tasks,
            self.metrics.gc_runs,
            if self.metrics.cache_hits + self.metrics.cache_misses > 0 {
                (self.metrics.cache_hits as f64
                    / (self.metrics.cache_hits + self.metrics.cache_misses) as f64)
                    * 100.0
            } else {
                0.0
            },
            self.memory_pool.iter().filter(|b| !b.is_empty()).count(),
            self.memory_pool.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_manager_creation() {
        let config = PerformanceConfig::default();
        let manager = PerformanceManager::new(config);

        assert_eq!(manager.metrics.startup_time, Duration::from_secs(0));
        assert_eq!(manager.memory_pool.len(), 8);
    }

    #[test]
    fn test_memory_pool() {
        let config = PerformanceConfig::default();
        let mut manager = PerformanceManager::new(config);

        let buffer = manager.get_buffer(1024);
        assert!(buffer.capacity() >= 1024);

        manager.return_buffer(buffer);
    }

    #[test]
    fn test_cache_operations() {
        let config = PerformanceConfig::default();
        let mut manager = PerformanceManager::new(config);

        // キャッシュミス
        assert!(manager.get_cached("test").is_none());
        assert_eq!(manager.metrics.cache_misses, 1);

        // データをキャッシュ
        manager.cache_data("test".to_string(), vec![1, 2, 3]);

        // キャッシュヒット
        let data = manager.get_cached("test");
        assert!(data.is_some());
        assert_eq!(data.unwrap(), vec![1, 2, 3]);
        assert_eq!(manager.metrics.cache_hits, 1);
    }

    #[test]
    fn test_metrics_update() {
        let config = PerformanceConfig::default();
        let mut manager = PerformanceManager::new(config);

        manager.update_memory_usage(1024 * 1024); // 1MB
        manager.update_cpu_usage(50.0);
        manager.update_active_tasks(5);

        assert_eq!(manager.metrics.memory_usage, 1024 * 1024);
        assert_eq!(manager.metrics.cpu_usage, 50.0);
        assert_eq!(manager.metrics.active_tasks, 5);
    }
}
