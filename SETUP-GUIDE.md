# 📖 詳細セットアップガイド

このガイドでは、WezTerm Parallelの本格的な利用に向けた設定とカスタマイズについて説明します。

**前提**: [クイックスタートガイド](QUICKSTART.md)を完了していること

## 🔧 システム要件

### 必須要件
- **OS**: Linux (Ubuntu 20.04+), macOS (11.0+), Windows (WSL2推奨)
- **WezTerm**: 20240203-110809-5046fc22以降
- **Rust**: 1.70.0以降
- **メモリ**: 最低512MB、推奨1GB以上
- **ディスク**: 100MB以上の空き容量

### オプション要件
- **Claude Code**: 自動起動・統合機能のため
- **Git**: ワークスペース管理・バージョン管理のため

## 📁 ディレクトリ構造

インストール後の構成：

```
~/.config/wezterm-parallel/
├── config.yaml          # メイン設定ファイル
├── workspaces.json      # ワークスペース状態
├── templates/           # カスタムテンプレート
│   ├── layouts/         # レイアウトテンプレート
│   └── processes/       # プロセステンプレート
└── logs/                # ログファイル
    ├── application.log
    └── processes/       # プロセス別ログ

~/.local/share/wezterm-parallel/
├── backups/             # ファイルバックアップ
└── cache/               # 一時ファイル・キャッシュ
```

## ⚙️ 設定ファイル詳細

### 1. メイン設定 (config.yaml)

デフォルト設定ファイルを生成：

```bash
# 設定ディレクトリ作成
mkdir -p ~/.config/wezterm-parallel

# デフォルト設定を生成
wezterm-parallel --generate-config > ~/.config/wezterm-parallel/config.yaml
```

設定ファイル例：

```yaml
# ~/.config/wezterm-parallel/config.yaml

# サーバー設定
server:
  host: "127.0.0.1"
  port: 8080
  websocket_port: 8081
  max_connections: 50
  
# プロセス管理
process_management:
  max_processes_per_workspace: 8
  auto_restart: true
  health_check_interval: 30
  process_timeout: 300
  
# Claude Code統合
claude_code:
  binary_path: "claude-code"  # PATHから検索
  auto_detect: true
  default_args: ["--interactive"]
  resource_limits:
    memory_mb: 512
    cpu_percent: 50
  
# ワークスペース設定
workspace:
  default_template: "basic"
  auto_save_interval: 60
  backup_enabled: true
  max_backup_files: 10
  
# ログ設定
logging:
  level: "info"           # trace, debug, info, warn, error
  file_enabled: true
  console_enabled: true
  max_file_size_mb: 10
  max_files: 5
  
# WezTerm統合
wezterm:
  config_path: "~/.config/wezterm/wezterm.lua"
  auto_reload: true
  keybindings_enabled: true
  
# メトリクス・監視
monitoring:
  metrics_enabled: true
  metrics_interval: 10
  alerts_enabled: true
  performance_tracking: true
  
# セキュリティ
security:
  api_key_required: false  # 本番環境では true を推奨
  cors_enabled: true
  allowed_origins: ["http://localhost:*"]
```

### 2. WezTerm設定統合

#### 基本統合

```lua
-- ~/.config/wezterm/wezterm.lua

local wezterm = require 'wezterm'
local config = {}

-- WezTerm Parallel統合を読み込み
local wtp_config = require('wezterm-parallel-integration')

-- 基本設定
config.font = wezterm.font('JetBrains Mono')
config.font_size = 12.0
config.color_scheme = 'Tomorrow Night'

-- WezTerm Parallel統合を適用
wtp_config.apply_integration(config)

return config
```

#### カスタムキーバインド

```lua
-- WezTerm Parallelキーバインドのカスタマイズ
local wtp_keys = {
  -- ワークスペース管理
  { key = 'n', mods = 'CTRL|SHIFT', action = wtp_config.actions.new_workspace },
  { key = 'w', mods = 'CTRL|SHIFT', action = wtp_config.actions.switch_workspace },
  { key = 'x', mods = 'CTRL|SHIFT', action = wtp_config.actions.close_workspace },
  
  -- テンプレート適用
  { key = 't', mods = 'ALT', action = wtp_config.actions.select_template },
  { key = 'T', mods = 'ALT', action = wtp_config.actions.apply_claude_template },
  
  -- プロセス管理
  { key = 'p', mods = 'CTRL|ALT', action = wtp_config.actions.manage_processes },
  { key = 'r', mods = 'CTRL|ALT', action = wtp_config.actions.restart_process },
  
  -- ダッシュボード
  { key = 'd', mods = 'CTRL|SHIFT', action = wtp_config.actions.open_dashboard },
}

-- 既存のキーバインドに追加
for _, key in ipairs(wtp_keys) do
  table.insert(config.keys, key)
end
```

