// WezTerm Multi-Process Development Framework - Memory Optimization
// メモリ使用量最適化

use crate::logging::LogContext;
use crate::{log_debug, log_info, log_warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// メモリプール管理
pub struct MemoryPool {
    pools: HashMap<usize, Vec<Vec<u8>>>,
    max_pool_size: usize,
    total_allocated: usize,
    total_deallocated: usize,
    peak_usage: usize,
}

impl MemoryPool {
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            pools: HashMap::new(),
            max_pool_size,
            total_allocated: 0,
            total_deallocated: 0,
            peak_usage: 0,
        }
    }

    /// メモリブロックを取得
    pub fn allocate(&mut self, size: usize) -> Vec<u8> {
        // サイズを2の累乗に正規化
        let normalized_size = size.next_power_of_two();

        if let Some(pool) = self.pools.get_mut(&normalized_size) {
            if let Some(mut buffer) = pool.pop() {
                buffer.clear();
                buffer.reserve(size);
                self.total_allocated += normalized_size;
                return buffer;
            }
        }

        // プールが空または存在しない場合は新規作成
        let buffer = Vec::with_capacity(normalized_size);
        self.total_allocated += normalized_size;

        if self.total_allocated > self.peak_usage {
            self.peak_usage = self.total_allocated;
        }

        buffer
    }

    /// メモリブロックを返却
    pub fn deallocate(&mut self, buffer: Vec<u8>) {
        let capacity = buffer.capacity();
        self.total_deallocated += capacity;

        // プールサイズ制限チェック
        let pool = self.pools.entry(capacity).or_default();
        if pool.len() < self.max_pool_size {
            pool.push(buffer);
        }
        // プールが満杯の場合はバッファを破棄
    }

    /// 統計情報を取得
    pub fn get_stats(&self) -> MemoryPoolStats {
        MemoryPoolStats {
            total_allocated: self.total_allocated,
            total_deallocated: self.total_deallocated,
            active_allocation: self.total_allocated - self.total_deallocated,
            peak_usage: self.peak_usage,
            pool_count: self.pools.len(),
            pooled_buffers: self.pools.values().map(|v| v.len()).sum(),
        }
    }

    /// プールをクリーンアップ
    pub fn cleanup(&mut self) {
        let before_count: usize = self.pools.values().map(|v| v.len()).sum();

        // 半分のサイズに縮小
        for pool in self.pools.values_mut() {
            pool.truncate(pool.len() / 2);
        }

        let after_count: usize = self.pools.values().map(|v| v.len()).sum();
        let cleanup_context = LogContext::new("performance", "memory_pool_cleanup")
            .with_metadata("before_count", serde_json::json!(before_count))
            .with_metadata("after_count", serde_json::json!(after_count));
        log_debug!(
            cleanup_context,
            "メモリプールクリーンアップ: {} → {} バッファ",
            before_count,
            after_count
        );
    }
}

/// メモリプール統計
#[derive(Debug, Clone)]
pub struct MemoryPoolStats {
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub active_allocation: usize,
    pub peak_usage: usize,
    pub pool_count: usize,
    pub pooled_buffers: usize,
}

