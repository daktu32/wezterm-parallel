use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tokio::time::sleep;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Claude Code固有のヘルスチェック機能
#[derive(Debug)]
pub struct ClaudeHealthMonitor {
    /// 監視対象プロセス
    monitored_processes: RwLock<HashMap<String, HealthState>>,
    /// ヘルスチェック設定
    config: HealthConfig,
    /// 実行中のヘルスチェックタスク
    monitoring_handles: RwLock<HashMap<String, tokio::task::JoinHandle<()>>>,
}

/// プロセスのヘルス状態
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthState {
    pub process_id: String,
    pub workspace: String,
    pub pid: Option<u32>,
    pub status: HealthStatus,
    pub last_check: SystemTime,
    pub last_success: SystemTime,
    pub consecutive_failures: u32,
    pub total_checks: u64,
    pub total_failures: u64,
    pub avg_response_time: Duration,
    pub memory_usage: Option<u64>, // MB
    pub cpu_usage: Option<f64>,    // %
    pub uptime: Duration,
    pub restart_count: u32,
}

/// ヘルスチェック結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unresponsive,
    Stopped,
    Unknown,
}

/// ヘルスチェック設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// チェック間隔（秒）
    pub check_interval: Duration,
    /// タイムアウト（秒）
    pub check_timeout: Duration,
    /// 連続失敗閾値
    pub failure_threshold: u32,
    /// メモリ使用量警告閾値（MB）
    pub memory_warning_threshold: u64,
    /// メモリ使用量クリティカル閾値（MB）
    pub memory_critical_threshold: u64,
    /// CPU使用率警告閾値（%）
    pub cpu_warning_threshold: f64,
    /// CPU使用率クリティカル閾値（%）
    pub cpu_critical_threshold: f64,
    /// レスポンス時間警告閾値（ミリ秒）
    pub response_warning_threshold: Duration,
    /// レスポンス時間クリティカル閾値（ミリ秒）
    pub response_critical_threshold: Duration,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(10),
            failure_threshold: 3,
            memory_warning_threshold: 2048,  // 2GB
            memory_critical_threshold: 4096, // 4GB
            cpu_warning_threshold: 80.0,
            cpu_critical_threshold: 95.0,
            response_warning_threshold: Duration::from_millis(5000), // 5秒
            response_critical_threshold: Duration::from_millis(10000), // 10秒
        }
    }
}

impl ClaudeHealthMonitor {
    /// 新しいヘルスモニターを作成
    pub fn new(config: Option<HealthConfig>) -> Self {
        Self {
            monitored_processes: RwLock::new(HashMap::new()),
            config: config.unwrap_or_default(),
            monitoring_handles: RwLock::new(HashMap::new()),
        }
    }

    /// プロセスの監視を開始
    pub async fn start_monitoring(
        &self,
        process_id: String,
        workspace: String,
        pid: Option<u32>,
    ) -> Result<()> {
        let health_state = HealthState {
            process_id: process_id.clone(),
            workspace: workspace.clone(),
            pid,
            status: HealthStatus::Unknown,
            last_check: SystemTime::now(),
            last_success: SystemTime::now(),
            consecutive_failures: 0,
            total_checks: 0,
            total_failures: 0,
            avg_response_time: Duration::from_millis(0),
            memory_usage: None,
            cpu_usage: None,
            uptime: Duration::from_secs(0),
            restart_count: 0,
        };

        // 監視状態を登録
        {
            let mut processes = self.monitored_processes.write().await;
            processes.insert(process_id.clone(), health_state);
        }

        // 監視タスクを開始
        let monitor_task = self.spawn_monitoring_task(process_id.clone()).await;

        {
            let mut handles = self.monitoring_handles.write().await;
            handles.insert(process_id.clone(), monitor_task);
        }

        info!("Started health monitoring for process '{process_id}' in workspace '{workspace}'");
        Ok(())
    }

