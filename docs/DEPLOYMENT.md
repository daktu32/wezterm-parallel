# デプロイメントガイド

## 概要

WezTermマルチプロセス開発補助ツールのインストール、設定、運用に関するガイドです。

## 1. システム要件

### 最小要件
- OS: macOS 12.0+, Linux (Ubuntu 20.04+), Windows 10+
- CPU: 2コア以上
- メモリ: 4GB RAM
- ストレージ: 500MB空き容量
- WezTerm: 最新安定版
- Rust: 1.70+ (開発時のみ)

### 推奨要件
- CPU: 4コア以上
- メモリ: 8GB RAM
- SSD推奨

## 2. インストール

### 2.1 バイナリインストール（推奨）

```bash
# リリースページから最新版をダウンロード
wget https://github.com/daktu32/wezterm-parallel/releases/latest/download/wezterm-parallel-$(uname -s)-$(uname -m).tar.gz

# 展開
tar xzf wezterm-parallel-*.tar.gz

# バイナリを適切な場所に配置
sudo mv wezterm-parallel /usr/local/bin/

# 実行権限付与
sudo chmod +x /usr/local/bin/wezterm-parallel
```

### 2.2 ソースからビルド

```bash
# リポジトリクローン
git clone https://github.com/daktu32/wezterm-parallel.git
cd wezterm-parallel

# ビルド
cargo build --release

# バイナリをインストール
sudo cp target/release/wezterm-parallel /usr/local/bin/
```

### 2.3 WezTerm設定統合

```bash
# 設定ディレクトリ作成
mkdir -p ~/.config/wezterm/wezterm-parallel

# Lua設定コピー
cp -r lua/* ~/.config/wezterm/

# テンプレートコピー
cp -r config/templates ~/.config/wezterm/
```

## 3. 初期設定

### 3.1 基本設定ファイル

`~/.config/wezterm-parallel/config.yaml`:
```yaml
# 基本設定
server:
  ipc_socket: /tmp/wezterm-parallel.sock
  websocket_port: 9999
  log_level: info

# Room設定
room:
  max_rooms: 8
  default_template: claude-dev
  state_dir: ~/.wezterm-parallel/rooms

# プロセス設定
process:
  max_processes: 16
  health_check_interval: 30
  restart_attempts: 3

# パフォーマンス設定
performance:
  metrics_interval: 1000
  cache_ttl: 300
  max_file_watchers: 1000
```

### 3.2 WezTerm設定

`~/.config/wezterm/wezterm.lua`に追加:
```lua
-- WezTerm Parallel統合
local wezterm_parallel = require('wezterm-parallel')

-- 初期化
wezterm_parallel.setup({
  socket_path = '/tmp/wezterm-parallel.sock',
  auto_start = true
})

-- キーバインド追加
config.keys = {
  -- Room操作
  { key = 'n', mods = 'CTRL|SHIFT', action = wezterm_parallel.create_room },
  { key = 'w', mods = 'CTRL|SHIFT', action = wezterm_parallel.switch_room },
  
  -- テンプレート操作 (Issue #18)
  { key = 't', mods = 'ALT', action = wezterm_parallel.show_template_picker },
  { key = 'T', mods = 'ALT', action = wezterm_parallel.apply_claude_template },
  { key = 's', mods = 'ALT', action = wezterm_parallel.save_current_layout },
}
```

## 4. 起動と停止

### 4.1 サービス起動

```bash
# フォアグラウンドで起動（デバッグ用）
wezterm-parallel

# バックグラウンドで起動
wezterm-parallel daemon

# systemdサービスとして起動（Linux）
sudo systemctl start wezterm-parallel
sudo systemctl enable wezterm-parallel
```

### 4.2 サービス停止

```bash
# 通常停止
wezterm-parallel stop

# 強制停止
pkill -f wezterm-parallel

# systemd
sudo systemctl stop wezterm-parallel
```

## 5. 使用方法

### 5.1 基本的なワークフロー

```bash
# 1. WezTermを起動
wezterm

# 2. 新規Room作成（Ctrl+Shift+N）
# または
echo '{"RoomCreate":{"name":"my-project","template":"claude-dev"}}' | nc -U /tmp/wezterm-parallel.sock

# 3. テンプレート適用（Alt+T）
# 自動的にペインが分割され、Claude Codeが起動

# 4. タスク実行
# 各ペインで並行して作業を実行
```

### 5.2 協調開発 (Issue #17)

```bash
# メインプロセスでタスク投入
echo '{"CoordinationMessage":{"TaskAssignment":{"to":"process-1","task":{"id":"feat-123","command":"implement login feature"}}}}' | nc -U /tmp/wezterm-parallel.sock

# 結果の確認
echo '{"GetStatus":null}' | nc -U /tmp/wezterm-parallel.sock
```

## 6. トラブルシューティング

### 6.1 起動しない

```bash
# ログ確認
tail -f ~/.wezterm-parallel/logs/wezterm-parallel.log

# ソケットファイル削除
rm -f /tmp/wezterm-parallel.sock

# 権限確認
ls -la ~/.wezterm-parallel/
```

### 6.2 プロセスが応答しない

```bash
# プロセス状態確認
wezterm-parallel status

# ヘルスチェック
echo '{"Ping":null}' | nc -U /tmp/wezterm-parallel.sock

# プロセス再起動
wezterm-parallel restart-process <process-id>
```

### 6.3 テンプレートが適用されない

```bash
# テンプレート検証
wezterm-parallel validate-template ~/.config/wezterm/templates/claude-dev.yaml

# キャッシュクリア
rm -rf ~/.wezterm-parallel/cache/

# Lua設定再読み込み
# WezTerm内で Ctrl+Shift+R
```

## 7. パフォーマンスチューニング

### 7.1 メモリ使用量削減

```yaml
# config.yaml
performance:
  metrics_retention: 3600  # 1時間のみ保持
  max_log_size: 10MB
  cache_size: 100MB
```

### 7.2 CPU使用率最適化

```yaml
process:
  health_check_interval: 60  # チェック間隔を延長
  metrics_interval: 5000     # メトリクス収集頻度を下げる
```

## 8. セキュリティ設定

### 8.1 Unix Socket権限

```bash
# ユーザーのみアクセス可能に設定
chmod 600 /tmp/wezterm-parallel.sock
```

### 8.2 ログのローテーション

```yaml
# config.yaml
logging:
  max_size: 100MB
  max_files: 5
  compression: true
```

## 9. アップデート

### 9.1 バイナリアップデート

```bash
# 現在のバージョン確認
wezterm-parallel --version

# 新バージョンダウンロードとインストール
# (上記インストール手順を参照)

# サービス再起動
wezterm-parallel restart
```

### 9.2 設定の移行

```bash
# 設定バックアップ
cp ~/.config/wezterm-parallel/config.yaml ~/.config/wezterm-parallel/config.yaml.bak

# 新設定の適用
wezterm-parallel migrate-config
```

## 10. アンインストール

```bash
# サービス停止
wezterm-parallel stop

# バイナリ削除
sudo rm /usr/local/bin/wezterm-parallel

# 設定削除（オプション）
rm -rf ~/.config/wezterm-parallel/
rm -rf ~/.wezterm-parallel/

# WezTerm設定から統合部分を削除
# ~/.config/wezterm/wezterm.lua を編集
```