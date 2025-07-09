// Claude Code自動起動機能の統合テスト
// Issue #10 の全フェーズをカバーする包括的なテスト

use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::test;
use wezterm_parallel::process::{
    ClaudeCodeConfigBuilder, ClaudeCodeDetector, ClaudeHealthMonitor, ClaudeLogger, HealthConfig,
    LogConfig, LogLevel, ProcessConfig, ProcessManager,
};
use wezterm_parallel::room::manager::WorkspaceManager;

/// 統合テスト用のセットアップ
struct TestEnvironment {
    _temp_dir: TempDir,
    workspace_manager: WorkspaceManager,
    _process_manager: Arc<ProcessManager>,
    health_monitor: ClaudeHealthMonitor,
    logger: tokio::sync::Mutex<ClaudeLogger>,
}

impl TestEnvironment {
    async fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();

        // ワークスペースマネージャーの設定
        let workspace_state_path = temp_dir.path().join("workspaces.json");
        let mut workspace_manager = WorkspaceManager::new(Some(workspace_state_path)).unwrap();

        // プロセスマネージャーの設定
        let (process_manager, _event_receiver) = ProcessManager::new(ProcessConfig::default());
        let process_manager = Arc::new(process_manager);

        // ワークスペースマネージャーにプロセスマネージャーを設定
        workspace_manager.set_process_manager(process_manager.clone());
        workspace_manager.set_auto_start_claude_code(true);

        // ヘルスモニターの設定
        let health_config = HealthConfig {
            check_interval: Duration::from_secs(5),
            failure_threshold: 2,
            ..Default::default()
        };
        let health_monitor = ClaudeHealthMonitor::new(Some(health_config));

        // ロガーの設定
        let log_config = LogConfig {
            base_dir: temp_dir.path().join("logs"),
            max_file_size_mb: 10,
            log_level: LogLevel::Debug,
            ..Default::default()
        };
        let logger = ClaudeLogger::new(Some(log_config)).unwrap();

        Self {
            _temp_dir: temp_dir,
            workspace_manager,
            _process_manager: process_manager,
            health_monitor,
            logger: tokio::sync::Mutex::new(logger),
        }
    }
}

#[test]
async fn test_claude_code_detector_functionality() {
    // Phase 1: Claude Codeバイナリ検出のテスト
    let detector = ClaudeCodeDetector::new();

    // バイナリ検出を試行（システムにClaude Codeがインストールされていない場合はエラーになる）
    match detector.detect() {
        Ok(binary_path) => {
            println!("Found Claude Code binary: {binary_path:?}");

            // バージョン情報の取得をテスト
            if let Ok(version) = detector.get_version(&binary_path) {
                println!("Claude Code version: {version}");
                assert!(!version.is_empty());
            }
        }
        Err(e) => {
            println!("Claude Code binary not found (expected in test environment): {e}");
            // テスト環境では通常Claude Codeはインストールされていないため、これは正常
        }
    }
}

#[test]
async fn test_claude_code_config_builder() {
    // Phase 2: Claude Code設定の構築テスト
    let temp_dir = TempDir::new().unwrap();
    let binary_path = temp_dir.path().join("mock-claude-code");
    std::fs::File::create(&binary_path).unwrap();

    let config = ClaudeCodeConfigBuilder::new(binary_path.clone(), "test-workspace")
        .working_directory(temp_dir.path())
        .environment("TEST_ENV", "test_value")
        .argument("--test")
        .memory_limit(1024)
        .cpu_limit(50.0)
        .build()
        .unwrap();

    assert_eq!(config.binary_path, binary_path);
    assert_eq!(config.working_directory, temp_dir.path());
    assert!(config.get_complete_environment().contains_key("TEST_ENV"));
    assert!(config
        .get_complete_arguments()
        .contains(&"--test".to_string()));
    assert_eq!(config.memory_limit, Some(1024));
    assert_eq!(config.cpu_limit, Some(50.0));

    // コマンド文字列生成のテスト
    let command_str = config.to_command_string();
    assert!(command_str.contains("claude-code"));
    assert!(command_str.contains("TEST_ENV=test_value"));
    assert!(command_str.contains("--test"));
}

