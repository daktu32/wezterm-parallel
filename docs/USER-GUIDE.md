# 📚 WezTerm Parallel ユーザーガイド

**実用的な使い方とベストプラクティス集**

このガイドでは、WezTerm Parallelを日常的に活用するための具体的な使い方とプロのコツを紹介します。

**前提**: [クイックスタートガイド](../QUICKSTART.md)を完了していること

## 🎯 基本的なワークフロー

### 1. 日常的な開発フロー

```bash
# 1. フレームワーク起動（朝一番）
wezterm-parallel &

# 2. プロジェクト用ワークスペース作成
curl -X POST http://localhost:8080/api/workspaces \
  -H "Content-Type: application/json" \
  -d '{"name": "my-project", "template": "claude-dev"}'

# 3. ダッシュボードで監視
# http://localhost:8081 をブラウザで開く

# 4. 終了時
curl -X DELETE http://localhost:8080/api/workspaces/my-project
pkill wezterm-parallel
```

### 2. WezTerm統合での効率的な操作

```lua
-- WezTerm内でのキーボードショートカット活用

-- 新規ワークスペース作成: Ctrl+Shift+N
-- ダッシュボード表示: Ctrl+Shift+D  
-- 状態確認: Ctrl+Alt+S

-- 基本的なペイン操作
-- 水平分割: Alt+Enter
-- 垂直分割: Alt+Shift+Enter
-- ペイン移動: Alt+h/j/k/l
```

## 🎨 実用的なユースケース

### ユースケース1: フルスタック開発

```yaml
# 設定例: ~/.config/wezterm-parallel/templates/fullstack.yaml

name: "フルスタック開発環境"
description: "フロントエンド・バックエンド・データベースの統合開発"

layout:
  type: "grid"
  panes:
    - id: "frontend"
      position: { row: 0, col: 0, width: 0.4, height: 0.6 }
      title: "Frontend Dev"
      command: "cd frontend && npm run dev"
      
    - id: "backend"  
      position: { row: 0, col: 1, width: 0.4, height: 0.6 }
      title: "Backend API"
      command: "cd backend && cargo run"
      
    - id: "database"
      position: { row: 1, col: 0, width: 0.4, height: 0.4 }
      title: "Database"
      command: "docker run -p 5432:5432 postgres:15"
      
    - id: "logs"
      position: { row: 1, col: 1, width: 0.4, height: 0.4 }
      title: "Combined Logs"
      command: "tail -f frontend/logs/*.log backend/logs/*.log"
      
    - id: "claude_assistant"
      position: { row: 0, col: 2, width: 0.2, height: 1.0 }
      title: "Claude Assistant"
      command: "claude-code --workspace fullstack-project"

processes:
  - name: "frontend-dev-server"
    command: "npm run dev"
    working_dir: "./frontend"
    environment:
      NODE_ENV: "development"
      VITE_API_URL: "http://localhost:8000"
    auto_restart: true
    
  - name: "backend-api-server"
    command: "cargo run"
    working_dir: "./backend"
    environment:
      RUST_LOG: "debug"
      DATABASE_URL: "postgresql://user:pass@localhost:5432/devdb"
    depends_on: ["postgres-db"]
    
  - name: "postgres-db"
    command: "docker run --rm -p 5432:5432 -e POSTGRES_PASSWORD=devpass postgres:15"
    health_check:
      command: "pg_isready -h localhost -p 5432"
      interval: 10
      timeout: 5
```

**実行方法**:
```bash
# カスタムテンプレートでワークスペース作成
curl -X POST http://localhost:8080/api/workspaces \
  -H "Content-Type: application/json" \
  -d '{"name": "fullstack-project", "template": "fullstack"}'
```

### ユースケース2: マイクロサービス開発

```yaml
# 設定例: ~/.config/wezterm-parallel/templates/microservices.yaml

name: "マイクロサービス開発"
description: "複数サービスの並行開発とテスト"

layout:
  type: "tabs"
  tabs:
    - name: "User Service"
      panes:
        - id: "user-api"
          command: "cd services/user && cargo run"
        - id: "user-tests"
          command: "cd services/user && cargo watch -x test"
          
    - name: "Order Service"  
      panes:
        - id: "order-api"
          command: "cd services/order && cargo run"
        - id: "order-tests"
          command: "cd services/order && cargo watch -x test"
          
    - name: "Gateway"
      panes:
        - id: "api-gateway"
          command: "cd gateway && npm run dev"
        - id: "monitoring"
          command: "docker run -p 3000:3000 grafana/grafana"

processes:
  - name: "service-user"
    command: "cargo run"
    working_dir: "./services/user"
    environment:
      SERVICE_PORT: "8001"
      DATABASE_URL: "postgresql://localhost:5432/userdb"
      
  - name: "service-order"
    command: "cargo run"
    working_dir: "./services/order"
    environment:
      SERVICE_PORT: "8002"
      DATABASE_URL: "postgresql://localhost:5432/orderdb"
      
  - name: "api-gateway"
    command: "npm run dev"
    working_dir: "./gateway"
    environment:
      USER_SERVICE_URL: "http://localhost:8001"
      ORDER_SERVICE_URL: "http://localhost:8002"
    depends_on: ["service-user", "service-order"]
```