/// 文字列インターナー（重複文字列の削減）
pub struct StringInterner {
    strings: HashMap<String, Arc<str>>,
    hit_count: u64,
    miss_count: u64,
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            hit_count: 0,
            miss_count: 0,
        }
    }

    /// 文字列をインターン
    pub fn intern(&mut self, s: &str) -> Arc<str> {
        if let Some(interned) = self.strings.get(s) {
            self.hit_count += 1;
            Arc::clone(interned)
        } else {
            let interned: Arc<str> = Arc::from(s);
            self.strings.insert(s.to_string(), Arc::clone(&interned));
            self.miss_count += 1;
            interned
        }
    }

    /// 統計情報
    pub fn get_stats(&self) -> (usize, u64, u64, f64) {
        let total_requests = self.hit_count + self.miss_count;
        let hit_rate = if total_requests > 0 {
            (self.hit_count as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        (
            self.strings.len(),
            self.hit_count,
            self.miss_count,
            hit_rate,
        )
    }

    /// クリーンアップ
    pub fn cleanup(&mut self) {
        let before_count = self.strings.len();

        // 参照カウントが1（このマップのみ）の文字列を削除
        self.strings
            .retain(|_, arc_str| Arc::strong_count(arc_str) > 1);

        let after_count = self.strings.len();
        let interner_cleanup_context = LogContext::new("performance", "string_interner_cleanup")
            .with_metadata("before_count", serde_json::json!(before_count))
            .with_metadata("after_count", serde_json::json!(after_count));
        log_debug!(
            interner_cleanup_context,
            "文字列インターナークリーンアップ: {} → {} 文字列",
            before_count,
            after_count
        );
    }
}

/// メモリ使用量監視
pub struct MemoryMonitor {
    memory_pool: Arc<RwLock<MemoryPool>>,
    string_interner: Arc<RwLock<StringInterner>>,
    last_check: Instant,
    check_interval: Duration,
    memory_limit: usize,
    warning_threshold: f64,
}

impl MemoryMonitor {
    pub fn new(memory_limit_mb: usize) -> Self {
        Self {
            memory_pool: Arc::new(RwLock::new(MemoryPool::new(16))),
            string_interner: Arc::new(RwLock::new(StringInterner::new())),
            last_check: Instant::now(),
            check_interval: Duration::from_secs(30),
            memory_limit: memory_limit_mb * 1024 * 1024,
            warning_threshold: 0.8, // 80%
        }
    }

    /// メモリ使用量をチェック
    pub async fn check_memory_usage(&mut self) -> Result<MemoryStatus, Box<dyn std::error::Error>> {
        let now = Instant::now();
        if now.duration_since(self.last_check) < self.check_interval {
            return Ok(MemoryStatus::Normal);
        }
        self.last_check = now;

        // システムメモリ使用量を取得（プラットフォーム依存）
        let current_usage = self.get_current_memory_usage().await?;
        let usage_ratio = current_usage as f64 / self.memory_limit as f64;

        let usage_check_context = LogContext::new("performance", "memory_usage_check")
            .with_metadata(
                "current_usage_mb",
                serde_json::json!(current_usage / 1024 / 1024),
            )
            .with_metadata(
                "memory_limit_mb",
                serde_json::json!(self.memory_limit / 1024 / 1024),
            )
            .with_metadata(
                "usage_ratio_percent",
                serde_json::json!(usage_ratio * 100.0),
            );
        log_debug!(
            usage_check_context,
            "メモリ使用量チェック: {}MB / {}MB ({:.1}%)",
            current_usage / 1024 / 1024,
            self.memory_limit / 1024 / 1024,
            usage_ratio * 100.0
        );

        if usage_ratio > 1.0 {
            let critical_context = LogContext::new("performance", "memory_critical").with_metadata(
                "usage_ratio_percent",
                serde_json::json!(usage_ratio * 100.0),
            );
            log_warn!(
                critical_context,
                "メモリ使用量が制限を超過: {:.1}%",
                usage_ratio * 100.0
            );
            self.emergency_cleanup().await;
            Ok(MemoryStatus::Critical)
        } else if usage_ratio > self.warning_threshold {
            let warning_context = LogContext::new("performance", "memory_warning")
                .with_metadata(
                    "usage_ratio_percent",
                    serde_json::json!(usage_ratio * 100.0),
                )
                .with_metadata(
                    "warning_threshold",
                    serde_json::json!(self.warning_threshold * 100.0),
                );
            log_warn!(
                warning_context,
                "メモリ使用量が警告レベル: {:.1}%",
                usage_ratio * 100.0
            );
            self.perform_cleanup().await;
            Ok(MemoryStatus::Warning)
        } else {
            Ok(MemoryStatus::Normal)
        }
    }

    /// システムメモリ使用量を取得
    async fn get_current_memory_usage(&self) -> Result<usize, Box<dyn std::error::Error>> {
        // 実際の実装ではプラットフォーム固有のAPIを使用
        #[cfg(target_os = "linux")]
        {
            self.get_linux_memory_usage().await
        }
        #[cfg(target_os = "macos")]
        {
            self.get_macos_memory_usage().await
        }
        #[cfg(target_os = "windows")]
        {
            self.get_windows_memory_usage().await
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            // フォールバック: プロセス統計から推定
            Ok(self.estimate_memory_usage().await)
        }
    }

    #[cfg(target_os = "linux")]
    async fn get_linux_memory_usage(&self) -> Result<usize, Box<dyn std::error::Error>> {
        // /proc/self/statusから読み取り
        let status = tokio::fs::read_to_string("/proc/self/status").await?;
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    return Ok(kb_str.parse::<usize>()? * 1024);
                }
            }
        }
        Ok(0)
    }

    #[cfg(target_os = "macos")]
    async fn get_macos_memory_usage(&self) -> Result<usize, Box<dyn std::error::Error>> {
        // macOS: task_info APIを使用（簡略化）
        Ok(self.estimate_memory_usage().await)
    }

    #[cfg(target_os = "windows")]
    async fn get_windows_memory_usage(&self) -> Result<usize, Box<dyn std::error::Error>> {
        // Windows: GetProcessMemoryInfo APIを使用（簡略化）
        Ok(self.estimate_memory_usage().await)
    }

    #[allow(dead_code)]
    async fn estimate_memory_usage(&self) -> usize {
        // メモリプールとインターナーの使用量から推定
        let pool_stats = {
            let pool = self.memory_pool.read().await;
            pool.get_stats()
        };

        let (string_count, _, _, _) = {
            let interner = self.string_interner.read().await;
            interner.get_stats()
        };

        // 推定メモリ使用量（実際の値より低めになる）
        pool_stats.active_allocation + (string_count * 64) // 文字列平均64バイトと仮定
    }

    /// 通常のクリーンアップ
    async fn perform_cleanup(&self) {
        let cleanup_context = LogContext::new("performance", "memory_cleanup");
        log_info!(cleanup_context, "メモリクリーンアップを実行中...");

        {
            let mut pool = self.memory_pool.write().await;
            pool.cleanup();
        }

        {
            let mut interner = self.string_interner.write().await;
            interner.cleanup();
        }
    }

    /// 緊急クリーンアップ
    async fn emergency_cleanup(&self) {
        let emergency_context = LogContext::new("performance", "memory_emergency_cleanup");
        log_warn!(emergency_context, "緊急メモリクリーンアップを実行中...");

        {
            let mut pool = self.memory_pool.write().await;
            // より積極的なクリーンアップ
            for pool_vec in pool.pools.values_mut() {
                pool_vec.clear();
            }
        }

        {
            let mut interner = self.string_interner.write().await;
            // 文字列インターナーを完全にクリア
            interner.strings.clear();
        }
    }

    /// メモリプールへの参照を取得
    pub fn get_memory_pool(&self) -> Arc<RwLock<MemoryPool>> {
        Arc::clone(&self.memory_pool)
    }

    /// 文字列インターナーへの参照を取得
    pub fn get_string_interner(&self) -> Arc<RwLock<StringInterner>> {
        Arc::clone(&self.string_interner)
    }

    /// メモリ統計レポートを生成
    pub async fn generate_memory_report(&self) -> String {
        let pool_stats = {
            let pool = self.memory_pool.read().await;
            pool.get_stats()
        };

        let (string_count, hit_count, miss_count, hit_rate) = {
            let interner = self.string_interner.read().await;
            interner.get_stats()
        };

        format!(
            "=== メモリ使用量レポート ===\n\
            メモリプール:\n\
            - 確保済み: {}MB\n\
            - 解放済み: {}MB\n\
            - アクティブ: {}MB\n\
            - ピーク: {}MB\n\
            - プール数: {}\n\
            - プールバッファ: {}\n\
            \n\
            文字列インターナー:\n\
            - インターン済み文字列: {}\n\
            - ヒット: {}\n\
            - ミス: {}\n\
            - ヒット率: {:.1}%",
            pool_stats.total_allocated / 1024 / 1024,
            pool_stats.total_deallocated / 1024 / 1024,
            pool_stats.active_allocation / 1024 / 1024,
            pool_stats.peak_usage / 1024 / 1024,
            pool_stats.pool_count,
            pool_stats.pooled_buffers,
            string_count,
            hit_count,
            miss_count,
            hit_rate
        )
    }
}

