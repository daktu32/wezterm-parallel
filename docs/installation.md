# 📥 インストールガイド

WezTerm Multi-Process Development Frameworkの詳細なインストール手順です。

## 前提条件

### システム要件

- **OS**: macOS 10.15+, Linux (Ubuntu 20.04+), Windows 10+
- **RAM**: 最小 4GB, 推奨 8GB+
- **ディスク**: 最小 1GB の空き容量

### 必要ソフトウェア

#### 1. WezTerm
```bash
# macOS (Homebrew)
brew install --cask wezterm

# Ubuntu/Debian
curl -LO https://github.com/wez/wezterm/releases/download/20230712-072601-f4abf8fd/wezterm-20230712-072601-f4abf8fd.Ubuntu20.04.deb
sudo dpkg -i wezterm-*.deb

# Windows
# https://github.com/wez/wezterm/releases から最新版をダウンロード
```

#### 2. Rust (1.70+)
```bash
# Rust インストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# バージョン確認
rustc --version
cargo --version
```

#### 3. Git
```bash
# macOS
brew install git

# Ubuntu/Debian  
sudo apt install git

# Windows
# https://git-scm.com/download/win からダウンロード
```

## インストール方法

### クイックインストール (推奨)

```bash
# 1. リポジトリをクローン
git clone https://github.com/your-org/wezterm-parallel.git
cd wezterm-parallel

# 2. インストールスクリプトを実行
./scripts/install.sh
```

### 手動インストール

#### Step 1: プロジェクトのクローン

```bash
git clone https://github.com/your-org/wezterm-parallel.git
cd wezterm-parallel
```

#### Step 2: バックエンドサービスのビルド

```bash
# 依存関係の確認
cargo check

# リリースビルド
cargo build --release

# ビルドの確認
./target/release/wezterm-multi-dev --version
```

#### Step 3: WezTerm設定のセットアップ

```bash
# 設定ディレクトリを作成
mkdir -p ~/.config/wezterm-multi-dev
mkdir -p ~/.config/wezterm

# 設定ファイルをコピー
cp config/wezterm.lua ~/.config/wezterm/
cp config/framework.yaml ~/.config/wezterm-multi-dev/

# シンボリックリンクを作成 (オプション)
ln -sf $(pwd)/wezterm-config ~/.config/wezterm-multi-dev/lua
```

#### Step 4: システムサービスの設定

```bash
# systemd サービスファイルを作成 (Linux)
sudo cp config/systemd/wezterm-multi-dev.service /etc/systemd/system/
sudo systemctl enable wezterm-multi-dev
sudo systemctl start wezterm-multi-dev

# macOS LaunchAgent (macOS)
cp config/launchd/com.wezterm-multi-dev.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.wezterm-multi-dev.plist
```

## 設定

### WezTerm設定ファイル

`~/.config/wezterm/wezterm.lua` を編集:

```lua
local wezterm = require 'wezterm'
local config = wezterm.config_builder()

-- フレームワーク読み込み
local framework_path = wezterm.home_dir .. '/.config/wezterm-multi-dev/lua'
package.path = package.path .. ';' .. framework_path .. '/?.lua'

local dashboard = require 'ui.dashboard_enhanced'
local pane_manager = require 'ui.pane_enhanced'
local task_manager = require 'ui.task_manager'

-- フレームワーク初期化
local framework_config = {
  dashboard = {
    enabled = true,
    update_interval = 5000,
    theme = "catppuccin",
  },
  pane_enhanced = {
    enabled = true,
    auto_optimize = true,
    smart_suggestions = true,
  },
  task_manager = {
    enabled = true,
    auto_save = true,
    time_tracking = true,
  },
}

dashboard.init(framework_config)
pane_manager.init(framework_config)
task_manager.init(framework_config)

-- キーバインド設定
config.keys = {
  -- ダッシュボード
  { key = 'd', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    dashboard.show_dashboard(window, pane)
  end)},
  
  -- ペイン管理
  { key = 'p', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    pane_manager.show_management_menu(window, pane)
  end)},
  
  -- タスク管理
  { key = 't', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    task_manager.show_task_list(window, pane)
  end)},
  
  -- ペイン同期トグル
  { key = 's', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    local tab = window:active_tab()
    local pane_sync = require 'ui.pane_sync'
    pane_sync.toggle_tab_sync(tab)
  end)},
  
  -- レイアウト選択
  { key = 'l', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    local layout_manager = require 'ui.layout_manager'
    layout_manager.show_layout_selector(window, pane)
  end)},
  
  -- ログビューア
  { key = 'g', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    local log_viewer = require 'ui.log_viewer'
    log_viewer.show_log_viewer(window, pane)
  end)},
}

-- テーマ設定
config.color_scheme = 'Catppuccin Mocha'
config.font = wezterm.font('JetBrains Mono', { weight = 'Medium' })
config.font_size = 13

-- タブバー設定
config.use_fancy_tab_bar = true
config.tab_bar_at_bottom = false

return config
```

### フレームワーク設定ファイル

`~/.config/wezterm-multi-dev/config.yaml`:

