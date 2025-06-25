# 🎓 WezTerm マルチプロセス開発フレームワーク - 初心者向けチュートリアル

このチュートリアルでは、WezTerm マルチプロセス開発フレームワークを段階的に学習し、効率的な開発環境を構築する方法を習得できます。

## 📋 前提条件

- WezTerm（最新安定版）がインストールされていること
- Rust 1.70+ がインストールされていること
- Git が使用可能であること
- 基本的なターミナル操作の知識

## 🎯 学習目標

このチュートリアルを完了すると、以下のスキルを習得できます：

- フレームワークの基本操作
- マルチペイン開発環境の構築
- ワークスペース管理の活用
- チーム開発での効率的なワークフロー
- 高度な機能のカスタマイズ

## 📚 シナリオ1: はじめての一歩 (15分)

### 目標
フレームワークの基本操作を体験し、主要機能を理解する

### 手順

#### 1. インストール確認
```bash
# フレームワークのビルド状況確認
cd /path/to/wezterm-parallel
cargo build --release

# バイナリの動作確認
./target/release/wezterm-multi-dev --version
```

#### 2. フレームワーク起動
```bash
# バックエンドサービスをデーモンモードで起動
./target/release/wezterm-multi-dev --daemon

# 起動確認
ps aux | grep wezterm-multi-dev
```

#### 3. WezTerm設定の適用
```bash
# 設定ディレクトリの作成
mkdir -p ~/.config/wezterm-multi-dev
mkdir -p ~/.config/wezterm

# 設定ファイルのコピー
cp config/templates/wezterm.lua ~/.config/wezterm/
cp config/templates/framework.yaml ~/.config/wezterm-multi-dev/
```

#### 4. WezTerm起動と基本操作
```bash
# WezTermを起動
wezterm
```

**実践する操作：**

| キー | 機能 | 確認内容 |
|------|------|----------|
| `Ctrl+Shift+D` | ダッシュボード表示 | システムメトリクス、プロセス一覧の表示 |
| `Ctrl+Shift+P` | ペイン管理メニュー | ペイン作成・管理オプションの確認 |
| `Ctrl+Shift+T` | タスク管理画面 | タスク追跡機能の表示 |
| `Ctrl+Shift+G` | ログビューア | ログ表示・フィルタリング機能 |

#### 5. 期待される結果
- ダッシュボードでシステム情報が表示される
- ペイン管理メニューが表示される
- エラーなくキーバインドが動作する

### トラブルシューティング

**問題**: ダッシュボードが表示されない
```bash
# WebSocketサービスの確認
curl --include \
     --no-buffer \
     --header "Connection: Upgrade" \
     --header "Upgrade: websocket" \
     --header "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
     --header "Sec-WebSocket-Version: 13" \
     http://localhost:9999/ws

# フレームワークの再起動
pkill wezterm-multi-dev
./target/release/wezterm-multi-dev --daemon
```

## 🔧 シナリオ2: Webアプリ開発環境セットアップ (30分)

### 目標
実践的なマルチペイン開発環境を構築し、React アプリケーション開発を体験する

### 準備

#### プロジェクト作成
```bash
# 作業ディレクトリの作成
mkdir ~/tutorial-projects && cd ~/tutorial-projects

# React アプリケーションの作成
npx create-react-app my-react-app
cd my-react-app
```

### ペイン構成の設計

**推奨レイアウト:**
```
┌─────────────────────────────────────────────────────────────┐
│  Workspace: React Development                               │
├─────────────────────┬───────────────────────────────────────┤
│  Main Development   │  Development Server                   │
│  (Claude Code)      │  (npm start)                         │
│                     │                                       │
│                     │                                       │
├─────────────────────┼───────────────────────────────────────┤
│  Dashboard          │  Logs & Testing                       │
│  (System Monitor)   │  (npm test, logs)                    │
│                     │                                       │
└─────────────────────┴───────────────────────────────────────┘
```

### 実践手順

#### 1. 基本ペイン作成
```bash
# WezTermでプロジェクトディレクトリに移動
cd ~/tutorial-projects/my-react-app

# ペイン管理メニューを開く
# Ctrl+Shift+P を押してメニューから選択
```

#### 2. 開発サーバー起動ペイン
```bash
# 新しいペインを作成（ペイン管理メニューから）
# または手動でペイン分割後、以下を実行
npm start
```

#### 3. ダッシュボード表示
```bash
# Ctrl+Shift+D でダッシュボードを表示
# システムメトリクスを確認
```

#### 4. テスト実行ペイン
```bash
# 別のペインでテストを実行
npm test
```

#### 5. Claude Code統合（想定操作）
```bash
# メインペインでClaude Codeを起動（実際の統合は今後実装）
# 現在は手動でエディタ操作をシミュレート
code .
```

### 実践タスク