### ユースケース3:機械学習開発

```yaml
# 設定例: ~/.config/wezterm-parallel/templates/ml-development.yaml

name: "機械学習開発環境"
description: "データ処理・モデル訓練・実験管理"

layout:
  type: "split"
  orientation: "horizontal"
  panes:
    - id: "jupyter"
      size: 0.6
      command: "jupyter lab --no-browser --port=8888"
      
    - id: "tensorboard"
      size: 0.2
      command: "tensorboard --logdir=./logs --port=6006"
      
    - id: "monitoring"
      size: 0.2
      command: "htop"

processes:
  - name: "data-preprocessing"
    command: "python scripts/preprocess.py"
    working_dir: "./ml-project"
    environment:
      CUDA_VISIBLE_DEVICES: "0"
      DATA_PATH: "./data/raw"
    auto_restart: false
    
  - name: "model-training"
    command: "python train.py --config configs/experiment_001.yaml"
    working_dir: "./ml-project"
    environment:
      CUDA_VISIBLE_DEVICES: "0,1"
      WANDB_PROJECT: "my-ml-project"
    restart_policy: "on-failure"
    
  - name: "model-serving"
    command: "uvicorn api.main:app --host 0.0.0.0 --port 8000"
    working_dir: "./ml-project"
    depends_on: ["model-training"]
```

## 🛠️ 高度な設定とカスタマイズ

### 1. 環境変数による設定管理

```bash
# ~/.bashrc または ~/.zshrc に追加

# WezTerm Parallel環境変数
export WEZTERM_PARALLEL_CONFIG="$HOME/.config/wezterm-parallel/config.yaml"
export WEZTERM_PARALLEL_LOG_LEVEL="info"
export WEZTERM_PARALLEL_API_HOST="127.0.0.1"
export WEZTERM_PARALLEL_API_PORT="8080"

# Claude Code設定
export CLAUDE_CODE_WORKSPACE_DIR="$HOME/code"
export CLAUDE_CODE_MAX_INSTANCES="3"

# 開発プロジェクト用ショートカット
alias wp='wezterm-parallel'
alias wpstat='curl -s http://localhost:8080/api/status | jq'
alias wpdash='open http://localhost:8081'
```

### 2. プロジェクト固有の設定

```yaml
# プロジェクトルート/.wezterm-parallel.yaml
# このファイルがあると、プロジェクト固有の設定が適用される

project:
  name: "my-awesome-project"
  default_template: "custom-stack"
  
workspace:
  auto_save_interval: 30
  backup_enabled: true
  
processes:
  max_per_workspace: 6
  health_check_interval: 15
  
claude_code:
  instances: 2
  working_directory: "./src"
  additional_args: ["--memory-limit", "1GB"]
  
environment:
  NODE_ENV: "development"
  RUST_LOG: "debug"
  DATABASE_URL: "sqlite://dev.db"
```

### 3. 高度なスクリプト自動化

```bash
#!/bin/bash
# scripts/dev-setup.sh - 開発環境の自動セットアップ

set -e

PROJECT_NAME="my-project"
WORKSPACE_TEMPLATE="fullstack"

echo "🚀 $PROJECT_NAME の開発環境をセットアップ中..."

# 1. 必要なサービスの起動確認
if ! pgrep -f wezterm-parallel > /dev/null; then
    echo "フレームワークを起動中..."
    wezterm-parallel &
    sleep 3
fi

# 2. データベース準備
echo "データベースをセットアップ中..."
docker run -d --name ${PROJECT_NAME}_db \
    -p 5432:5432 \
    -e POSTGRES_PASSWORD=devpass \
    -e POSTGRES_DB=${PROJECT_NAME} \
    postgres:15

# 3. 依存関係インストール
echo "依存関係をインストール中..."
npm install --prefix frontend/
cargo build --manifest-path backend/Cargo.toml

# 4. ワークスペース作成
echo "ワークスペースを作成中..."
curl -X POST http://localhost:8080/api/workspaces \
    -H "Content-Type: application/json" \
    -d "{\"name\": \"$PROJECT_NAME\", \"template\": \"$WORKSPACE_TEMPLATE\"}"

# 5. ダッシュボードを開く
echo "ダッシュボードを開いています..."
open http://localhost:8081

echo "✅ セットアップ完了！"
echo "📊 ダッシュボード: http://localhost:8081"
echo "🔧 API: http://localhost:8080/api/status"
```