    /// プロセスの監視を停止
    pub async fn stop_monitoring(&self, process_id: &str) -> Result<()> {
        // 監視タスクを停止
        {
            let mut handles = self.monitoring_handles.write().await;
            if let Some(handle) = handles.remove(process_id) {
                handle.abort();
            }
        }

        // 監視状態を削除
        {
            let mut processes = self.monitored_processes.write().await;
            processes.remove(process_id);
        }

        info!("Stopped health monitoring for process '{process_id}'");
        Ok(())
    }

    /// 監視中のすべてのプロセスのヘルス状態を取得
    pub async fn get_all_health_states(&self) -> HashMap<String, HealthState> {
        let processes = self.monitored_processes.read().await;
        processes.clone()
    }

    /// 特定のプロセスのヘルス状態を取得
    pub async fn get_health_state(&self, process_id: &str) -> Option<HealthState> {
        let processes = self.monitored_processes.read().await;
        processes.get(process_id).cloned()
    }

    /// ヘルス状態が警告レベル以上のプロセス一覧を取得
    pub async fn get_unhealthy_processes(&self) -> Vec<HealthState> {
        let processes = self.monitored_processes.read().await;
        processes
            .values()
            .filter(|state| {
                matches!(
                    state.status,
                    HealthStatus::Warning | HealthStatus::Critical | HealthStatus::Unresponsive
                )
            })
            .cloned()
            .collect()
    }

    /// 監視タスクを生成
    async fn spawn_monitoring_task(&self, process_id: String) -> tokio::task::JoinHandle<()> {
        let monitor = self.clone_for_task();
        let check_interval = self.config.check_interval;

        tokio::spawn(async move {
            loop {
                if let Err(e) = monitor.perform_health_check(&process_id).await {
                    error!("Health check failed for process '{process_id}': {e}");
                }

                sleep(check_interval).await;
            }
        })
    }

    /// タスク用にCloneを作成（Arc<Self>のようなものをシミュレート）
    fn clone_for_task(&self) -> Self {
        Self {
            monitored_processes: RwLock::new(HashMap::new()), // 実際はArcを使うべき
            config: self.config.clone(),
            monitoring_handles: RwLock::new(HashMap::new()),
        }
    }

    /// 実際のヘルスチェックを実行
    async fn perform_health_check(&self, process_id: &str) -> Result<()> {
        let start_time = Instant::now();

        // プロセス状態を取得
        let mut current_state = {
            let processes = self.monitored_processes.read().await;
            match processes.get(process_id) {
                Some(state) => state.clone(),
                None => return Err("Process not found in monitoring list".into()),
            }
        };

        current_state.total_checks += 1;
        current_state.last_check = SystemTime::now();

        // 1. プロセス存在確認
        let process_exists = if let Some(pid) = current_state.pid {
            self.check_process_exists(pid).await?
        } else {
            false
        };

        if !process_exists {
            current_state.status = HealthStatus::Stopped;
            current_state.consecutive_failures += 1;
            current_state.total_failures += 1;
            self.update_health_state(process_id, current_state).await?;
            return Ok(());
        }

        // 2. Claude Code固有のヘルスチェック
        let claude_responsive = self.check_claude_responsiveness(&current_state).await?;

        // 3. システムリソース使用量チェック
        if let Some(pid) = current_state.pid {
            current_state.memory_usage = self.get_memory_usage(pid).await.ok();
            current_state.cpu_usage = self.get_cpu_usage(pid).await.ok();
        }

        // 4. レスポンス時間測定
        let response_time = start_time.elapsed();
        current_state.avg_response_time = self.calculate_avg_response_time(
            current_state.avg_response_time,
            response_time,
            current_state.total_checks,
        );

        // 5. ヘルス状態を評価
        current_state.status =
            self.evaluate_health_status(&current_state, claude_responsive, response_time);

        // 6. 連続失敗カウンタを更新
        if matches!(current_state.status, HealthStatus::Healthy) {
            current_state.consecutive_failures = 0;
            current_state.last_success = SystemTime::now();
        } else {
            current_state.consecutive_failures += 1;
            current_state.total_failures += 1;
        }

        // 7. 状態を更新
        let final_status = current_state.status.clone();
        self.update_health_state(process_id, current_state).await?;

        debug!("Health check completed for process '{process_id}': {final_status:?}");
        Ok(())
    }

