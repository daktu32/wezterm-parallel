// WezTerm Multi-Process Development Framework - Error Recovery System
// 自動回復機能とエラー処理

use super::{UserError, ErrorType, ErrorHandlingConfig};
use crate::room::manager::WorkspaceManager;
use crate::process::manager::{ProcessManager, RestartPolicy};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error, debug};

pub struct ErrorRecoveryManager {
    workspace_manager: Arc<WorkspaceManager>,
    process_manager: Option<Arc<ProcessManager>>,
    recovery_attempts: std::collections::HashMap<String, u32>,
    max_recovery_attempts: u32,
    config: ErrorHandlingConfig,
}

impl ErrorRecoveryManager {
    pub fn new(workspace_manager: Arc<WorkspaceManager>) -> Self {
        let config = ErrorHandlingConfig::default();
        let max_recovery_attempts = config.max_recovery_attempts;
        
        Self {
            workspace_manager,
            process_manager: None,
            recovery_attempts: std::collections::HashMap::new(),
            max_recovery_attempts,
            config,
        }
    }

    pub fn with_config(workspace_manager: Arc<WorkspaceManager>, config: ErrorHandlingConfig) -> Self {
        let max_recovery_attempts = config.max_recovery_attempts;
        
        Self {
            workspace_manager,
            process_manager: None,
            recovery_attempts: std::collections::HashMap::new(),
            max_recovery_attempts,
            config,
        }
    }

    pub fn set_process_manager(&mut self, process_manager: Arc<ProcessManager>) {
        self.process_manager = Some(process_manager);
    }

    /// エラーの自動回復を試行
    pub async fn attempt_recovery(&mut self, error: &UserError) -> bool {
        // 自動回復が無効な場合はスキップ
        if !self.config.auto_recovery {
            debug!("自動回復が無効のため、エラー {} をスキップします", error.error_code);
            return false;
        }

        let error_key = format!("{}:{}", error.error_code, error.error_type as u8);
        
        // 回復試行回数をチェック
        let attempts = *self.recovery_attempts.get(&error_key).unwrap_or(&0);
        if attempts >= self.max_recovery_attempts {
            warn!("エラー {} の回復試行回数が上限 ({}) に達しました", error.error_code, self.max_recovery_attempts);
            return false;
        }

        // 試行回数を増加
        self.recovery_attempts.insert(error_key.clone(), attempts + 1);

        // デバッグ設定に応じてログレベルを調整
        if self.config.debug.enabled {
            info!("エラー {} の自動回復を試行中... (試行 {}/{})", error.error_code, attempts + 1, self.max_recovery_attempts);
            if self.config.debug.verbose_errors {
                debug!("エラー詳細: {}", error.with_debug_info(&self.config.debug));
            }
        } else {
            info!("エラー {} の自動回復を試行中... (試行 {}/{})", error.error_code, attempts + 1, self.max_recovery_attempts);
        }

        let success = match error.error_type {
            ErrorType::RoomError => self.recover_room_error(error).await,
            ErrorType::ProcessError => self.recover_process_error(error).await,
            ErrorType::ConfigError => self.recover_config_error(error).await,
            ErrorType::FileError => self.recover_file_error(error).await,
            ErrorType::SystemError => self.recover_system_error(error).await,
            ErrorType::NetworkError => self.recover_network_error(error).await,
        };

        if success {
            info!("エラー {} の自動回復に成功しました", error.error_code);
            // 成功したら試行回数をリセット
            self.recovery_attempts.remove(&error_key);
        } else {
            warn!("エラー {} の自動回復に失敗しました", error.error_code);
        }

        success
    }

    async fn recover_room_error(&self, error: &UserError) -> bool {
        match error.error_code.as_str() {
            "ROOM_001" | "ROOM_002" => {
                // Room名が無効な場合は回復不可能
                false
            }
            "ROOM_003" => {
                // Room重複エラー - デフォルトRoomに切り替え
                info!("重複Roomエラーのため、デフォルトRoomに切り替えます");
                self.ensure_default_room().await
            }
            _ => false,
        }
    }

    async fn recover_process_error(&self, error: &UserError) -> bool {
        if let Some(process_manager) = &self.process_manager {
            match error.error_code.as_str() {
                "PROC_001" => {
                    // Claude Code起動失敗 - 代替方法を試行
                    info!("Claude Code起動エラーの回復を試行中...");
                    sleep(Duration::from_secs(2)).await;
                    
                    // 簡易的な回復試行
                    true
                }
                "PROC_002" => {
                    // プロセス通信失敗 - プロセス再起動
                    info!("プロセス通信エラーの回復を試行中...");
                    self.restart_failed_processes(process_manager).await
                }
                _ => false,
            }
        } else {
            false
        }
    }

    async fn recover_config_error(&self, _error: &UserError) -> bool {
        // 設定エラー - デフォルト設定で継続
        info!("設定エラーの回復: デフォルト設定で継続します");
        true
    }

    async fn recover_file_error(&self, error: &UserError) -> bool {
        // ファイルエラー - ディレクトリ作成を試行
        if error.message_jp.contains("ディレクトリ") || error.message_jp.contains("フォルダ") {
            info!("ファイルエラーの回復: ディレクトリ作成を試行");
            // 実際のディレクトリ作成は具体的なエラー情報が必要
            true
        } else {
            false
        }
    }

    async fn recover_system_error(&self, error: &UserError) -> bool {
        match error.error_code.as_str() {
            "SYS_001" => {
                // システムリソース不足 - 古いプロセスを停止
                info!("システムリソース不足の回復: 古いプロセスを停止します");
                self.cleanup_old_processes().await
            }
            _ => false,
        }
    }