## ⚡ パフォーマンス最適化のベストプラクティス

### 1. リソース使用量の監視

```bash
# リアルタイムリソース監視
watch -n 2 'curl -s http://localhost:8080/api/metrics | jq .resource_usage'

# プロセス別メモリ使用量
curl -s http://localhost:8080/api/processes | jq '.[] | {name: .name, memory_mb: .memory_usage_mb}'

# システム全体の監視
curl -s http://localhost:8080/api/system/metrics | jq .
```

### 2. 設定の最適化

```yaml
# パフォーマンス重視の設定
performance:
  # プロセス管理
  max_concurrent_processes: 4        # CPUコア数に合わせて調整
  process_spawn_delay_ms: 500       # プロセス起動間隔
  
  # メモリ管理
  memory_limits:
    per_process_mb: 512
    total_workspace_mb: 2048
    garbage_collection_interval: 300
    
  # ディスクI/O
  file_sync:
    async_writes: true
    batch_size: 100
    flush_interval_ms: 1000
    
  # ネットワーク
  websocket:
    ping_interval: 30
    message_queue_size: 1000
    compression_enabled: true
```

### 3. プロファイリングと診断

```bash
# CPUプロファイリング
RUST_LOG=trace wezterm-parallel --profile=cpu --duration=60s

# メモリプロファイリング  
RUST_LOG=trace wezterm-parallel --profile=memory --output=memory-profile.json

# ネットワーク診断
wezterm-parallel --diagnose-network

# 設定検証
wezterm-parallel --validate-config --verbose
```

## 🔧 トラブルシューティング

### よくある問題と解決策

#### 1. パフォーマンス関連

**問題**: システムが重い・応答が遅い
```bash
# 診断手順
# 1. リソース使用量確認
curl -s http://localhost:8080/api/system/resources

# 2. プロセス状況確認
curl -s http://localhost:8080/api/processes | jq '.[] | select(.status != "running")'

# 3. ログ確認
tail -f ~/.config/wezterm-parallel/logs/application.log | grep -E "(ERROR|WARN)"

# 解決策
# - プロセス数制限の調整
# - メモリ制限の見直し
# - 不要なプロセスの停止
```

**問題**: メモリ使用量が多い
```bash
# メモリリーク検出
wezterm-parallel --detect-memory-leaks

# プロセス別メモリ使用量
ps aux | grep -E "(wezterm-parallel|claude-code)" | awk '{print $2, $4, $11}'

# 解決策：メモリ制限の設定
# config.yaml:
process_management:
  memory_limits:
    per_process_mb: 512
    auto_kill_threshold_mb: 1024
```

#### 2. 接続関連

**問題**: APIに接続できない
```bash
# 診断
netstat -tlnp | grep -E "(8080|8081)"
curl -v http://localhost:8080/api/health

# よくある原因と解決策
# 1. ポート競合
sudo lsof -i :8080 -i :8081
# → 他プロセス停止またはポート変更

# 2. ファイアウォール
sudo ufw status
# → 必要ポートの許可

# 3. 設定エラー  
wezterm-parallel --check-config
# → 設定ファイル修正
```

**問題**: WebSocketダッシュボードが更新されない
```bash
# WebSocket接続診断
curl -i -N -H "Connection: Upgrade" \
     -H "Upgrade: websocket" \
     -H "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
     -H "Sec-WebSocket-Version: 13" \
     http://localhost:8081/ws

# 解決策
# 1. ブラウザキャッシュクリア
# 2. WebSocket設定確認
# 3. プロキシ設定の確認
```

#### 3. プロセス管理関連

**問題**: プロセスが起動しない
```bash
# プロセス起動ログ確認
tail -f ~/.config/wezterm-parallel/logs/processes/workspace-name/process-name.log

# 権限確認
ls -la $(which claude-code)
ls -la ~/.config/wezterm-parallel/

# 環境変数確認
env | grep -E "(PATH|RUST_|NODE_|PYTHON_)"

# 解決策
# 1. 実行権限の付与
# 2. PATH設定の確認
# 3. 依存関係の確認
```

