# WezTerm マルチプロセス開発補助ツール - アーキテクチャ仕様書
---
**Last Updated**: 2025-07-03
**Version**: 0.1.0
**Next Review**: 2025-10-01
---


## 1. システム全体アーキテクチャ

### 1.1 アーキテクチャ概要

```
┌─────────────────────────────────────────────────────────────┐
│                     WezTerm Terminal                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │  Workspace A    │  │  Workspace B    │  │ Control Pane │ │
│  │ ┌─────┬─────┬───┐│  │ ┌─────┬─────┐   │  │ ┌──────────┐ │ │
│  │ │Pane1│Pane2│...││  │ │Pane1│Pane2│   │  │ │Dashboard │ │ │
│  │ └─────┴─────┴───┘│  │ └─────┴─────┘   │  │ └──────────┘ │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Lua Configuration Layer                  │
└─────────────────────────────────────────────────────────────┘
            │                    │                    │
            ▼                    ▼                    ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Workspace      │    │  Process        │    │  Communication │
│  Manager        │    │  Manager        │    │  Hub            │
└─────────────────┘    └─────────────────┘    └─────────────────┘
            │                    │                    │
            └────────────────────┼────────────────────┘
                                 ▼
                    ┌─────────────────────────┐
                    │   Backend Services      │
                    │ ┌─────────────────────┐ │
                    │ │ Claude Code         │ │
                    │ │ Process Pool        │ │
                    │ └─────────────────────┘ │
                    │ ┌─────────────────────┐ │
                    │ │ State Management    │ │
                    │ └─────────────────────┘ │
                    │ ┌─────────────────────┐ │
                    │ │ IPC Server          │ │
                    │ └─────────────────────┘ │
                    └─────────────────────────┘
```

### 1.2 レイヤー構成

#### フロントエンドレイヤー (WezTerm + Lua)
- **WezTerm Terminal**: ユーザーインターフェース (🔄 Phase 2で統合予定)
- **Lua Configuration**: 設定管理とイベントハンドリング (✅ 3,239行準備済み)
- **Workspace Management**: ワークスペースとペインの管理 (✅ 実装済み)

#### バックエンドレイヤー (Rust)
- **Process Manager**: Claude Codeプロセスの管理 (✅ 実装済み)
- **Communication Hub**: プロセス間通信の仲介 (✅ 実装済み)
- **State Management**: アプリケーション状態の永続化 (✅ 実装済み)

## 2. コンポーネント詳細設計

### 2.1 Workspace Manager

```lua
-- workspace_manager.lua
local WorkspaceManager = {}

WorkspaceManager.config = {
  max_workspaces = 8,
  default_layout = "three_pane",
  auto_save_interval = 30, -- seconds
}

function WorkspaceManager:create_workspace(name, template)
  -- ワークスペース作成ロジック
end

function WorkspaceManager:switch_workspace(name)
  -- ワークスペース切り替えロジック
end

function WorkspaceManager:save_state()
  -- 状態永続化ロジック
end

return WorkspaceManager
```

**責務**: (✅ 実装完了)
- ワークスペースのライフサイクル管理
- テンプレートベースのワークスペース作成
- ワークスペース間の切り替え制御
- 状態の永続化と復元

**インターフェース**:
- `create_workspace(name, template)`: 新規ワークスペース作成
- `switch_workspace(name)`: ワークスペース切り替え
- `delete_workspace(name)`: ワークスペース削除
- `list_workspaces()`: ワークスペース一覧取得

### 2.2 Process Manager

```rust
// process_manager.rs
use std::collections::HashMap;
use tokio::process::{Child, Command};

pub struct ProcessManager {
    processes: HashMap<String, ClaudeCodeProcess>,
    config: ProcessConfig,
}

pub struct ClaudeCodeProcess {
    id: String,
    child: Child,
    workspace: String,
    status: ProcessStatus,
    last_heartbeat: SystemTime,
}

impl ProcessManager {
    pub async fn spawn_process(&mut self, workspace: &str) -> Result<String> {
        // Claude Codeプロセス起動
    }
    
    pub async fn monitor_processes(&mut self) {
        // プロセス監視とヘルスチェック
    }
    
    pub async fn restart_process(&mut self, process_id: &str) -> Result<()> {
        // プロセス再起動
    }
}
```

**責務**: (✅ 実装完了)
- Claude Codeプロセスの起動・停止・再起動
- プロセスのヘルスモニタリング
- リソース使用量の監視
- 異常終了時の自動復旧