1. **コンポーネント作成**: `src/components/Tutorial.js` を作成
2. **スタイル変更**: `src/App.css` を編集
3. **リアルタイム確認**: 開発サーバーペインで変更を確認
4. **テスト追加**: `src/App.test.js` にテストを追加

### 期待される結果
- 4つのペインが適切に配置されている
- 開発サーバーが正常に動作している
- ダッシュボードでリソース使用量が確認できる
- ファイル変更が即座に反映される

## 🚀 シナリオ3: チーム開発ワークフロー (45分)

### 目標
マルチプロジェクト・同期機能を活用し、フロントエンド + バックエンド開発を体験する

### プロジェクト構成

#### バックエンドプロジェクト作成
```bash
# APIサーバープロジェクト
mkdir ~/tutorial-projects/api-server && cd ~/tutorial-projects/api-server
npm init -y
npm install express cors
```

#### 簡単なAPIサーバー作成
```javascript
// server.js
const express = require('express');
const cors = require('cors');
const app = express();

app.use(cors());
app.use(express.json());

app.get('/api/hello', (req, res) => {
  res.json({ message: 'Hello from API!' });
});

app.listen(3001, () => {
  console.log('API Server running on port 3001');
});
```

### ワークスペース管理

#### ワークスペース1: フロントエンド
```bash
# Ctrl+Shift+N で新しいワークスペースを作成
# 名前: "Frontend Development"
cd ~/tutorial-projects/my-react-app
```

**ペイン構成:**
1. **開発ペイン**: React コンポーネント開発
2. **サーバーペイン**: `npm start`
3. **テストペイン**: `npm test`
4. **Gitペイン**: バージョン管理

#### ワークスペース2: バックエンド  
```bash
# 新しいワークスペースを作成
# 名前: "Backend Development"
cd ~/tutorial-projects/api-server
```

**ペイン構成:**
1. **開発ペイン**: API開発
2. **サーバーペイン**: `node server.js`
3. **ログペイン**: サーバーログ監視
4. **データベースペイン**: データ管理

### 同期操作の実践

#### ペイン同期機能
```bash
# 同期対象のペインを選択
# Ctrl+Shift+S でペイン同期を有効化

# 両方のワークスペースのGitペインで同時実行
git add .
git commit -m "Implement API integration"
git push origin main
```

#### ワークスペース切り替え
```bash
# Ctrl+Shift+W でワークスペース一覧表示
# キーボードでワークスペースを選択
```

### 統合テスト

#### API連携の実装
```javascript
// my-react-app/src/App.js に追加
useEffect(() => {
  fetch('http://localhost:3001/api/hello')
    .then(res => res.json())
    .then(data => console.log(data));
}, []);
```

#### 動作確認
1. バックエンドワークスペースでAPI起動
2. フロントエンドワークスペースでReact起動  
3. ブラウザでAPIレスポンス確認
4. ダッシュボードで両プロジェクトの状況確認

### 期待される結果
- 2つのワークスペースがスムーズに切り替わる
- ペイン同期でGit操作が同時実行される
- フロントエンド・バックエンド間の通信が成功する
- ダッシュボードで全体的な開発状況が把握できる

## 🏗️ シナリオ4: 高度な機能活用 (60分)

### 目標
フレームワークの全機能をマスターし、カスタマイズされた開発環境を構築する

### カスタム設定の作成

#### フレームワーク設定ファイル
```yaml
# ~/.config/wezterm-multi-dev/config.yaml
framework:
  version: "1.0.0"
  auto_start: true
  log_level: "debug"
  data_dir: "~/.local/share/wezterm-multi-dev"

dashboard:
  enabled: true
  theme: "catppuccin"
  update_interval: 2000
  panels:
    - system_metrics
    - process_list
    - log_viewer
    - task_summary
    - custom_metrics

pane_management:
  enabled: true
  auto_sync: false
  layout_persistence: true
  max_panes_per_tab: 6
  
  sync:
    exclude_patterns:
      - "^git push"
      - "^npm publish"
      - "^rm -rf"
      - "^sudo"

task_management:
  enabled: true
  auto_save: true
  time_tracking: true
  kanban:
    columns:
      - "backlog"
      - "in_progress"
      - "review"
      - "testing"
      - "completed"

themes:
  current: "custom_dark"
  custom_dark:
    colors:
      primary: "#61dafb"
      secondary: "#21232a"
      success: "#28a745"
      warning: "#ffc107"
      error: "#dc3545"
```