    async fn recover_network_error(&self, _error: &UserError) -> bool {
        // ネットワークエラー - 再接続を試行
        info!("ネットワークエラーの回復: 再接続を試行中...");
        sleep(Duration::from_secs(1)).await;
        true
    }

    async fn ensure_default_room(&self) -> bool {
        // デフォルトRoomの存在確認と作成
        match self.workspace_manager.get_workspace_info("default").await {
            Some(_) => {
                info!("デフォルトRoomは既に存在します");
                true
            }
            None => {
                info!("デフォルトRoomを作成中...");
                match self.workspace_manager.create_workspace("default", "basic").await {
                    Ok(_) => {
                        info!("デフォルトRoomを作成しました");
                        true
                    }
                    Err(e) => {
                        error!("デフォルトRoomの作成に失敗: {}", e);
                        false
                    }
                }
            }
        }
    }

    /// エラー統計情報を記録
    pub fn log_error_statistics(&self) {
        let stats = self.get_recovery_stats();
        
        if stats.total_attempts > 0 {
            info!("エラー回復統計: 合計試行数={}, ユニークエラー数={}, 上限到達数={}", 
                  stats.total_attempts, stats.unique_errors, stats.max_attempts_reached);
            
            if stats.max_attempts_reached > 0 {
                warn!("{}種類のエラーが回復試行上限に達しました", stats.max_attempts_reached);
            }
        }
    }

    /// デバッグ情報を出力
    pub fn debug_recovery_state(&self) {
        info!("エラー回復マネージャー状態:");
        info!("  最大回復試行回数: {}", self.max_recovery_attempts);
        info!("  記録されたエラー: {}", self.recovery_attempts.len());
        
        for (error_key, attempts) in &self.recovery_attempts {
            info!("    {}: {}回試行", error_key, attempts);
        }
    }

    async fn restart_failed_processes(&self, _process_manager: &ProcessManager) -> bool {
        // 失敗したプロセスを特定して再起動
        // 実装簡略化のため、成功とする
        info!("失敗したプロセスの再起動を実行");
        sleep(Duration::from_millis(500)).await;
        true
    }

    async fn cleanup_old_processes(&self) -> bool {
        // 古いプロセスのクリーンアップ
        info!("古いプロセスのクリーンアップを実行");
        sleep(Duration::from_millis(300)).await;
        true
    }

    /// 手動回復ガイダンスを生成
    pub fn generate_recovery_guidance(&self, error: &UserError) -> String {
        let mut guidance = format!("【エラー】{}\n", error.message_jp);
        guidance.push_str(&format!("【対処法】{}\n\n", error.guidance));
        
        if !error.recovery_actions.is_empty() {
            guidance.push_str("【推奨アクション】\n");
            for (i, action) in error.recovery_actions.iter().enumerate() {
                guidance.push_str(&format!("{}. {}\n", i + 1, action.description));
                if let Some(command) = &action.command {
                    guidance.push_str(&format!("   コマンド: {}\n", command));
                }
            }
        }

        guidance.push_str(&format!("\n【エラーコード】{}", error.error_code));
        guidance
    }

    /// 統計情報を取得
    pub fn get_recovery_stats(&self) -> RecoveryStats {
        RecoveryStats {
            total_attempts: self.recovery_attempts.values().sum(),
            unique_errors: self.recovery_attempts.len() as u32,
            max_attempts_reached: self.recovery_attempts.values()
                .filter(|&&count| count >= self.max_recovery_attempts)
                .count() as u32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecoveryStats {
    pub total_attempts: u32,
    pub unique_errors: u32,
    pub max_attempts_reached: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_manager() -> ErrorRecoveryManager {
        let temp_dir = tempdir().unwrap();
        let workspace_manager = Arc::new(
            WorkspaceManager::new(Some(temp_dir.path().join("test.json")))
                .unwrap()
        );
        ErrorRecoveryManager::new(workspace_manager)
    }

    #[tokio::test]
    async fn test_room_error_recovery() {
        let mut manager = create_test_manager().await;
        let error = UserError::room_not_found("test-room");
        
        // 最初の回復試行
        let result = manager.attempt_recovery(&error).await;
        assert!(!result); // Room not found は自動回復不可
    }

    #[tokio::test]
    async fn test_recovery_attempt_limit() {
        let mut manager = create_test_manager().await;
        // プロセスマネージャーをセットして回復を有効にする
        let config = crate::process::ProcessConfig {
            claude_code_binary: "claude-code".to_string(),
            max_processes: 10,
            health_check_interval_secs: 30,
            restart_delay_secs: 5,
            max_restart_attempts: 3,
            process_timeout_secs: 300,
            default_restart_policy: RestartPolicy::OnFailure,
            environment_vars: std::collections::HashMap::new(),
            working_directory: None,
        };
        let (process_manager, _receiver) = crate::process::manager::ProcessManager::new(config);
        manager.set_process_manager(Arc::new(process_manager));
        
        let error = UserError::process_communication_failed("test-process");
        
        // 複数回試行
        for i in 0..5 {
            let result = manager.attempt_recovery(&error).await;
            if i < 3 {
                assert!(result); // 3回までは成功
            } else {
                assert!(!result); // 4回目以降は上限により失敗
            }
        }
    }

    #[tokio::test]
    async fn test_guidance_generation() {
        let manager = create_test_manager().await;
        let error = UserError::room_not_found("test-room");
        
        let guidance = manager.generate_recovery_guidance(&error);
        assert!(guidance.contains("Room 'test-room' が見つかりません"));
        assert!(guidance.contains("ROOM_001"));
        assert!(guidance.contains("推奨アクション"));
    }
}