**インターフェース**:
- `spawn_process(workspace)`: プロセス起動
- `kill_process(process_id)`: プロセス終了
- `get_process_status(process_id)`: プロセス状態取得
- `list_processes()`: プロセス一覧取得

### 2.3 Communication Hub

```rust
// communication_hub.rs
use tokio::net::{UnixListener, UnixStream};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Message {
    TaskRequest { id: String, workspace: String, command: String },
    TaskResponse { id: String, result: TaskResult },
    StatusUpdate { process_id: String, status: ProcessStatus },
    BroadcastMessage { content: String, sender: String },
}

pub struct CommunicationHub {
    listener: UnixListener,
    connections: HashMap<String, UnixStream>,
    message_queue: VecDeque<Message>,
}

impl CommunicationHub {
    pub async fn start_server(&mut self) -> Result<()> {
        // IPCサーバー起動
    }
    
    pub async fn route_message(&mut self, message: Message) -> Result<()> {
        // メッセージルーティング
    }
    
    pub async fn broadcast(&mut self, message: Message) -> Result<()> {
        // ブロードキャスト送信
    }
}
```

**責務**: (✅ 実装完了)
- プロセス間通信の仲介
- メッセージルーティング
- ブロードキャスト通信
- 通信セッション管理

**通信プロトコル**:
- **Unix Domain Socket**: ローカルプロセス間通信
- **JSON Protocol**: メッセージフォーマット
- **WebSocket (オプション)**: リモート接続対応

### 2.4 State Management

```rust
// state_management.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct GlobalState {
    workspaces: HashMap<String, WorkspaceState>,
    processes: HashMap<String, ProcessState>,
    user_preferences: UserPreferences,
    last_updated: SystemTime,
}

#[derive(Serialize, Deserialize)]
pub struct WorkspaceState {
    name: String,
    layout: LayoutConfig,
    panes: Vec<PaneState>,
    active_tasks: Vec<TaskState>,
}

pub struct StateManager {
    state: GlobalState,
    state_file: PathBuf,
    auto_save_enabled: bool,
}

impl StateManager {
    pub fn save_state(&self) -> Result<()> {
        // 状態をファイルに永続化
    }
    
    pub fn load_state(&mut self) -> Result<()> {
        // ファイルから状態を復元
    }
    
    pub fn update_workspace_state(&mut self, workspace: &str, state: WorkspaceState) {
        // ワークスペース状態更新
    }
}
```

**責務**: (✅ 実装完了)
- アプリケーション状態の永続化
- 状態の復元とマイグレーション
- 設定管理
- バックアップとロールバック

**ストレージ**:
- **Primary**: JSON/YAML ファイル
- **Backup**: SQLite データベース（オプション）
- **Location**: `~/.config/wezterm-parallel/` (✅ 実装済み)

## 3. データフロー設計

### 3.1 ワークスペース作成フロー

```
User Request → Lua Handler → Workspace Manager → Process Manager
     │              │              │                    │
     │              │              │                    ▼
     │              │              │           Spawn Claude Code
     │              │              │                    │
     │              │              ▼                    │
     │              │         Create Panes              │
     │              │              │                    │
     │              ▼              │                    │
     │        Configure Layout     │                    │
     │              │              │                    │
     ▼              ▼              ▼                    ▼
State Update ← Save State ← Register Workspace ← IPC Setup
```

### 3.2 タスク実行フロー

```
Task Request → Communication Hub → Target Process
     │                │                 │
     │                │                 ▼
     │                │           Execute Task
     │                │                 │
     │                ▼                 │
     │          Queue Management        │
     │                │                 │
     │                │                 ▼
     │                │           Send Response
     │                │                 │
     ▼                ▼                 ▼
Status Update ← Route Response ← Task Complete
```

### 3.3 プロセス監視フロー

```
Process Manager → Health Check → Process Status
     │                │               │
     │                │               ▼
     │                │         Status Update
     │                │               │
     │                ▼               │
     │          Failure Detection     │
     │                │               │
     ▼                ▼               ▼
Auto Restart ← Alert System ← Communication Hub
```

## 4. 設定管理設計

### 4.1 設定ファイル構造