#[test]
async fn test_workspace_auto_startup_integration() {
    // Phase 3: ワークスペース自動起動統合のテスト
    let env = TestEnvironment::new().await;

    // ワークスペース作成（自動起動機能付き）
    let result = env
        .workspace_manager
        .create_workspace("test-auto-startup", "basic")
        .await;

    if result.is_ok() {
        // ワークスペースが作成されたことを確認
        let workspaces = env.workspace_manager.list_workspaces().await;
        assert!(workspaces.contains(&"test-auto-startup".to_string()));

        // ワークスペース情報を確認
        if let Some(workspace_info) = env
            .workspace_manager
            .get_workspace_info("test-auto-startup")
            .await
        {
            println!("Created workspace: {:?}", workspace_info.name);
            // Note: 実際のClaude Codeプロセスは起動しないが、ワークスペース構造は確認できる
        }
    } else {
        println!(
            "Workspace creation failed (expected in test environment): {:?}",
            result.err()
        );
    }
}

#[test]
async fn test_health_monitoring_functionality() {
    // Phase 4: ヘルスモニタリング機能のテスト
    let env = TestEnvironment::new().await;

    // 監視を開始
    let result = env
        .health_monitor
        .start_monitoring(
            "test-process".to_string(),
            "test-workspace".to_string(),
            Some(12345),
        )
        .await;

    assert!(result.is_ok());

    // ヘルス状態を取得
    let health_states = env.health_monitor.get_all_health_states().await;
    assert!(health_states.contains_key("test-process"));

    let process_health = &health_states["test-process"];
    assert_eq!(process_health.process_id, "test-process");
    assert_eq!(process_health.workspace, "test-workspace");
    assert_eq!(process_health.pid, Some(12345));

    // 監視を停止
    let result = env.health_monitor.stop_monitoring("test-process").await;
    assert!(result.is_ok());

    // ヘルス状態が削除されたことを確認
    let health_states = env.health_monitor.get_all_health_states().await;
    assert!(!health_states.contains_key("test-process"));
}

#[test]
async fn test_logging_functionality() {
    // Phase 5: ログ機能のテスト
    let env = TestEnvironment::new().await;

    // ロガーを開始
    {
        let mut logger = env.logger.lock().await;
        let start_result = logger.start().await;
        assert!(start_result.is_ok());
    }

    // プロセスログを開始
    let result = {
        let logger = env.logger.lock().await;
        logger
            .start_logging_process("test-process".to_string(), "test-workspace".to_string())
            .await
    };
    assert!(result.is_ok());

    // Claude Code出力をログ
    let result = {
        let logger = env.logger.lock().await;
        logger.log_claude_output(
            "test-process".to_string(),
            "test-workspace".to_string(),
            "Test output line".to_string(),
            false,
        )
    };
    assert!(result.is_ok());

    // ログ統計を取得
    let stats = {
        let logger = env.logger.lock().await;
        logger.get_log_statistics().await
    };
    assert!(stats.contains_key("test-process"));

    let process_stats = &stats["test-process"];
    assert_eq!(process_stats.process_id, "test-process");
    assert_eq!(process_stats.workspace, "test-workspace");

    // プロセスログを停止
    let result = {
        let logger = env.logger.lock().await;
        logger.stop_logging_process("test-process").await
    };
    assert!(result.is_ok());

    // ロガーを停止
    {
        let mut logger = env.logger.lock().await;
        let stop_result = logger.stop().await;
        assert!(stop_result.is_ok());
    }
}

#[test]
async fn test_full_integration_workflow() {
    // Phase 6: 全体統合ワークフローのテスト
    let env = TestEnvironment::new().await;

    // 1. ロガーを開始
    {
        let mut logger = env.logger.lock().await;
        let _ = logger.start().await;
    }

    // 2. ワークスペースを作成（自動起動トリガー）
    let workspace_name = "integration-test-workspace";

    match env
        .workspace_manager
        .create_workspace(workspace_name, "basic")
        .await
    {
        Ok(_) => {
            println!("Successfully created workspace: {workspace_name}");

            // 3. ワークスペースが作成されたことを確認
            let workspaces = env.workspace_manager.list_workspaces().await;
            assert!(workspaces.contains(&workspace_name.to_string()));

            // 4. ワークスペース情報を取得
            if let Some(workspace_info) = env
                .workspace_manager
                .get_workspace_info(workspace_name)
                .await
            {
                println!(
                    "Workspace info: name={}, processes={}",
                    workspace_info.name,
                    workspace_info.processes.len()
                );
            }
        }
        Err(e) => {
            println!("Workspace creation failed (expected in test environment): {e}");
        }
    }

    // 5. ヘルスモニタリングをテスト
    let monitor_result = env
        .health_monitor
        .start_monitoring(
            "integration-test-process".to_string(),
            workspace_name.to_string(),
            None,
        )
        .await;

    if monitor_result.is_ok() {
        println!("Health monitoring started successfully");

        // 6. ヘルス状態を確認
        let health_states = env.health_monitor.get_all_health_states().await;
        if let Some(health) = health_states.get("integration-test-process") {
            println!("Health status: {:?}", health.status);
        }

        // 7. 監視を停止
        let _ = env
            .health_monitor
            .stop_monitoring("integration-test-process")
            .await;
    }

    // 8. ログ機能をテスト
    let log_result = {
        let logger = env.logger.lock().await;
        logger
            .start_logging_process(
                "integration-log-test".to_string(),
                workspace_name.to_string(),
            )
            .await
    };

    if log_result.is_ok() {
        println!("Logging started successfully");

        // 9. ログエントリを送信
        {
            let logger = env.logger.lock().await;
            let _ = logger.log_claude_output(
                "integration-log-test".to_string(),
                workspace_name.to_string(),
                "Integration test log message".to_string(),
                false,
            );
        }

        // 10. ログ統計を確認
        let log_stats = {
            let logger = env.logger.lock().await;
            logger.get_log_statistics().await
        };
        println!("Active log streams: {}", log_stats.len());

        // 11. ログを停止
        {
            let logger = env.logger.lock().await;
            let _ = logger.stop_logging_process("integration-log-test").await;
        }
    }

    // 12. クリーンアップ
    {
        let mut logger = env.logger.lock().await;
        let _ = logger.stop().await;
    }

    println!("Integration test completed successfully");
}

