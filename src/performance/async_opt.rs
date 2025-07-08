// WezTerm Multi-Process Development Framework - Async Performance Optimization
// 非同期処理パフォーマンス最適化

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// Type alias for complex async execution function
type AsyncExecutionFn<'a> = std::pin::Pin<
    Box<
        dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + 'a,
    >,
>;

/// Type alias for batch processor function
type BatchProcessorFn<T> = Arc<
    dyn Fn(Vec<T>) -> BoxFuture<'static, Result<(), Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + Sync,
>;

/// 非同期タスクプール
pub struct AsyncTaskPool {
    pool_size: usize,
    active_tasks: Arc<RwLock<Vec<JoinHandle<()>>>>,
    task_queue: Arc<RwLock<VecDeque<Box<dyn AsyncTask + Send + Sync>>>>,
    semaphore: Arc<Semaphore>,
    stats: Arc<RwLock<AsyncStats>>,
}

/// 非同期タスクトレイト
pub trait AsyncTask: Send + Sync {
    fn execute(&self) -> AsyncExecutionFn<'_>;
    fn priority(&self) -> TaskPriority;
    fn estimated_duration(&self) -> Duration;
    fn task_type(&self) -> &'static str;
}

/// タスク優先度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// 非同期処理統計
#[derive(Debug, Clone, Default)]
pub struct AsyncStats {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_execution_time: Duration,
    pub peak_concurrent_tasks: usize,
    pub current_queue_size: usize,
}

impl AsyncTaskPool {
    pub fn new(pool_size: usize) -> Self {
        info!("非同期タスクプール初期化: サイズ={}", pool_size);

        Self {
            pool_size,
            active_tasks: Arc::new(RwLock::new(Vec::new())),
            task_queue: Arc::new(RwLock::new(VecDeque::new())),
            semaphore: Arc::new(Semaphore::new(pool_size)),
            stats: Arc::new(RwLock::new(AsyncStats::default())),
        }
    }

    /// タスクをキューに追加
    pub async fn submit_task(&self, task: Box<dyn AsyncTask + Send + Sync>) {
        {
            let mut queue = self.task_queue.write().await;
            queue.push_back(task);

            // 優先度によるソート
            let mut sorted_queue: Vec<_> = queue.drain(..).collect();
            sorted_queue.sort_by_key(|b| std::cmp::Reverse(b.priority()));
            queue.extend(sorted_queue);

            let mut stats = self.stats.write().await;
            stats.total_tasks += 1;
            stats.current_queue_size = queue.len();
        }

        self.process_queue().await;
    }

    /// キューを処理
    async fn process_queue(&self) {
        let permit = match self.semaphore.try_acquire() {
            Ok(permit) => permit,
            Err(_) => {
                debug!("タスクプールが満杯、キューで待機中");
                return;
            }
        };

        let task = {
            let mut queue = self.task_queue.write().await;
            queue.pop_front()
        };

        if let Some(task) = task {
            let stats_ref = Arc::clone(&self.stats);

            // タスクを実行し、実行時間を測定
            let start_time = Instant::now();
            let task_type = task.task_type().to_string();

            debug!("タスク実行開始: {}", task_type);

            match task.execute().await {
                Ok(_) => {
                    let duration = start_time.elapsed();
                    debug!("タスク完了: {} ({:?})", task_type, duration);

                    let mut stats = stats_ref.write().await;
                    stats.completed_tasks += 1;
                    stats.average_execution_time = Duration::from_nanos(
                        ((stats.average_execution_time.as_nanos() as u64
                            * (stats.completed_tasks - 1)
                            + duration.as_nanos() as u64)
                            / stats.completed_tasks) as u64,
                    );
                }
                Err(e) => {
                    warn!("タスク失敗: {} - {}", task_type, e);
                    let mut stats = stats_ref.write().await;
                    stats.failed_tasks += 1;
                }
            }

            drop(permit);

            // 完了したタスクをクリーンアップ
            self.cleanup_completed_tasks().await;
        }
    }

    /// 完了したタスクをクリーンアップ
    async fn cleanup_completed_tasks(&self) {
        let mut active_tasks = self.active_tasks.write().await;
        active_tasks.retain(|handle| !handle.is_finished());
    }

    /// 全タスクの完了を待機
    pub async fn wait_for_completion(&self) {
        loop {
            let has_active_tasks = {
                let active_tasks = self.active_tasks.read().await;
                !active_tasks.is_empty()
            };

            let queue_size = {
                let queue = self.task_queue.read().await;
                queue.len()
            };

            if !has_active_tasks && queue_size == 0 {
                break;
            }

            // 少し待機してから再チェック
            tokio::time::sleep(Duration::from_millis(100)).await;

            // 完了したタスクをクリーンアップ
            self.cleanup_completed_tasks().await;
        }
    }