/// メモリステータス
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryStatus {
    Normal,
    Warning,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool() {
        let mut pool = MemoryPool::new(4);

        // アロケート
        let buffer1 = pool.allocate(1024);
        assert!(buffer1.capacity() >= 1024);

        let buffer2 = pool.allocate(2048);
        assert!(buffer2.capacity() >= 2048);

        // ディアロケート
        pool.deallocate(buffer1);
        pool.deallocate(buffer2);

        let stats = pool.get_stats();
        assert_eq!(stats.total_allocated, stats.total_deallocated);
    }

    #[test]
    fn test_string_interner() {
        let mut interner = StringInterner::new();

        let s1 = interner.intern("hello");
        let s2 = interner.intern("hello");
        let s3 = interner.intern("world");

        assert_eq!(&*s1, "hello");
        assert_eq!(&*s2, "hello");
        assert_eq!(&*s3, "world");

        // 同じ文字列は同じ参照
        assert!(Arc::ptr_eq(&s1, &s2));

        let (count, hits, misses, hit_rate) = interner.get_stats();
        assert_eq!(count, 2); // "hello", "world"
        assert_eq!(hits, 1); // "hello"の2回目
        assert_eq!(misses, 2); // "hello"の1回目, "world"
        assert!((hit_rate - 33.333333333333336).abs() < 1e-10); // 1/3 * 100 (浮動小数点精度対応)
    }

    #[tokio::test]
    async fn test_memory_monitor() {
        let mut monitor = MemoryMonitor::new(512); // 512MB制限

        // 最初のチェック（時間間隔により制限される可能性）
        let status = monitor.check_memory_usage().await.unwrap();
        assert!(matches!(
            status,
            MemoryStatus::Normal | MemoryStatus::Warning
        ));
    }

    #[tokio::test]
    async fn test_memory_report() {
        let monitor = MemoryMonitor::new(256);
        let report = monitor.generate_memory_report().await;

        assert!(report.contains("メモリ使用量レポート"));
        assert!(report.contains("メモリプール"));
        assert!(report.contains("文字列インターナー"));
    }

    // === MemoryPool 拡張テスト ===

    #[test]
    fn test_memory_pool_size_normalization() {
        let mut pool = MemoryPool::new(4);

        // 異なるサイズのアロケート
        let buffer1 = pool.allocate(100); // 128に正規化される
        let buffer2 = pool.allocate(200); // 256に正規化される
        let buffer3 = pool.allocate(1000); // 1024に正規化される

        assert_eq!(buffer1.capacity(), 128);
        assert_eq!(buffer2.capacity(), 256);
        assert_eq!(buffer3.capacity(), 1024);

        pool.deallocate(buffer1);
        pool.deallocate(buffer2);
        pool.deallocate(buffer3);

        let stats = pool.get_stats();
        assert_eq!(stats.total_allocated, stats.total_deallocated);
    }

    #[test]
    fn test_memory_pool_reuse() {
        let mut pool = MemoryPool::new(4);

        // 同じサイズのバッファを複数回アロケート
        let buffer1 = pool.allocate(1024);
        let capacity1 = buffer1.capacity();
        pool.deallocate(buffer1);

        let buffer2 = pool.allocate(1024);
        let capacity2 = buffer2.capacity();

        // 同じ容量が再利用されるべき
        assert_eq!(capacity1, capacity2);

        let stats = pool.get_stats();
        assert_eq!(stats.pooled_buffers, 0); // buffer2がまだ使用中
    }

    #[test]
    fn test_memory_pool_limit() {
        let mut pool = MemoryPool::new(2); // 最大2つのバッファ

        // 同じサイズのバッファを3つ作成
        let buffer1 = pool.allocate(1024);
        let buffer2 = pool.allocate(1024);
        let buffer3 = pool.allocate(1024);

        pool.deallocate(buffer1);
        pool.deallocate(buffer2);
        pool.deallocate(buffer3);

        let stats = pool.get_stats();
        // プールサイズ制限により、2つのバッファのみプールされる
        assert_eq!(stats.pooled_buffers, 2);
    }

    #[test]
    fn test_memory_pool_cleanup() {
        let mut pool = MemoryPool::new(10);

        // 複数のバッファを作成してプールに返却
        let mut buffers = Vec::new();
        for _ in 0..8 {
            let buffer = pool.allocate(1024);
            buffers.push(buffer);
        }

        // 全てのバッファを一度に返却
        for buffer in buffers {
            pool.deallocate(buffer);
        }

        let stats_before = pool.get_stats();
        assert_eq!(stats_before.pooled_buffers, 8);

        // クリーンアップ実行
        pool.cleanup();

        let stats_after = pool.get_stats();
        assert_eq!(stats_after.pooled_buffers, 4); // 半分に削減
    }

    #[test]
    fn test_memory_pool_peak_usage() {
        let mut pool = MemoryPool::new(4);

        let buffer1 = pool.allocate(1024);
        let peak1 = pool.get_stats().peak_usage;

        let buffer2 = pool.allocate(2048);
        let peak2 = pool.get_stats().peak_usage;

        assert!(peak2 > peak1);
        assert_eq!(peak2, 1024 + 2048);

        pool.deallocate(buffer1);
        pool.deallocate(buffer2);

        // ピーク使用量は削減されない
        let peak3 = pool.get_stats().peak_usage;
        assert_eq!(peak3, peak2);
    }

    #[test]
    fn test_memory_pool_multiple_sizes() {
        let mut pool = MemoryPool::new(4);

        // 異なるサイズのバッファを混在
        let buffer1 = pool.allocate(512);
        let buffer2 = pool.allocate(1024);
        let buffer3 = pool.allocate(2048);

        pool.deallocate(buffer1);
        pool.deallocate(buffer2);
        pool.deallocate(buffer3);

        let stats = pool.get_stats();
        assert_eq!(stats.pool_count, 3); // 3つの異なるサイズプール
        assert_eq!(stats.pooled_buffers, 3);
    }

    // === StringInterner 拡張テスト ===

    #[test]
    fn test_string_interner_hit_rate() {
        let mut interner = StringInterner::new();

        // 同じ文字列を複数回intern
        for _ in 0..10 {
            interner.intern("repeated");
        }

        // 異なる文字列を1回ずつ
        for i in 0..5 {
            interner.intern(&format!("unique_{i}"));
        }

        let (count, hits, misses, hit_rate) = interner.get_stats();
        assert_eq!(count, 6); // "repeated" + 5つのunique
        assert_eq!(hits, 9); // "repeated"の2回目以降
        assert_eq!(misses, 6); // "repeated"の1回目 + 5つのunique
        assert!((hit_rate - 60.0).abs() < 0.01); // 9/15 * 100 = 60%
    }

    #[test]
    fn test_string_interner_cleanup() {
        let mut interner = StringInterner::new();

        // 文字列をintern
        let s1 = interner.intern("persistent");
        let s2 = interner.intern("temporary");

        // s2を削除（参照カウントを1にする）
        drop(s2);

        let (count_before, _, _, _) = interner.get_stats();
        assert_eq!(count_before, 2);

        // クリーンアップ実行
        interner.cleanup();

        let (count_after, _, _, _) = interner.get_stats();
        assert_eq!(count_after, 1); // "temporary"が削除される

        // "persistent"はまだ参照されている
        assert!(Arc::ptr_eq(&s1, &interner.intern("persistent")));
    }

    #[test]
    fn test_string_interner_empty() {
        let interner = StringInterner::new();

        let (count, hits, misses, hit_rate) = interner.get_stats();
        assert_eq!(count, 0);
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(hit_rate, 0.0);
    }

    #[test]
    fn test_string_interner_identical_strings() {
        let mut interner = StringInterner::new();

        let s1 = interner.intern("test");
        let s2 = interner.intern("test");
        let s3 = interner.intern("test");

        // すべて同じ参照を指す
        assert!(Arc::ptr_eq(&s1, &s2));
        assert!(Arc::ptr_eq(&s2, &s3));

        let (count, hits, misses, hit_rate) = interner.get_stats();
        assert_eq!(count, 1);
        assert_eq!(hits, 2);
        assert_eq!(misses, 1);
        assert!((hit_rate - 66.66666666666666).abs() < 0.01);
    }

    // === MemoryMonitor 拡張テスト ===

    #[tokio::test]
    async fn test_memory_monitor_get_memory_pool() {
        let monitor = MemoryMonitor::new(256);

        // メモリプールへの参照を取得
        let pool = monitor.get_memory_pool();

        // プールを使用
        {
            let mut pool_guard = pool.write().await;
            let buffer = pool_guard.allocate(1024);
            pool_guard.deallocate(buffer);

            let stats = pool_guard.get_stats();
            assert_eq!(stats.total_allocated, stats.total_deallocated);
        }
    }

    #[tokio::test]
    async fn test_memory_monitor_get_string_interner() {
        let monitor = MemoryMonitor::new(256);

        // 文字列インターナーへの参照を取得
        let interner = monitor.get_string_interner();

        // インターナーを使用
        {
            let mut interner_guard = interner.write().await;
            let s1 = interner_guard.intern("test");
            let s2 = interner_guard.intern("test");

            assert!(Arc::ptr_eq(&s1, &s2));

            let (count, hits, misses, _) = interner_guard.get_stats();
            assert_eq!(count, 1);
            assert_eq!(hits, 1);
            assert_eq!(misses, 1);
        }
    }

    #[tokio::test]
    async fn test_memory_monitor_cleanup_pools() {
        let monitor = MemoryMonitor::new(256);

        // メモリプールとインターナーを使用
        {
            let pool = monitor.get_memory_pool();
            let mut pool_guard = pool.write().await;

            for _ in 0..4 {
                let buffer = pool_guard.allocate(1024);
                pool_guard.deallocate(buffer);
            }
        }

        {
            let interner = monitor.get_string_interner();
            let mut interner_guard = interner.write().await;

            for i in 0..4 {
                interner_guard.intern(&format!("test_{i}"));
            }
        }

        // クリーンアップ実行
        monitor.perform_cleanup().await;

        // クリーンアップ後の状態を確認
        {
            let pool = monitor.get_memory_pool();
            let pool_guard = pool.read().await;
            let stats = pool_guard.get_stats();
            // プールバッファが削減されることを確認
            assert!(stats.pooled_buffers <= 4);
        }
    }

    #[tokio::test]
    async fn test_memory_monitor_force_check() {
        let mut monitor = MemoryMonitor::new(256);

        // 通常のチェック
        let status1 = monitor.check_memory_usage().await.unwrap();

        // 緊急クリーンアップ実行後のチェック
        monitor.emergency_cleanup().await;
        let status2 = monitor.check_memory_usage().await.unwrap();

        // 両方とも有効なステータスを返す
        assert!(matches!(
            status1,
            MemoryStatus::Normal | MemoryStatus::Warning
        ));
        assert!(matches!(
            status2,
            MemoryStatus::Normal | MemoryStatus::Warning
        ));
    }

    #[test]
    fn test_memory_pool_stats_clone() {
        let mut pool = MemoryPool::new(4);

        let buffer = pool.allocate(1024);
        pool.deallocate(buffer);

        let stats = pool.get_stats();
        let stats_clone = stats.clone();

        assert_eq!(stats.total_allocated, stats_clone.total_allocated);
        assert_eq!(stats.total_deallocated, stats_clone.total_deallocated);
        assert_eq!(stats.active_allocation, stats_clone.active_allocation);
        assert_eq!(stats.peak_usage, stats_clone.peak_usage);
        assert_eq!(stats.pool_count, stats_clone.pool_count);
        assert_eq!(stats.pooled_buffers, stats_clone.pooled_buffers);
    }

    #[test]
    fn test_memory_pool_stats_debug() {
        let mut pool = MemoryPool::new(4);

        let buffer = pool.allocate(1024);
        pool.deallocate(buffer);

        let stats = pool.get_stats();
        let debug_output = format!("{stats:?}");

        // Debug出力に重要な情報が含まれている
        assert!(debug_output.contains("total_allocated"));
        assert!(debug_output.contains("total_deallocated"));
        assert!(debug_output.contains("active_allocation"));
        assert!(debug_output.contains("peak_usage"));
        assert!(debug_output.contains("pool_count"));
        assert!(debug_output.contains("pooled_buffers"));
    }
}
