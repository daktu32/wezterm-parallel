# 🎯 機能ガイド

WezTerm Multi-Process Development Frameworkの全機能の詳細な使い方ガイドです。

## 📋 機能一覧

### 🎯 [リアルタイムダッシュボード](dashboard.md)
- システムメトリクス監視
- プロセス状況表示
- リアルタイム更新
- カスタマイズ可能なパネル

### 🔄 [ペイン管理システム](pane-management.md)
- [ペイン同期](pane-sync.md)
- [レイアウト管理](layout-management.md)
- [動的ペイン作成](dynamic-panes.md)

### 📋 [タスク管理](task-management.md)
- プロジェクト管理
- 時間追跡
- カンバンボード
- 生産性分析

### 📊 [ログ管理](log-management.md)
- ログ収集・表示
- 高度なフィルタリング
- リアルタイム検索
- ログローテーション

## 🚀 クイックスタート

### 基本操作

#### ダッシュボードの表示
```
Ctrl+Shift+D
```

ダッシュボードでは以下の情報を確認できます：
- CPU使用率とメモリ使用量
- 実行中のプロセス一覧
- システムログ
- タスクの進捗状況

#### ペイン同期の開始
```
Ctrl+Shift+S
```

現在のタブ内の全ペインで入力を同期します。一つのペインで入力したコマンドが他の全ペインでも実行されます。

#### タスク管理画面の表示
```
Ctrl+Shift+T
```

プロジェクトのタスク一覧とカンバンボードを表示します。

### ワークフロー例

#### 1. 開発プロジェクトのセットアップ
```bash
# 1. プロジェクト用のレイアウトを適用
Ctrl+Shift+L → "Development Main"

# 2. ダッシュボードでシステム状況を確認
Ctrl+Shift+D

# 3. タスクを作成
Ctrl+Shift+T → "新機能の実装"
```

#### 2. 複数サーバーでの並行作業
```bash
# 1. 各ペインでサーバーにSSH接続
ssh user@server1
ssh user@server2
ssh user@server3

# 2. ペイン同期を有効化
Ctrl+Shift+S

# 3. 全サーバーで同じコマンドを実行
sudo systemctl status nginx
```

#### 3. ログ監視とデバッグ
```bash
# 1. ログビューアを表示
Ctrl+Shift+G

# 2. フィルターを設定
Level: ERROR
Source: application

# 3. リアルタイムでログを監視
# エラーログが発生すると自動で表示される
```

## 🎨 テーマとカスタマイズ

### テーマの変更

設定ファイル (`~/.config/wezterm-multi-dev/config.yaml`) でテーマを変更：

```yaml
themes:
  current: "catppuccin"  # dark, light, catppuccin, nord
```

### カスタムカラー設定

```yaml
themes:
  custom:
    colors:
      primary: "#your-color"
      secondary: "#your-color"
      warning: "#your-color"
      error: "#your-color"
```

## ⌨️ キーボードショートカット一覧

### 全般
| キー | 機能 |
|------|------|
| `Ctrl+Shift+D` | ダッシュボード表示 |
| `Ctrl+Shift+H` | ヘルプ表示 |
| `Ctrl+Shift+/` | キーバインド一覧 |

### ペイン操作
| キー | 機能 |
|------|------|
| `Ctrl+Shift+P` | ペイン管理メニュー |
| `Ctrl+Shift+S` | ペイン同期トグル |
| `Ctrl+Shift+L` | レイアウト選択 |
| `Ctrl+Shift+N` | 新規ペイン作成 |
| `Ctrl+Shift+X` | ペイン削除 |

### タスク管理
| キー | 機能 |
|------|------|
| `Ctrl+Shift+T` | タスク一覧表示 |
| `Ctrl+Shift+A` | クイックタスク追加 |
| `Ctrl+Shift+K` | カンバンボード表示 |
| `Ctrl+Shift+R` | 時間追跡開始/停止 |

### ログ・監視
| キー | 機能 |
|------|------|
| `Ctrl+Shift+G` | ログビューア表示 |
| `Ctrl+Shift+M` | メトリクス表示 |
| `Ctrl+Shift+F` | ログ検索 |

### ナビゲーション
| キー | 機能 |
|------|------|
| `Ctrl+Tab` | 次のタブ |
| `Ctrl+Shift+Tab` | 前のタブ |
| `Alt+1-9` | タブ直接移動 |
| `Ctrl+Shift+W` | ワークスペース選択 |

## 🔧 高度な使い方

### カスタムレイアウトの作成

1. **レイアウト選択画面を開く**
   ```
   Ctrl+Shift+L
   ```

2. **"Create Custom Layout..."を選択**

3. **レイアウト定義を入力**
   ```lua
   {
     name = "マイカスタムレイアウト",
     panes = {
       {title = "エディタ", role = "editor", size = 0.6},
       {title = "ターミナル", role = "terminal", size = 0.2},
       {title = "ログ", role = "logs", size = 0.2},
     }
   }
   ```

### プロジェクトテンプレートの作成

```yaml
# ~/.config/wezterm-multi-dev/projects/web-app.yaml
name: "Web Application"
layout: "dev_main"
tasks:
  - name: "開発サーバー起動"
    command: "npm run dev"
  - name: "テスト実行"
    command: "npm test"
scripts:
  setup: |
    npm install
    npm run build
```

### 自動化スクリプト

```lua
-- ~/.config/wezterm-multi-dev/lua/custom/automation.lua
local automation = {}

function automation.daily_standup()
    -- 毎日のスタンドアップ用の情報を表示
    local task_manager = require 'ui.task_manager'
    local dashboard = require 'ui.dashboard'
    
    -- 昨日完了したタスク
    local completed_tasks = task_manager.get_completed_tasks_since(yesterday)
    
    -- 今日の予定タスク
    local today_tasks = task_manager.get_tasks_for_today()
    
    -- システム状況
    local system_status = dashboard.get_system_summary()
    
    return {
        completed = completed_tasks,
        planned = today_tasks,
        system = system_status
    }
end

return automation
```

## 📊 パフォーマンス監視

### システムメトリクス

ダッシュボードで監視できる項目：

- **CPU使用率**: リアルタイム・履歴グラフ
- **メモリ使用量**: 物理・仮想メモリ
- **ディスク使用量**: 読み書き速度・使用容量
- **ネットワーク**: 送受信速度
- **プロセス**: CPU・メモリ使用量ランキング

### アラート設定

```yaml
# config.yaml
alerts:
  cpu_threshold: 80
  memory_threshold: 85
  disk_threshold: 90
  notification_channels:
    - desktop
    - wezterm_toast
```

## 🤝 チーム利用

### 設定の共有

```bash
# チーム用設定をエクスポート
wezterm-multi-dev export-config --team > team-config.yaml

# チーム設定をインポート
wezterm-multi-dev import-config team-config.yaml
```

### プロジェクトテンプレート

```yaml
# team-project-template.yaml
name: "チーム開発テンプレート"
layout: "team_dev"
shared_tasks:
  - name: "コードレビュー"
    priority: "high"
  - name: "テストカバレッジ向上"
    priority: "medium"
sync_settings:
  exclude_patterns:
    - "rm -rf"
    - "sudo shutdown"
```

## 次のステップ

- [設定ガイド](../configuration.md) でより詳細なカスタマイズ
- [開発ガイド](../development.md) でプラグイン開発
- [API リファレンス](../api-reference.md) でスクリプト作成

各機能の詳細については、個別の機能ガイドをご覧ください。