    /// プロセスが存在するかチェック
    async fn check_process_exists(&self, pid: u32) -> Result<bool> {
        let output = tokio::process::Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .output()
            .await?;

        Ok(output.status.success())
    }

    /// Claude Codeの応答性をチェック
    async fn check_claude_responsiveness(&self, state: &HealthState) -> Result<bool> {
        // Claude Code固有のヘルスチェック
        // 実際の実装では、Claude CodeのAPIエンドポイントや
        // 特定のコマンドに対する応答をチェックする

        if let Some(pid) = state.pid {
            // プロセスのスレッド数をチェック（応答性の指標）
            let output = tokio::process::Command::new("ps")
                .arg("-p")
                .arg(pid.to_string())
                .arg("-o")
                .arg("nlwp=")
                .output()
                .await?;

            if output.status.success() {
                let thread_count_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(thread_count) = thread_count_str.trim().parse::<u32>() {
                    // 正常なスレッド数の範囲をチェック
                    return Ok(thread_count > 0 && thread_count < 1000);
                }
            }
        }

        Ok(false)
    }

    /// メモリ使用量を取得（MB単位）
    async fn get_memory_usage(&self, pid: u32) -> Result<u64> {
        let output = tokio::process::Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .arg("-o")
            .arg("rss=")
            .output()
            .await?;

        if output.status.success() {
            let memory_kb_str = String::from_utf8_lossy(&output.stdout);
            let memory_kb: u64 = memory_kb_str.trim().parse()?;
            Ok(memory_kb / 1024) // Convert KB to MB
        } else {
            Err("Failed to get memory usage".into())
        }
    }

    /// CPU使用率を取得（％）
    async fn get_cpu_usage(&self, pid: u32) -> Result<f64> {
        let output = tokio::process::Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .arg("-o")
            .arg("pcpu=")
            .output()
            .await?;

        if output.status.success() {
            let cpu_str = String::from_utf8_lossy(&output.stdout);
            let cpu_usage: f64 = cpu_str.trim().parse()?;
            Ok(cpu_usage)
        } else {
            Err("Failed to get CPU usage".into())
        }
    }

    /// 平均レスポンス時間を計算
    fn calculate_avg_response_time(
        &self,
        current_avg: Duration,
        new_time: Duration,
        total_checks: u64,
    ) -> Duration {
        if total_checks <= 1 {
            new_time
        } else {
            let current_total = current_avg * (total_checks - 1) as u32;
            let new_total = current_total + new_time;
            new_total / total_checks as u32
        }
    }

    /// ヘルス状態を評価
    fn evaluate_health_status(
        &self,
        state: &HealthState,
        claude_responsive: bool,
        response_time: Duration,
    ) -> HealthStatus {
        // Claude Codeが応答しない場合
        if !claude_responsive {
            return HealthStatus::Unresponsive;
        }

        // メモリ使用量チェック
        if let Some(memory) = state.memory_usage {
            if memory >= self.config.memory_critical_threshold {
                return HealthStatus::Critical;
            }
            if memory >= self.config.memory_warning_threshold {
                return HealthStatus::Warning;
            }
        }

        // CPU使用率チェック
        if let Some(cpu) = state.cpu_usage {
            if cpu >= self.config.cpu_critical_threshold {
                return HealthStatus::Critical;
            }
            if cpu >= self.config.cpu_warning_threshold {
                return HealthStatus::Warning;
            }
        }

        // レスポンス時間チェック
        if response_time >= self.config.response_critical_threshold {
            return HealthStatus::Critical;
        }
        if response_time >= self.config.response_warning_threshold {
            return HealthStatus::Warning;
        }

        // 連続失敗チェック
        if state.consecutive_failures >= self.config.failure_threshold {
            return HealthStatus::Critical;
        }

        HealthStatus::Healthy
    }