    /// 統計情報を取得
    pub async fn get_stats(&self) -> AsyncStats {
        let stats = self.stats.read().await;
        let mut stats_copy = stats.clone();

        // 現在のキューサイズを更新
        let queue = self.task_queue.read().await;
        stats_copy.current_queue_size = queue.len();

        stats_copy
    }

    /// パフォーマンスレポートを生成
    pub async fn generate_report(&self) -> String {
        let stats = self.get_stats().await;
        let success_rate = if stats.total_tasks > 0 {
            (stats.completed_tasks as f64 / stats.total_tasks as f64) * 100.0
        } else {
            0.0
        };

        format!(
            "=== 非同期処理パフォーマンスレポート ===\n\
            プールサイズ: {}\n\
            総タスク数: {}\n\
            完了タスク: {}\n\
            失敗タスク: {}\n\
            成功率: {:.1}%\n\
            平均実行時間: {:?}\n\
            ピーク同時実行数: {}\n\
            現在のキューサイズ: {}",
            self.pool_size,
            stats.total_tasks,
            stats.completed_tasks,
            stats.failed_tasks,
            success_rate,
            stats.average_execution_time,
            stats.peak_concurrent_tasks,
            stats.current_queue_size
        )
    }
}

/// バッチ処理最適化
pub struct BatchProcessor<T> {
    batch_size: usize,
    flush_interval: Duration,
    buffer: Arc<RwLock<Vec<T>>>,
    processor: BatchProcessorFn<T>,
    flush_handle: Option<JoinHandle<()>>,
}

type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

impl<T: Send + Sync + 'static> BatchProcessor<T> {
    pub fn new<F, Fut>(batch_size: usize, flush_interval: Duration, processor: F) -> Self
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + 'static,
    {
        let processor =
            Arc::new(
                move |items: Vec<T>| -> BoxFuture<
                    'static,
                    Result<(), Box<dyn std::error::Error + Send + Sync>>,
                > { Box::pin(processor(items)) },
            );

        Self {
            batch_size,
            flush_interval,
            buffer: Arc::new(RwLock::new(Vec::new())),
            processor,
            flush_handle: None,
        }
    }

    /// バッチ処理を開始
    pub fn start(&mut self) {
        let buffer = Arc::clone(&self.buffer);
        let processor = Arc::clone(&self.processor);
        let _batch_size = self.batch_size;
        let flush_interval = self.flush_interval;

        self.flush_handle = Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(flush_interval);

            loop {
                interval.tick().await;

                let items_to_process = {
                    let mut buffer = buffer.write().await;
                    if buffer.is_empty() {
                        continue;
                    }
                    std::mem::take(&mut *buffer)
                };

                if !items_to_process.is_empty() {
                    debug!("バッチ処理実行: {} アイテム", items_to_process.len());
                    if let Err(e) = processor(items_to_process).await {
                        warn!("バッチ処理エラー: {}", e);
                    }
                }
            }
        }));
    }

    /// アイテムを追加
    pub async fn add_item(&self, item: T) {
        let should_flush = {
            let mut buffer = self.buffer.write().await;
            buffer.push(item);
            buffer.len() >= self.batch_size
        };

        if should_flush {
            self.flush().await;
        }
    }

    /// 即座にフラッシュ
    pub async fn flush(&self) {
        let items_to_process = {
            let mut buffer = self.buffer.write().await;
            if buffer.is_empty() {
                return;
            }
            std::mem::take(&mut *buffer)
        };

        debug!("手動フラッシュ実行: {} アイテム", items_to_process.len());
        if let Err(e) = (self.processor)(items_to_process).await {
            warn!("フラッシュ処理エラー: {}", e);
        }
    }

    /// 停止
    pub async fn stop(&mut self) {
        // 残りのアイテムをフラッシュ
        self.flush().await;

        if let Some(handle) = self.flush_handle.take() {
            handle.abort();
        }
    }
}

/// スレッドプール監視
pub struct ThreadPoolMonitor {
    monitor_interval: Duration,
    monitor_handle: Option<JoinHandle<()>>,
    stats_sender: mpsc::UnboundedSender<ThreadPoolStats>,
}

#[derive(Debug, Clone)]
pub struct ThreadPoolStats {
    pub active_threads: usize,
    pub queued_tasks: usize,
    pub completed_tasks: u64,
    pub cpu_utilization: f64,
    pub timestamp: Instant,
}

impl ThreadPoolMonitor {
    pub fn new(monitor_interval: Duration) -> (Self, mpsc::UnboundedReceiver<ThreadPoolStats>) {
        let (stats_sender, stats_receiver) = mpsc::unbounded_channel();

        (
            Self {
                monitor_interval,
                monitor_handle: None,
                stats_sender,
            },
            stats_receiver,
        )
    }