```yaml
# ~/.config/wezterm-parallel/config.yaml
framework:
  version: "1.0.0"
  log_level: "info"
  
workspaces:
  templates:
    default:
      layout: "three_pane_horizontal"
      auto_start_processes: true
      panes:
        - { position: "left", command: "claude-code --workspace=main" }
        - { position: "center", command: "claude-code --workspace=test" }
        - { position: "right", command: "htop" }
    
    web_dev:
      layout: "four_pane_grid"
      auto_start_processes: true
      panes:
        - { position: "top-left", command: "claude-code --workspace=frontend" }
        - { position: "top-right", command: "claude-code --workspace=backend" }
        - { position: "bottom-left", command: "npm run dev" }
        - { position: "bottom-right", command: "tail -f logs/app.log" }

processes:
  claude_code:
    max_instances: 16
    restart_policy: "always"
    health_check_interval: 10
    timeout: 30
    environment:
      CLAUDE_API_KEY: "${CLAUDE_API_KEY}"
      
communication:
  protocol: "unix_socket"
  socket_path: "/tmp/wezterm-parallel.sock"
  message_timeout: 5000
  
ui:
  theme: "dark"
  show_process_status: true
  auto_hide_inactive_panes: false
  keybindings:
    create_workspace: "ctrl+shift+n"
    switch_workspace: "ctrl+shift+w"
    kill_process: "ctrl+shift+k"
```

### 4.2 Lua設定統合

```lua
-- ~/.config/wezterm/wezterm.lua
local wezterm = require 'wezterm'
local multi_dev = require 'multi_dev_framework'

local config = wezterm.config_builder()

-- Multi-dev framework integration
multi_dev.setup(config, {
  config_path = wezterm.home_dir .. '/.config/wezterm-parallel/config.yaml',
  auto_start = true,
  debug = false
})

-- Custom keybindings
config.keys = {
  -- Framework specific keys
  { key = 'n', mods = 'CTRL|SHIFT', action = multi_dev.actions.create_workspace },
  { key = 'w', mods = 'CTRL|SHIFT', action = multi_dev.actions.switch_workspace },
  { key = 'd', mods = 'CTRL|SHIFT', action = multi_dev.actions.show_dashboard },
  
  -- Standard WezTerm keys
  { key = 'c', mods = 'CTRL|SHIFT', action = wezterm.action.CopyTo 'Clipboard' },
  { key = 'v', mods = 'CTRL|SHIFT', action = wezterm.action.PasteFrom 'Clipboard' },
}

return config
```

## 5. セキュリティ設計

### 5.1 プロセス分離

```rust
// security.rs
use std::os::unix::process::CommandExt;

impl ProcessManager {
    fn create_secure_process(&self, workspace: &str) -> Command {
        let mut cmd = Command::new("claude-code");
        
        // プロセス分離設定
        cmd.uid(get_user_id());
        cmd.gid(get_group_id());
        
        // 環境変数の制限
        cmd.env_clear();
        cmd.env("PATH", "/usr/local/bin:/usr/bin:/bin");
        cmd.env("WORKSPACE", workspace);
        
        // ファイルアクセス制限（chroot風）
        let workspace_dir = format!("/workspaces/{}", workspace);
        cmd.current_dir(&workspace_dir);
        
        cmd
    }
}
```

### 5.2 通信セキュリティ

```rust
// secure_communication.rs
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};

pub struct SecureCommunicationHub {
    encryption_key: LessSafeKey,
    hub: CommunicationHub,
}

impl SecureCommunicationHub {
    pub fn encrypt_message(&self, message: &[u8]) -> Result<Vec<u8>> {
        // メッセージ暗号化
    }
    
    pub fn decrypt_message(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        // メッセージ復号化
    }
}
```

## 6. パフォーマンス最適化

### 6.1 メモリ管理

```rust
// memory_optimization.rs
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct OptimizedProcessManager {
    // Arc<RwLock<T>>でメモリ効率化
    processes: Arc<RwLock<HashMap<String, Arc<ClaudeCodeProcess>>>>,
    
    // プロセスプールでインスタンス再利用
    process_pool: ProcessPool,
    
    // メモリ使用量監視
    memory_monitor: MemoryMonitor,
}

impl OptimizedProcessManager {
    async fn cleanup_inactive_processes(&mut self) {
        // 非アクティブプロセスのクリーンアップ
    }
    
    async fn reuse_process(&mut self, workspace: &str) -> Option<Arc<ClaudeCodeProcess>> {
        // プロセス再利用ロジック
    }
}
```

### 6.2 非同期処理最適化