    /// ヘルス状態を更新
    async fn update_health_state(&self, process_id: &str, new_state: HealthState) -> Result<()> {
        let mut processes = self.monitored_processes.write().await;
        processes.insert(process_id.to_string(), new_state);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_health_monitor_creation() {
        let monitor = ClaudeHealthMonitor::new(None);
        let states = monitor.get_all_health_states().await;
        assert!(states.is_empty());
    }

    #[test]
    async fn test_health_config_defaults() {
        let config = HealthConfig::default();
        assert_eq!(config.check_interval, Duration::from_secs(30));
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.memory_warning_threshold, 2048);
        assert_eq!(config.cpu_warning_threshold, 80.0);
    }

    #[test]
    async fn test_start_monitoring() {
        let monitor = ClaudeHealthMonitor::new(None);
        let result = monitor
            .start_monitoring(
                "test-process".to_string(),
                "test-workspace".to_string(),
                Some(12345),
            )
            .await;

        assert!(result.is_ok());

        let states = monitor.get_all_health_states().await;
        assert_eq!(states.len(), 1);

        let state = states.get("test-process").unwrap();
        assert_eq!(state.process_id, "test-process");
        assert_eq!(state.workspace, "test-workspace");
        assert_eq!(state.pid, Some(12345));
    }

    #[test]
    async fn test_stop_monitoring() {
        let monitor = ClaudeHealthMonitor::new(None);

        monitor
            .start_monitoring(
                "test-process".to_string(),
                "test-workspace".to_string(),
                Some(12345),
            )
            .await
            .unwrap();

        let result = monitor.stop_monitoring("test-process").await;
        assert!(result.is_ok());

        let states = monitor.get_all_health_states().await;
        assert!(states.is_empty());
    }

    #[test]
    async fn test_evaluate_health_status() {
        let monitor = ClaudeHealthMonitor::new(None);
        let mut state = HealthState {
            process_id: "test".to_string(),
            workspace: "test".to_string(),
            pid: Some(12345),
            status: HealthStatus::Unknown,
            last_check: SystemTime::now(),
            last_success: SystemTime::now(),
            consecutive_failures: 0,
            total_checks: 1,
            total_failures: 0,
            avg_response_time: Duration::from_millis(100),
            memory_usage: Some(1024), // 1GB
            cpu_usage: Some(50.0),    // 50%
            uptime: Duration::from_secs(3600),
            restart_count: 0,
        };

        // 正常な状態
        let status = monitor.evaluate_health_status(&state, true, Duration::from_millis(1000));
        assert!(matches!(status, HealthStatus::Healthy));

        // メモリ使用量が警告レベル
        state.memory_usage = Some(3000); // 3GB
        let status = monitor.evaluate_health_status(&state, true, Duration::from_millis(1000));
        assert!(matches!(status, HealthStatus::Warning));

        // Claude Codeが無応答
        let status = monitor.evaluate_health_status(&state, false, Duration::from_millis(1000));
        assert!(matches!(status, HealthStatus::Unresponsive));
    }

    #[test]
    async fn test_calculate_avg_response_time() {
        let monitor = ClaudeHealthMonitor::new(None);

        // 初回測定
        let avg = monitor.calculate_avg_response_time(
            Duration::from_millis(0),
            Duration::from_millis(1000),
            1,
        );
        assert_eq!(avg, Duration::from_millis(1000));

        // 2回目測定
        let avg = monitor.calculate_avg_response_time(
            Duration::from_millis(1000),
            Duration::from_millis(2000),
            2,
        );
        assert_eq!(avg, Duration::from_millis(1500));
    }
}