#### カスタムキーバインド
```lua
-- ~/.config/wezterm/wezterm.lua の追加設定
config.keys = {
  -- 既存のキーバインドに追加
  
  -- カスタムレイアウト
  { key = '1', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    -- Development Layout (2x2)
    local layout_manager = require 'ui.layout_manager'
    layout_manager.apply_layout(window, pane, "development_2x2")
  end)},
  
  { key = '2', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    -- Testing Layout (1x3)
    local layout_manager = require 'ui.layout_manager'
    layout_manager.apply_layout(window, pane, "testing_1x3")
  end)},
  
  -- 高度なペイン操作
  { key = 'r', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    -- ペインリスタート
    local pane_manager = require 'ui.pane_enhanced'
    pane_manager.restart_pane_process(window, pane)
  end)},
  
  -- タスク管理ショートカット
  { key = 'n', mods = 'CTRL|ALT', action = wezterm.action_callback(function(window, pane)
    -- 新しいタスク作成
    local task_manager = require 'ui.task_manager'
    task_manager.quick_add_task(window, pane)
  end)},
}
```

### 高度な機能の実践

#### 1. カスタムレイアウトの作成
```bash
# Ctrl+Shift+L でレイアウト選択メニューを開く
# "Create Custom Layout" を選択
# レイアウト名: "fullstack_dev"
# ペイン配置をカスタマイズ
```

#### 2. タスク管理システムの活用
```bash
# Ctrl+Shift+T でタスク管理を開く
# 新しいプロジェクトを作成: "Tutorial Completion"
# タスクを追加:
#   - Setup development environment
#   - Implement frontend features  
#   - Create API endpoints
#   - Write integration tests
#   - Deploy application
```

#### 3. メトリクス監視の設定
```yaml
# config.yaml に追加
custom_metrics:
  - name: "Build Time"
    command: "npm run build"
    threshold: 30
    alert_on_failure: true
  
  - name: "Test Coverage"
    command: "npm test -- --coverage --watchAll=false"
    threshold: 80
    type: "percentage"
  
  - name: "Bundle Size"
    command: "du -sh build/"
    threshold: "2M"
    type: "filesize"
```

#### 4. 自動化スクリプトの連携
```bash
# プロジェクトルートに自動化スクリプト作成
# scripts/dev-setup.sh
#!/bin/bash
echo "Setting up development environment..."
npm install
npm run build
npm test
echo "Development environment ready!"
```

#### 5. チーム共有設定の作成
```yaml
# team-config.yaml
team_settings:
  default_layout: "fullstack_dev"
  shared_tasks: true
  notification_channels:
    - slack_webhook: "https://hooks.slack.com/..."
  
  code_standards:
    pre_commit_hooks: true
    linting: true
    formatting: true
    
  deployment:
    auto_deploy_staging: true
    require_approval_production: true
```

### 統合演習

#### プロジェクトの完全ワークフロー
1. **プロジェクト初期化**
   - カスタムレイアウトで環境セットアップ
   - タスク管理でマイルストーン設定

2. **開発フェーズ**
   - フロントエンド・バックエンド並行開発
   - リアルタイムメトリクス監視
   - 自動テスト実行

3. **統合・テストフェーズ**
   - ペイン同期でE2Eテスト実行
   - カバレッジ監視
   - パフォーマンステスト

4. **デプロイフェーズ**
   - ビルドプロセス監視
   - デプロイメント確認
   - 本番環境ヘルスチェック

### 期待される結果
- カスタム設定が正常に適用される
- チーム向けのワークフロー テンプレートが作成される
- 自動化スクリプトが統合されている
- メトリクス監視が継続的に動作する

## 🎓 学習目標の確認

### 基礎レベル ✅
- [x] ダッシュボードでシステム状況確認
- [x] ペイン作成・切り替え操作
- [x] 基本的なキーバインド習得

### 応用レベル ✅
- [x] ワークスペース管理
- [x] ペイン同期機能活用
- [x] カスタム設定の作成

### エキスパートレベル ✅
- [x] チーム向けテンプレート作成
- [x] パフォーマンス最適化
- [x] 外部ツール連携

## 🔄 継続的な学習

### 次のステップ
1. **プロダクション環境での運用**
   - 実際のプロジェクトでの活用
   - チームメンバーへの教育
   - ベストプラクティスの確立

2. **コミュニティ参加**
   - GitHubでのフィードバック提供
   - 新機能の提案
   - バグレポートの作成

3. **拡張機能の開発**
   - カスタムプラグイン作成
   - 外部ツール統合
   - ワークフロー自動化

### リソース
- [フレームワーク公式ドキュメント](../README.md)
- [設定リファレンス](configuration.md)
- [トラブルシューティングガイド](troubleshooting.md)
- [GitHub リポジトリ](https://github.com/your-org/wezterm-parallel)

## 🎉 チュートリアル完了

おめでとうございます！WezTerm マルチプロセス開発フレームワークの基本から高度な機能まで習得しました。

このフレームワークを活用して、より効率的で楽しい開発体験を実現してください。

---

**最終更新**: 2025-06-23  
**バージョン**: 1.0.0  
**作成者**: Claude Code Assistant