```yaml
# フレームワーク基本設定
framework:
  version: "1.0.0"
  auto_start: true
  log_level: "info"
  data_dir: "~/.local/share/wezterm-multi-dev"
  socket_path: "/tmp/wezterm-multi-dev.sock"

# ダッシュボード設定
dashboard:
  enabled: true
  theme: "catppuccin"
  update_interval: 3000
  panels:
    - system_metrics
    - process_list
    - log_viewer
    - task_summary
  
  metrics:
    cpu_history_length: 60
    memory_history_length: 60
    network_monitoring: true
    disk_monitoring: true

# ペイン管理設定
pane_management:
  enabled: true
  auto_sync: false
  layout_persistence: true
  max_panes_per_tab: 8
  
  sync:
    broadcast_delay_ms: 50
    exclude_patterns:
      - "^exit$"
      - "^logout$"
      - "^reboot$"
      - "^shutdown"
      - "^rm -rf"
      - "^sudo rm"
  
  layouts:
    auto_apply: true
    save_custom: true
    suggestions: true

# タスク管理設定
task_management:
  enabled: true
  auto_save: true
  save_interval: 300
  time_tracking: true
  notifications: true
  
  kanban:
    enabled: true
    columns:
      - "todo"
      - "in_progress"
      - "review" 
      - "completed"

# ログ設定
logging:
  enabled: true
  level: "info"
  file: "~/.local/share/wezterm-multi-dev/logs/framework.log"
  rotation: "daily"
  max_files: 7
  
  collection:
    buffer_size: 10000
    flush_interval: 1000
    realtime_streaming: true

# プロセス管理設定
process_management:
  enabled: true
  auto_restart: true
  health_check_interval: 30
  
  claude_code:
    auto_start: false
    instances: 1
    timeout: 300

# テーマ設定
themes:
  current: "catppuccin"
  
  catppuccin:
    colors:
      primary: "#89b4fa"
      secondary: "#a6e3a1"
      warning: "#f9e2af"
      error: "#f38ba8"
      background: "#1e1e2e"
      foreground: "#cdd6f4"
    
    fonts:
      ui: "JetBrains Mono"
      size: 12
```

## 起動と確認

### バックエンドサービスの起動

```bash
# フォアグラウンドで起動
./target/release/wezterm-multi-dev

# バックグラウンドで起動
./target/release/wezterm-multi-dev --daemon

# ログ確認
tail -f ~/.local/share/wezterm-multi-dev/logs/framework.log
```

### WezTermでの確認

1. **WezTermを起動**
   ```bash
   wezterm
   ```

2. **フレームワーク機能を確認**
   - `Ctrl+Shift+D`: ダッシュボード表示
   - `Ctrl+Shift+P`: ペイン管理メニュー
   - `Ctrl+Shift+T`: タスク管理

3. **ログ確認**
   ```bash
   # フレームワークログ
   tail -f ~/.local/share/wezterm-multi-dev/logs/framework.log
   
   # WezTermログ (デバッグ情報)
   wezterm --config 'log_level="DEBUG"'
   ```

## トラブルシューティング

### よくある問題

#### 1. "wezterm-multi-dev command not found"

```bash
# パスの確認
echo $PATH

# バイナリの場所確認
which wezterm-multi-dev

# パスに追加
export PATH="$PATH:$(pwd)/target/release"
echo 'export PATH="$PATH:/path/to/wezterm-parallel/target/release"' >> ~/.bashrc
```

#### 2. "Failed to bind socket"

```bash
# 既存のソケットファイルを削除
rm -f /tmp/wezterm-multi-dev.sock

# 権限確認
ls -la /tmp/wezterm-multi-dev.sock

# 再起動
./target/release/wezterm-multi-dev
```

#### 3. "Lua module not found"

```bash
# 設定パスの確認
ls -la ~/.config/wezterm-multi-dev/lua/

# シンボリックリンクの再作成
ln -sf $(pwd)/wezterm-config ~/.config/wezterm-multi-dev/lua

# WezTerm設定の確認
wezterm show-config
```

#### 4. ダッシュボードが表示されない

```bash
# WebSocketサービスの確認
curl --include \
     --no-buffer \
     --header "Connection: Upgrade" \
     --header "Upgrade: websocket" \
     --header "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
     --header "Sec-WebSocket-Version: 13" \
     http://localhost:8080/ws

# フレームワークの再起動
pkill wezterm-multi-dev
./target/release/wezterm-multi-dev
```

### デバッグモード

```bash
# デバッグログ有効化
export RUST_LOG=debug
./target/release/wezterm-multi-dev

# WezTermデバッグモード
wezterm --config 'log_level="DEBUG"' --config 'debug_key_events=true'
```

### 完全なアンインストール

```bash
# サービス停止
sudo systemctl stop wezterm-multi-dev    # Linux
launchctl unload ~/Library/LaunchAgents/com.wezterm-multi-dev.plist  # macOS

# ファイル削除
rm -rf ~/.config/wezterm-multi-dev
rm -rf ~/.local/share/wezterm-multi-dev
rm -f ~/.config/wezterm/wezterm.lua

# システムサービス削除
sudo rm /etc/systemd/system/wezterm-multi-dev.service  # Linux
rm ~/Library/LaunchAgents/com.wezterm-multi-dev.plist  # macOS
```

## 次のステップ

インストールが完了したら:

1. [機能ガイド](features/README.md) で各機能の詳細を確認
2. [設定ガイド](configuration.md) でカスタマイズ方法を学習
3. [チュートリアル](tutorial.md) で実際の使用例を試す

## サポート

問題が解決しない場合:

- [GitHub Issues](https://github.com/your-org/wezterm-parallel/issues) でバグ報告
- [GitHub Discussions](https://github.com/your-org/wezterm-parallel/discussions) で質問
- [トラブルシューティングガイド](troubleshooting.md) を確認