```rust
// async_optimization.rs
use tokio::task::JoinSet;
use std::time::Duration;

pub struct AsyncTaskManager {
    task_set: JoinSet<TaskResult>,
    semaphore: tokio::sync::Semaphore,
}

impl AsyncTaskManager {
    pub async fn execute_parallel_tasks(&mut self, tasks: Vec<Task>) -> Vec<TaskResult> {
        let mut results = Vec::new();
        
        for task in tasks {
            let permit = self.semaphore.acquire().await.unwrap();
            let handle = tokio::spawn(async move {
                let _permit = permit; // 自動解放
                task.execute().await
            });
            self.task_set.spawn(handle);
        }
        
        while let Some(result) = self.task_set.join_next().await {
            results.push(result.unwrap());
        }
        
        results
    }
}
```

## 7. モニタリング・ログ設計

### 7.1 構造化ログ

```rust
// logging.rs
use tracing::{info, warn, error, instrument};
use serde_json::json;

pub struct FrameworkLogger {
    subscriber: tracing_subscriber::Registry,
}

impl FrameworkLogger {
    #[instrument(level = "info")]
    pub fn log_workspace_created(&self, workspace: &str, template: &str) {
        info!(
            workspace = workspace,
            template = template,
            event = "workspace_created",
            "Workspace created successfully"
        );
    }
    
    #[instrument(level = "warn")]
    pub fn log_process_restart(&self, process_id: &str, reason: &str) {
        warn!(
            process_id = process_id,
            reason = reason,
            event = "process_restart",
            "Process restarted due to failure"
        );
    }
}
```

### 7.2 メトリクス収集

```rust
// metrics.rs
use prometheus::{Counter, Histogram, Gauge, Registry};

pub struct FrameworkMetrics {
    workspace_count: Gauge,
    process_count: Gauge,
    task_duration: Histogram,
    error_count: Counter,
    registry: Registry,
}

impl FrameworkMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();
        
        Self {
            workspace_count: Gauge::new("workspaces_active", "Active workspace count").unwrap(),
            process_count: Gauge::new("processes_active", "Active process count").unwrap(),
            task_duration: Histogram::new("task_duration_seconds", "Task execution duration").unwrap(),
            error_count: Counter::new("errors_total", "Total error count").unwrap(),
            registry,
        }
    }
    
    pub fn record_task_completion(&self, duration: Duration) {
        self.task_duration.observe(duration.as_secs_f64());
    }
}
```

## 8. 拡張性・保守性

### 8.1 プラグインアーキテクチャ

```rust
// plugin_system.rs
use dlopen::wrapper::{Container, WrapperApi};

#[derive(WrapperApi)]
struct PluginApi {
    init: fn() -> i32,
    execute: fn(*const c_char) -> *const c_char,
    cleanup: fn(),
}

pub struct PluginManager {
    plugins: HashMap<String, Container<PluginApi>>,
}

impl PluginManager {
    pub fn load_plugin(&mut self, name: &str, path: &str) -> Result<()> {
        let container: Container<PluginApi> = unsafe { Container::load(path)? };
        container.init();
        self.plugins.insert(name.to_string(), container);
        Ok(())
    }
    
    pub fn execute_plugin(&self, name: &str, input: &str) -> Result<String> {
        if let Some(plugin) = self.plugins.get(name) {
            let c_input = CString::new(input)?;
            let result = plugin.execute(c_input.as_ptr());
            // 結果処理
        }
        Err("Plugin not found".into())
    }
}
```

### 8.2 イベントシステム

```rust
// event_system.rs
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub enum FrameworkEvent {
    WorkspaceCreated { name: String },
    ProcessStarted { id: String, workspace: String },
    TaskCompleted { id: String, result: TaskResult },
    Error { message: String, context: String },
}

pub struct EventSystem {
    sender: broadcast::Sender<FrameworkEvent>,
    receivers: Vec<broadcast::Receiver<FrameworkEvent>>,
}

impl EventSystem {
    pub fn publish(&self, event: FrameworkEvent) -> Result<()> {
        self.sender.send(event)?;
        Ok(())
    }
    
    pub fn subscribe(&mut self) -> broadcast::Receiver<FrameworkEvent> {
        self.sender.subscribe()
    }
}
```

この設計により、WezTerm単体でも堅牢で拡張可能なマルチプロセス並行開発環境を構築できます。

## 関連ドキュメント
- [プロジェクト概要](../README.md)
- [ドキュメント体系](DOCUMENTATION-MAP.md)
- [アーキテクチャ](ARCHITECTURE.md)
- [貢献ガイド](CONTRIBUTING.md)