## 🎨 テンプレートシステム

### 1. レイアウトテンプレート作成

カスタムテンプレートの作成例：

```yaml
# ~/.config/wezterm-parallel/templates/layouts/my-dev.yaml

name: "My Development Setup"
description: "個人用開発環境レイアウト"
version: "1.0"

layout:
  type: "grid"
  panes:
    - id: "editor"
      position: { row: 0, col: 0, width: 0.6, height: 0.8 }
      title: "Editor"
      command: "nvim"
      
    - id: "terminal"
      position: { row: 0, col: 1, width: 0.4, height: 0.4 }
      title: "Terminal"
      command: "bash"
      
    - id: "logs"
      position: { row: 1, col: 1, width: 0.4, height: 0.4 }
      title: "Logs"
      command: "tail -f logs/application.log"
      
    - id: "status"
      position: { row: 1, col: 0, width: 0.6, height: 0.2 }
      title: "Status"
      command: "htop"

processes:
  - name: "development-server"
    command: "npm run dev"
    restart_policy: "always"
    environment:
      NODE_ENV: "development"
      PORT: "3000"
      
  - name: "file-watcher"
    command: "npm run watch"
    restart_policy: "on-failure"

keybindings:
  - key: "F5"
    action: "restart_all_processes"
  - key: "F12"
    action: "toggle_logs_pane"
```

### 2. プロセステンプレート

```yaml
# ~/.config/wezterm-parallel/templates/processes/web-stack.yaml

name: "Web Development Stack"
description: "フルスタック開発用プロセス構成"

processes:
  frontend:
    command: "npm run dev"
    working_dir: "./frontend"
    environment:
      NODE_ENV: "development"
      VITE_API_URL: "http://localhost:8000"
    health_check:
      url: "http://localhost:3000"
      interval: 30
      
  backend:
    command: "cargo run"
    working_dir: "./backend"
    environment:
      RUST_LOG: "debug"
      DATABASE_URL: "sqlite://dev.db"
    depends_on: ["database"]
    
  database:
    command: "docker run -p 5432:5432 postgres:15"
    health_check:
      command: "pg_isready -h localhost -p 5432"
      interval: 10
      
  claude_assistant:
    command: "claude-code --workspace web-project"
    auto_start: true
    resource_limits:
      memory_mb: 1024
      cpu_percent: 30
```

## 🔌 高度な機能設定

### 1. API設定とセキュリティ

```yaml
# 本番環境向け設定例
security:
  api_key_required: true
  api_key: "${WEZTERM_PARALLEL_API_KEY}"  # 環境変数から取得
  cors_enabled: true
  allowed_origins: 
    - "https://mydomain.com"
    - "http://localhost:3000"
  rate_limiting:
    enabled: true
    requests_per_minute: 100
    
ssl:
  enabled: false  # 必要に応じてTLS設定
  cert_file: "/path/to/cert.pem"
  key_file: "/path/to/key.pem"
```

### 2. メトリクス・アラート設定

```yaml
monitoring:
  metrics_enabled: true
  exporters:
    prometheus:
      enabled: true
      port: 9090
      path: "/metrics"
    json:
      enabled: true
      file: "~/.local/share/wezterm-parallel/metrics.json"
      
alerts:
  enabled: true
  rules:
    - name: "high_cpu_usage"
      condition: "cpu_usage > 80"
      duration: "5m"
      action: "log_warning"
      
    - name: "process_failure"
      condition: "process_status == 'failed'"
      action: "restart_process"
      max_restarts: 3
      
    - name: "memory_leak_detection"
      condition: "memory_usage_mb > 1024"
      duration: "10m"
      action: "alert_notification"
```

### 3. バックアップ・復旧設定