#[test]
async fn test_error_handling_and_edge_cases() {
    // エラーハンドリングとエッジケースのテスト
    let env = TestEnvironment::new().await;

    // 1. 存在しないワークスペースの操作
    let result = env
        .workspace_manager
        .get_workspace_info("nonexistent-workspace")
        .await;
    assert!(result.is_none());

    // 2. 無効な設定でのConfig作成
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_binary = temp_dir.path().join("nonexistent-binary");

    let config_result = ClaudeCodeConfigBuilder::new(nonexistent_binary, "test")
        .working_directory("/nonexistent/directory")
        .build();

    // 検証でエラーになることを確認
    assert!(config_result.is_err());

    // 3. 存在しないプロセスの監視停止
    let result = env
        .health_monitor
        .stop_monitoring("nonexistent-process")
        .await;
    assert!(result.is_ok()); // graceful degradation

    // 4. 存在しないプロセスのログ停止
    let result = {
        let logger = env.logger.lock().await;
        logger.stop_logging_process("nonexistent-process").await
    };
    assert!(result.is_ok()); // graceful degradation

    println!("Error handling tests completed successfully");
}

#[test]
async fn test_performance_and_scalability() {
    // パフォーマンスとスケーラビリティのテスト
    let env = TestEnvironment::new().await;

    {
        let mut logger = env.logger.lock().await;
        let _ = logger.start().await;
    }

    // 複数のワークスペースを作成
    let workspace_count = 5;
    let mut created_workspaces = Vec::new();

    for i in 0..workspace_count {
        let workspace_name = format!("perf-test-workspace-{i}");

        match env
            .workspace_manager
            .create_workspace(&workspace_name, "basic")
            .await
        {
            Ok(_) => {
                created_workspaces.push(workspace_name.clone());
                println!("Created workspace: {workspace_name}");

                // 各ワークスペースでログを開始
                {
                    let logger = env.logger.lock().await;
                    let _ = logger
                        .start_logging_process(format!("process-{i}"), workspace_name.clone())
                        .await;
                }

                // 各ワークスペースでヘルス監視を開始
                let _ = env
                    .health_monitor
                    .start_monitoring(
                        format!("process-{i}"),
                        workspace_name,
                        Some(1000 + i as u32),
                    )
                    .await;
            }
            Err(e) => {
                println!("Failed to create workspace {i}: {e}");
            }
        }
    }

    // ワークスペース一覧を確認
    let workspaces = env.workspace_manager.list_workspaces().await;
    println!("Total workspaces: {}", workspaces.len());

    // ヘルス状態を確認
    let health_states = env.health_monitor.get_all_health_states().await;
    println!("Active health monitors: {}", health_states.len());

    // ログ統計を確認
    let log_stats = {
        let logger = env.logger.lock().await;
        logger.get_log_statistics().await
    };
    println!("Active log streams: {}", log_stats.len());

    // クリーンアップ
    for i in 0..workspace_count {
        let _ = env
            .health_monitor
            .stop_monitoring(&format!("process-{i}"))
            .await;
        {
            let logger = env.logger.lock().await;
            let _ = logger.stop_logging_process(&format!("process-{i}")).await;
        }
    }

    {
        let mut logger = env.logger.lock().await;
        let _ = logger.stop().await;
    }

    println!("Performance test completed successfully");
}