    /// 監視を開始
    pub fn start_monitoring(&mut self, task_pool: Arc<AsyncTaskPool>) {
        let stats_sender = self.stats_sender.clone();
        let monitor_interval = self.monitor_interval;

        self.monitor_handle = Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(monitor_interval);

            loop {
                interval.tick().await;

                let async_stats = task_pool.get_stats().await;
                let stats = ThreadPoolStats {
                    active_threads: async_stats.peak_concurrent_tasks,
                    queued_tasks: async_stats.current_queue_size,
                    completed_tasks: async_stats.completed_tasks,
                    cpu_utilization: 0.0, // TODO: 実際のCPU使用率を取得
                    timestamp: Instant::now(),
                };

                if stats_sender.send(stats).is_err() {
                    debug!("スレッドプール監視停止: レシーバーが閉じられました");
                    break;
                }
            }
        }));
    }

    /// 監視を停止
    pub fn stop_monitoring(&mut self) {
        if let Some(handle) = self.monitor_handle.take() {
            handle.abort();
        }
    }
}

// 具体的なタスク実装例

/// ファイル処理タスク
pub struct FileProcessingTask {
    file_path: String,
    operation: String,
}

impl FileProcessingTask {
    pub fn new(file_path: String, operation: String) -> Self {
        Self {
            file_path,
            operation,
        }
    }
}

impl AsyncTask for FileProcessingTask {
    fn execute(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            debug!("ファイル処理実行: {} ({})", self.file_path, self.operation);

            // 実際のファイル処理をシミュレート
            tokio::time::sleep(Duration::from_millis(50)).await;

            Ok(())
        })
    }

    fn priority(&self) -> TaskPriority {
        match self.operation.as_str() {
            "critical" => TaskPriority::Critical,
            "high" => TaskPriority::High,
            "normal" => TaskPriority::Normal,
            _ => TaskPriority::Low,
        }
    }

    fn estimated_duration(&self) -> Duration {
        Duration::from_millis(50)
    }

    fn task_type(&self) -> &'static str {
        "file_processing"
    }
}

/// データベース操作タスク
pub struct DatabaseTask {
    query: String,
    priority_level: TaskPriority,
}

impl DatabaseTask {
    pub fn new(query: String, priority_level: TaskPriority) -> Self {
        Self {
            query,
            priority_level,
        }
    }
}

impl AsyncTask for DatabaseTask {
    fn execute(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            debug!("データベース操作実行: {}", self.query);

            // データベース操作をシミュレート
            tokio::time::sleep(Duration::from_millis(30)).await;

            Ok(())
        })
    }

    fn priority(&self) -> TaskPriority {
        self.priority_level
    }

    fn estimated_duration(&self) -> Duration {
        Duration::from_millis(30)
    }

    fn task_type(&self) -> &'static str {
        "database_operation"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_task_pool() {
        let pool = AsyncTaskPool::new(2);

        let task1 = Box::new(FileProcessingTask::new(
            "test1.txt".to_string(),
            "normal".to_string(),
        ));

        let task2 = Box::new(DatabaseTask::new(
            "SELECT * FROM test".to_string(),
            TaskPriority::High,
        ));

        pool.submit_task(task1).await;
        pool.submit_task(task2).await;

        pool.wait_for_completion().await;

        let stats = pool.get_stats().await;
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.completed_tasks, 2);
    }

    #[tokio::test]
    async fn test_batch_processor() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);

        let mut processor = BatchProcessor::new(
            3,                          // バッチサイズ
            Duration::from_millis(100), // フラッシュ間隔
            move |items: Vec<i32>| {
                let tx = tx.clone();
                async move {
                    tx.send(items.len()).await.unwrap();
                    Ok(())
                }
            },
        );

        processor.start();

        // アイテムを追加
        processor.add_item(1).await;
        processor.add_item(2).await;
        processor.add_item(3).await; // バッチサイズに達するのでフラッシュ

        // バッチが処理されることを確認
        let batch_size = rx.recv().await.unwrap();
        assert_eq!(batch_size, 3);

        processor.stop().await;
    }

    #[tokio::test]
    async fn test_thread_pool_monitor() {
        let pool = Arc::new(AsyncTaskPool::new(2));
        let (mut monitor, mut stats_receiver) = ThreadPoolMonitor::new(Duration::from_millis(50));

        monitor.start_monitoring(Arc::clone(&pool));

        // 統計を受信
        tokio::time::timeout(Duration::from_millis(100), stats_receiver.recv())
            .await
            .unwrap()
            .unwrap();

        monitor.stop_monitoring();
    }
}