```yaml
backup:
  enabled: true
  interval: "1h"
  retention:
    hourly: 24
    daily: 7
    weekly: 4
    monthly: 3
  
  targets:
    - type: "workspace_state"
      path: "~/.config/wezterm-parallel/workspaces.json"
    - type: "user_files"
      patterns: ["**/*.md", "**/*.yaml", "**/*.json"]
      exclude: ["**/node_modules/**", "**/target/**"]
  
  storage:
    type: "local"  # 将来: s3, gcs, etc.
    path: "~/.local/share/wezterm-parallel/backups"
    compression: true
    encryption: false  # 将来実装予定
```

## 🧪 開発・デバッグ設定

### 1. 開発モード

```bash
# 開発モードで起動（詳細ログ・自動リロード）
RUST_LOG=debug wezterm-parallel --dev-mode

# 設定ファイルの変更を監視
wezterm-parallel --watch-config

# 特定のモジュールのみデバッグ
RUST_LOG=wezterm_parallel::process=trace wezterm-parallel
```

### 2. デバッグ用設定

```yaml
# デバッグ用設定 (config-debug.yaml)
logging:
  level: "trace"
  console_enabled: true
  file_enabled: true
  pretty_logs: true
  
development:
  hot_reload: true
  auto_restart_on_crash: true
  detailed_errors: true
  performance_profiling: true
  
testing:
  mock_claude_code: true  # Claude Codeがない環境での testing
  fake_delay_ms: 100      # レスポンス遅延のシミュレーション
  error_injection: false  # エラー処理のテスト
```

## 🔄 アップデート・メンテナンス

### 1. 自動アップデート

```bash
# バージョン確認
wezterm-parallel --version

# 最新版への更新
cargo install --git https://github.com/daktu32/wezterm-parallel --force

# 設定ファイル互換性チェック
wezterm-parallel --check-config
```

### 2. 定期メンテナンス

```bash
# ログローテーション
wezterm-parallel --rotate-logs

# キャッシュクリア
wezterm-parallel --clear-cache

# バックアップファイル整理
wezterm-parallel --cleanup-backups
```

## 📊 パフォーマンス最適化

### 1. システムチューニング

```yaml
performance:
  # プロセス並行度
  max_concurrent_processes: 4  # CPUコア数に応じて調整
  
  # メモリ使用量制限
  memory_limits:
    per_workspace_mb: 2048
    total_system_mb: 4096
    
  # ディスクI/O最適化
  disk_io:
    async_writes: true
    buffer_size_kb: 64
    fsync_interval: 5
    
  # ネットワーク最適化
  network:
    keep_alive: true
    tcp_nodelay: true
    connection_pool_size: 20
```

### 2. リソース監視

```bash
# リアルタイムリソース監視
wezterm-parallel --monitor

# パフォーマンスレポート生成
wezterm-parallel --performance-report
```

## 🆘 トラブルシューティング

### よくある問題と解決策

#### 1. 起動時の問題

**問題**: サービスが起動しない
```bash
# 詳細ログで原因を確認
RUST_LOG=debug wezterm-parallel

# ポート使用状況確認
sudo lsof -i :8080 -i :8081

# 設定ファイル検証
wezterm-parallel --validate-config
```

#### 2. WezTerm統合の問題

**問題**: キーバインドが効かない
```bash
# WezTerm設定の構文チェック
wezterm show-config

# WezTerm統合モジュールのテスト
wezterm-parallel --test-wezterm-integration
```

#### 3. パフォーマンスの問題

**問題**: 動作が重い
```bash
# リソース使用量確認
wezterm-parallel --resource-usage

# プロファイリング実行
wezterm-parallel --profile=cpu
```

### ログファイルの確認

```bash
# アプリケーションログ
tail -f ~/.config/wezterm-parallel/logs/application.log

# 特定プロセスのログ
tail -f ~/.config/wezterm-parallel/logs/processes/workspace-name/process-name.log

# エラーログのみ表示
grep ERROR ~/.config/wezterm-parallel/logs/application.log
```

## 📚 次のステップ

1. **[ユーザーガイド](docs/USER-GUIDE.md)**: 実用的な使い方とワークフロー
2. **[API Documentation](https://daktu32.github.io/wezterm-parallel/)**: プログラム的な操作
3. **[カスタマイズガイド](docs/CUSTOMIZATION.md)**: 高度なカスタマイズ
4. **[FAQ](docs/FAQ.md)**: よくある質問と回答

---

🔧 これで本格的なWezTerm Parallelの利用準備が整いました！

不明な点があれば [GitHubのIssues](https://github.com/daktu32/wezterm-parallel/issues) でお気軽にお聞きください。