**問題**: プロセスが頻繁に再起動する
```bash
# 再起動ログ確認
grep "restart" ~/.config/wezterm-parallel/logs/application.log

# プロセス詳細確認
curl -s http://localhost:8080/api/processes/failing-process | jq .

# 解決策
# 1. ヘルスチェック設定の調整
# 2. タイムアウト設定の増加
# 3. 依存関係の確認
```

### 4. Claude Code統合関連

**問題**: Claude Codeが起動しない
```bash
# Claude Code直接テスト
claude-code --version
claude-code --help

# 設定確認
grep -A 10 "claude_code:" ~/.config/wezterm-parallel/config.yaml

# ログ確認
grep "claude" ~/.config/wezterm-parallel/logs/application.log

# 解決策
# 1. Claude Codeパスの確認
# 2. 認証状態の確認
# 3. 設定でauto_start: falseに変更して手動起動テスト
```

## 🎓 上級者向けテクニック

### 1. カスタムプラグイン開発

```yaml
# ~/.config/wezterm-parallel/plugins/productivity-tracker.yaml
plugin:
  name: "productivity-tracker"
  version: "1.0.0"
  description: "作業時間とタスク完了率を追跡"
  
  hooks:
    - event: "workspace_created"
      action: "start_time_tracking"
    - event: "process_started"  
      action: "log_activity"
    - event: "workspace_closed"
      action: "generate_summary"
      
  api_endpoints:
    - path: "/api/plugins/productivity/summary"
      method: "GET"
      handler: "get_productivity_summary"
      
  dashboard_widgets:
    - name: "productivity-chart"
      position: "bottom-right"
      size: "small"
```

### 2. 外部ツール統合

```bash
#!/bin/bash
# scripts/integrate-with-ide.sh - IDEとの統合スクリプト

# VS Code統合
code --install-extension wezterm-parallel-vscode
echo '{"wezterm.parallel.apiUrl": "http://localhost:8080"}' > .vscode/settings.json

# JetBrains IDE統合  
curl -X POST http://localhost:8080/api/integrations/jetbrains \
    -d '{"ide": "IntelliJ IDEA", "project_path": "'$(pwd)'"}'

# Vim/Neovim統合
echo 'let g:wezterm_parallel_api = "http://localhost:8080"' >> ~/.vimrc
```

### 3. CI/CD統合

```yaml
# .github/workflows/wezterm-parallel-test.yml
name: WezTerm Parallel Integration Test

on: [push, pull_request]

jobs:
  test-with-wezterm-parallel:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup WezTerm Parallel
      run: |
        cargo build --release
        ./setup.sh
        
    - name: Start Framework  
      run: |
        ./target/release/wezterm-parallel &
        sleep 5
        
    - name: Run Integration Tests
      run: |
        # テスト用ワークスペース作成
        curl -X POST http://localhost:8080/api/workspaces \
          -d '{"name": "ci-test", "template": "basic"}'
          
        # テスト実行
        npm test
        cargo test
        
    - name: Cleanup
      run: |
        curl -X DELETE http://localhost:8080/api/workspaces/ci-test
        pkill wezterm-parallel
```

## 📊 メトリクスとアナリティクス

### 生産性測定

```bash
# 日次レポート生成
curl -s http://localhost:8080/api/analytics/daily-report | jq .

# 週次サマリー
curl -s http://localhost:8080/api/analytics/weekly-summary | jq .

# プロジェクト別統計
curl -s http://localhost:8080/api/analytics/project-stats | jq .
```

### パフォーマンス監視

```bash
# ダッシュボード用メトリクス
curl -s http://localhost:8080/api/metrics/dashboard | jq .

# システムヘルス
curl -s http://localhost:8080/api/system/health | jq .

# 異常検知
curl -s http://localhost:8080/api/alerts/active | jq .
```

## 🔮 次のステップ

1. **[API Documentation](https://daktu32.github.io/wezterm-parallel/)**: プログラム的な操作
2. **[カスタマイズガイド](CUSTOMIZATION.md)**: テーマ・プラグイン・拡張機能
3. **[管理者ガイド](ADMIN-GUIDE.md)**: チームでの運用・管理
4. **[FAQ](FAQ.md)**: よくある質問と回答

---

🎉 **これでWezTerm Parallelマスターです！** 

より高度な使い方や質問があれば [GitHubのDiscussions](https://github.com/daktu32/wezterm-parallel/discussions) でお気軽にご相談ください。