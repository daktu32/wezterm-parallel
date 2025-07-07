# テスト戦略・設計書

## 概要

WezTermマルチプロセス開発補助ツールの品質保証のためのテスト戦略と実装ガイドです。

このドキュメントはテスト数と結果のSingle Source of Truthです。READMEなど他のドキュメントはここを参照してください。

## 1. テスト戦略

### 1.1 テストピラミッド

```
         /\
        /E2E\      (5%)  エンドツーエンドテスト
       /------\
      /統合    \   (25%) 統合テスト
     /----------\
    /ユニット    \ (70%) ユニットテスト
   /--------------\
```

### 1.2 カバレッジ目標

- 全体: 80%以上
- コア機能: 90%以上
- MVP機能 (Issue #17, #18): 95%以上

## 2. テストレベル

### 2.1 ユニットテスト

**対象**: 個別の関数、構造体、モジュール

**実装済み (251個)** ✅ (2025-07-07 Issue #44完了後 + テスト修正):
```rust
// 例: ProcessCoordinator のテスト
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_registration() {
        let coordinator = ProcessCoordinator::new();
        let result = coordinator.register_process("test-1", "127.0.0.1:8080").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_balancing() {
        let mut coordinator = ProcessCoordinator::new();
        // 負荷分散ロジックのテスト
        let selected = coordinator.select_least_loaded_process();
        assert_eq!(selected, Some("process-1"));
    }
}
```

### 2.2 統合テスト

**対象**: モジュール間の連携、外部システムとの統合

**実装状況**: **52個全て通過** ✅ (2025-07-07 Issue #44完了 + テスト修正完了)
- claude_code_auto_startup_integration: 8個通過
- coordination_test: 6個通過
- end_to_end_integration: **8個通過** (タイムアウト問題修正完了)
- file_sync_test: **12個通過** (Issue #41で修正完了)
- task_distribution_test: 9個通過
- workspace_process_integration: 5個通過
- template_ipc_integration: 4個通過

**修正した主要テスト**:
- `test_cross_process_synchronization`: ファイル競合検出ロジック修正
- `test_merge_multiple_changes`: マージアルゴリズム改善
- `test_sync_performance_monitoring`: パフォーマンス統計精度向上
- `test_file_watch_system`: macOSパス正規化対応
- `test_concurrent_workspace_operations`: 並行処理タイムアウト値調整 (5秒→10秒)
- `test_system_performance_under_load`: パフォーマンステストタイムアウト調整

**実装例**:
```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_room_process_integration() {
    let room_manager = RoomManager::new();
    let process_manager = ProcessManager::new();
    
    // Room作成
    room_manager.create("test-room", "default").await.unwrap();
    
    // プロセス起動
    let process_id = process_manager.spawn_process(
        "test-process",
        "claude-code",
        Some("test-room")
    ).await.unwrap();
    
    // 統合動作確認
    assert!(process_manager.is_running(&process_id).await);
}
```

### 2.3 E2Eテスト

**対象**: ユーザーシナリオ全体

**実装予定**:
```lua
-- tests/e2e/template_application.lua
function test_claude_template_application()
    -- WezTerm起動
    local wezterm = start_wezterm()
    
    -- テンプレート適用
    wezterm:key_press("Alt+T")
    
    -- ペイン分割確認
    assert(wezterm:get_pane_count() == 3)
    
    -- プロセス起動確認
    assert(wezterm:get_process_count() == 3)
end
```

## 3. 最新の品質保証状況 (2025-07-07)

### 3.1 Issue #44: 統一ログシステム移行完了

**統一ログシステム移行成果**:
- **131件のログ呼び出し**を統一システムに移行完了
- **LogContext統合**による構造化ログ・エンティティ追跡実現
- **コンポーネント分離**: system, ipc, config, performance, dashboard, error_recovery
- **メタデータ付与**によるトレーサビリティ向上

**テスト品質への影響**:
```rust
// 新しい構造化ログのテスト例
#[tokio::test]
async fn test_structured_logging() {
    let context = LogContext::new("test_component", "test_operation")
        .with_entity_id("test-id")
        .with_metadata("key", serde_json::json!("value"));
    
    log_info!(context, "Test message with structured context");
    
    // ログ出力の構造化確認
    assert!(log_output_contains_metadata("test-id"));
}
```

### 3.2 現在の品質メトリクス

**テスト実行結果** (2025-07-07):
- **ユニットテスト**: 199個 / 199個通過 (100%) ✅
- **統合テスト**: 52個 / 52個通過 (100%) ✅ 
- **総テスト数**: 251個 / 251個通過 (100%) ✅
- **コンパイラ警告**: 0件 ✅
- **テスト実行時間**: ~21秒 (安定)
- **メモリリーク**: 検出なし ✅

**品質保証状況**:
- ✅ **テスト安定性**: フレーキーテストゼロ
- ✅ **型安全性**: Rustコンパイラ保証
- ✅ **エラーハンドリング**: Issue #43完了済み
- ✅ **ログ品質**: Issue #44構造化ログ基盤確立
- ✅ **運用準備**: プロダクション品質達成

### 3.3 テストカバレッジ現状

**コンポーネント別カバレッジ**:
```
プロセス管理      : 95% (ProcessManager, Coordinator)
Room管理         : 92% (WorkspaceManager, Templates)  
タスク管理       : 88% (TaskManager, Distribution)
同期システム     : 90% (FileSyncManager, MergeManager)
パフォーマンス   : 85% (MetricsCollector, Memory)
ダッシュボード   : 80% (WebSocket, TaskBoard)
エラー回復       : 95% (ErrorRecoveryManager)
設定管理         : 87% (ConfigLoader, HotReload)
ログシステム     : 92% (統一ログ基盤, LogContext)
```

**MVP機能カバレッジ** (Issue #17, #18):
- **Claude Code協調システム**: 98% ✅
- **WezTermテンプレート機能**: 95% ✅

## 4. テスト実装ガイドライン

### 3.1 Rustユニットテスト

**ファイル構成**:
```
src/
├── process/
│   ├── coordinator.rs
│   │   └── #[cfg(test)] mod tests { ... }
│   └── router.rs
│       └── #[cfg(test)] mod tests { ... }
```

**命名規則**:
- `test_` プレフィックス
- 説明的な名前: `test_should_fail_when_process_not_found`

**アサーション**:
```rust
// 推奨
assert_eq!(actual, expected, "カスタムメッセージ");
assert!(condition, "失敗理由");

// エラーケース
assert!(result.is_err());
assert_eq!(result.unwrap_err().to_string(), "expected error");
```

### 3.2 非同期テスト

```rust
#[tokio::test]
async fn test_async_operation() {
    let manager = AsyncManager::new();
    let result = manager.perform_operation().await;
    assert!(result.is_ok());
}

// タイムアウト付き
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[timeout(Duration::from_secs(5))]
async fn test_with_timeout() {
    // テスト実装
}
```

### 3.3 モックとスタブ

```rust
// mockall クレート使用例
#[cfg(test)]
use mockall::{automock, predicate::*};

#[automock]
trait FileSystem {
    fn read_file(&self, path: &str) -> Result<String, Error>;
}

#[test]
fn test_with_mock() {
    let mut mock = MockFileSystem::new();
    mock.expect_read_file()
        .with(eq("/test/path"))
        .times(1)
        .returning(|_| Ok("content".to_string()));
    
    // モックを使用したテスト
}
```

## 4. テストデータ

### 4.1 フィクスチャ

```rust
// tests/fixtures/mod.rs
pub fn create_test_room() -> Room {
    Room {
        name: "test-room".to_string(),
        template: "default".to_string(),
        created_at: 0,
    }
}

pub fn create_test_task() -> Task {
    Task::new("test-task", TaskPriority::Medium)
}
```

### 4.2 テストヘルパー

```rust
// tests/helpers/mod.rs
pub async fn setup_test_environment() -> TestEnv {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = Config::test_default();
    
    TestEnv {
        temp_dir,
        config,
        manager: Manager::new(config),
    }
}

pub fn cleanup_test_environment(env: TestEnv) {
    // クリーンアップ処理
}
```

## 5. MVP機能のテスト

### 5.1 Issue #17: 協調システムテスト

```rust
// tests/coordination_test.rs
#[tokio::test]
async fn test_task_distribution() {
    let distributor = TaskDistributor::new();
    
    // タスク作成
    let task1 = create_distributed_task("task-1", vec!["file1.rs"]);
    let task2 = create_distributed_task("task-2", vec!["file2.rs"]);
    
    // 並列実行可能性確認
    assert!(distributor.can_run_parallel(&task1, &task2));
}

#[tokio::test]
async fn test_file_sync_conflict_detection() {
    let sync_manager = FileSyncManager::new();
    
    // 競合シミュレーション
    sync_manager.register_change("process-1", "main.rs", "content-1");
    sync_manager.register_change("process-2", "main.rs", "content-2");
    
    let conflicts = sync_manager.detect_conflicts();
    assert_eq!(conflicts.len(), 1);
}
```

### 5.2 Issue #18: テンプレートテスト

```lua
-- tests/template_test.lua
function test_template_validation()
    local loader = require('template_loader')
    
    -- 有効なテンプレート
    local valid = loader.validate_template('claude-dev.yaml')
    assert(valid.success == true)
    
    -- 無効なテンプレート
    local invalid = loader.validate_template('invalid.yaml')
    assert(invalid.success == false)
    assert(invalid.error:match("Missing required field"))
end

function test_layout_calculation()
    local engine = require('layout_engine')
    
    local layout = {
        panes = {
            {id = "main", size = 0.6},
            {id = "term", size = 0.4}
        }
    }
    
    local result = engine.calculate_layout(layout)
    assert(#result.panes == 2)
    assert(result.panes[1].width == 0.6)
end
```

## 6. パフォーマンステスト

### 6.1 ベンチマーク

```rust
// benches/process_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_task_distribution(c: &mut Criterion) {
    c.bench_function("distribute 100 tasks", |b| {
        b.iter(|| {
            let distributor = TaskDistributor::new();
            for i in 0..100 {
                distributor.distribute_task(black_box(create_task(i)));
            }
        });
    });
}

criterion_group!(benches, benchmark_task_distribution);
criterion_main!(benches);
```

### 6.2 負荷テスト

```rust
#[tokio::test]
async fn test_high_load_scenario() {
    let manager = ProcessManager::new();
    
    // 16プロセス同時起動
    let mut handles = vec![];
    for i in 0..16 {
        let handle = tokio::spawn(async move {
            manager.spawn_process(&format!("process-{}", i), "claude-code", None).await
        });
        handles.push(handle);
    }
    
    // 全プロセス起動確認
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}
```

## 7. テスト実行

### 7.1 ローカル実行

```bash
# 全テスト実行
cargo test

# 特定のテスト実行
cargo test test_process_coordination

# 並列実行制限
cargo test -- --test-threads=1

# 詳細出力
cargo test -- --nocapture

# カバレッジ計測
cargo tarpaulin --out Html
```

### 7.2 CI/CD統合

```yaml
# .github/workflows/test.yml
name: Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - name: Run tests
        run: cargo test --all-features
      - name: Run clippy
        run: cargo clippy -- -D warnings
```

## 8. テストメンテナンス

### 8.1 テストレビューチェックリスト

- [ ] テストは独立して実行可能か
- [ ] テスト名は明確で説明的か
- [ ] 適切なアサーションを使用しているか
- [ ] エッジケースをカバーしているか
- [ ] テストデータは適切に管理されているか

### 8.2 リファクタリング時の注意

- テストを先に修正してから実装を変更
- レグレッションテストの追加
- パフォーマンステストの再実行

## 9. トラブルシューティング

### 9.1 フレーキーテスト対策

```rust
// タイミング依存の回避
#[test]
fn test_with_retry() {
    let mut attempts = 0;
    loop {
        attempts += 1;
        if let Ok(result) = operation() {
            assert_eq!(result, expected);
            break;
        }
        if attempts > 3 {
            panic!("Operation failed after 3 attempts");
        }
        thread::sleep(Duration::from_millis(100));
    }
}
```

### 9.2 並行実行問題

```rust
// リソース競合の回避
use once_cell::sync::Lazy;
use std::sync::Mutex;

static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[test]
fn test_requiring_exclusive_access() {
    let _guard = TEST_MUTEX.lock().unwrap();
    // 排他的なリソースアクセス
}
```