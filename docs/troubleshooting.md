# 🐛 トラブルシューティングガイド

WezTerm Multi-Process Development Frameworkでよくある問題と解決方法をまとめました。

## 🚨 緊急時の対処法

### フレームワークが応答しない場合

```bash
# 1. プロセス強制終了
pkill -f wezterm-multi-dev

# 2. ソケットファイル削除
rm -f /tmp/wezterm-multi-dev.sock

# 3. 設定ファイル確認
wezterm-multi-dev --check-config

# 4. 再起動
./target/release/wezterm-multi-dev
```

### WezTermが起動しない場合

```bash
# 1. 設定ファイルの構文確認
lua -c "dofile('~/.config/wezterm/wezterm.lua')"

# 2. バックアップ設定で起動
wezterm --config-file /dev/null

# 3. デバッグモードで起動
wezterm --config 'log_level="DEBUG"'
```

## 🔧 インストール・セットアップ問題

### "command not found: wezterm-multi-dev"

**原因**: バイナリがパスに含まれていない

```bash
# 解決方法
export PATH="$PATH:/path/to/wezterm-parallel/target/release"

# 永続化
echo 'export PATH="$PATH:/path/to/wezterm-parallel/target/release"' >> ~/.bashrc
source ~/.bashrc
```

### "Failed to bind socket"

**原因**: ソケットファイルの権限問題または既存プロセス

```bash
# 1. 既存プロセス確認
ps aux | grep wezterm-multi-dev

# 2. プロセス終了
pkill -f wezterm-multi-dev

# 3. ソケットファイル削除
sudo rm -f /tmp/wezterm-multi-dev.sock

# 4. 権限確認
ls -la /tmp/

# 5. 代替ソケットパス使用
wezterm-multi-dev --socket-path ~/wezterm-multi-dev.sock
```

### Rustビルドエラー

```bash
# 1. Rustバージョン確認
rustc --version

# 2. Rust更新
rustup update

# 3. 依存関係クリーンアップ
cargo clean

# 4. 再ビルド
cargo build --release

# 5. 依存関係の問題の場合
cargo update
cargo build --release
```

### Luaモジュールエラー

```bash
# 1. Lua設定パス確認
echo $LUA_PATH

# 2. モジュールパス確認
ls -la ~/.config/wezterm-multi-dev/lua/

# 3. シンボリックリンク再作成
ln -sf $(pwd)/wezterm-config ~/.config/wezterm-multi-dev/lua

# 4. WezTerm設定確認
wezterm show-config
```

## 🎯 機能別トラブルシューティング

### ダッシュボード問題

#### ダッシュボードが表示されない

```bash
# 1. WebSocketサービス確認
curl -v http://localhost:8080/ws

# 2. ポート使用状況確認
lsof -i :8080

# 3. 設定確認
grep -A 10 "dashboard:" ~/.config/wezterm-multi-dev/config.yaml

# 4. ログ確認
tail -f ~/.local/share/wezterm-multi-dev/logs/framework.log
```

#### メトリクスが更新されない

```bash
# 1. 更新間隔確認
grep "update_interval" ~/.config/wezterm-multi-dev/config.yaml

# 2. システム権限確認
cat /proc/stat   # CPU情報
cat /proc/meminfo   # メモリ情報

# 3. バックエンドプロセス確認
ps aux | grep wezterm-multi-dev
```

### ペイン同期問題

#### 同期が動作しない

```bash
# 1. 同期グループ確認
# WezTermで Ctrl+Shift+P → "Show sync groups"

# 2. 除外パターン確認
grep -A 5 "exclude_patterns:" ~/.config/wezterm-multi-dev/config.yaml

# 3. ペイン状態確認
# WezTermで Ctrl+Shift+P → "Show pane info"
```

#### 意図しないコマンドが同期される

```bash
# 1. 除外パターン追加
# config.yamlのexclude_patternsに追加:
# - "^your-command-pattern$"

# 2. 同期一時停止
# Ctrl+Shift+S でトグル

# 3. 特定ペインを同期から除外
# ペイン管理メニューから設定
```

### タスク管理問題

#### タスクが保存されない

```bash
# 1. 権限確認
ls -la ~/.local/share/wezterm-multi-dev/

# 2. ディスク容量確認
df -h ~/.local/share/wezterm-multi-dev/

# 3. 自動保存設定確認
grep "auto_save" ~/.config/wezterm-multi-dev/config.yaml

# 4. 手動保存
# タスク管理画面で Ctrl+S
```

#### 時間追跡が機能しない

```bash
# 1. 時間追跡設定確認
grep -A 5 "time_tracking:" ~/.config/wezterm-multi-dev/config.yaml

# 2. システム時刻確認
date

# 3. タスク状態確認
# タスク管理画面で対象タスクの詳細を確認
```

### ログ管理問題

#### ログが表示されない

```bash
# 1. ログ収集設定確認
grep -A 10 "logging:" ~/.config/wezterm-multi-dev/config.yaml

# 2. ログファイル確認
ls -la ~/.local/share/wezterm-multi-dev/logs/

# 3. 権限確認
tail ~/.local/share/wezterm-multi-dev/logs/framework.log

# 4. ログレベル確認
grep "log_level" ~/.config/wezterm-multi-dev/config.yaml
```

#### ログ検索が機能しない

```bash
# 1. 検索インデックス再構築
wezterm-multi-dev --rebuild-index

# 2. メモリ使用量確認
free -h

# 3. ログファイルサイズ確認
du -h ~/.local/share/wezterm-multi-dev/logs/
```

## ⚡ パフォーマンス問題

### 動作が重い

```bash
# 1. CPU使用率確認
top -p $(pgrep wezterm-multi-dev)

# 2. メモリ使用量確認
ps -o pid,vsz,rss,comm -p $(pgrep wezterm-multi-dev)

# 3. 設定最適化
# config.yamlで以下を調整:
# - update_interval: 5000  # 更新間隔を長く
# - buffer_size: 5000      # バッファサイズを小さく
# - max_entries: 1000      # 最大エントリー数を減らす

# 4. ログレベル下げる
# log_level: "warn"
```

### メモリリークの疑い

```bash
# 1. 長期間の監視
while true; do
  ps -o pid,vsz,rss,comm -p $(pgrep wezterm-multi-dev)
  sleep 60
done > memory_usage.log

# 2. デバッグモードで実行
RUST_LOG=debug wezterm-multi-dev

# 3. プロファイリング
# config.yamlでprofiling: trueに設定
```

### WebSocket接続問題

```bash
# 1. 接続テスト
websocat ws://127.0.0.1:8080/ws

# 2. ファイアウォール確認
sudo ufw status
sudo iptables -L

# 3. ポート変更
# config.yamlでport設定を変更

# 4. 接続数制限確認
netstat -an | grep :8080 | wc -l
```

## 🔍 デバッグ手順

### 詳細ログ取得

```bash
# 1. デバッグモード有効化
export RUST_LOG=debug
wezterm-multi-dev

# 2. WezTermデバッグ
wezterm --config 'log_level="DEBUG"' --config 'debug_key_events=true'

# 3. ネットワークデバッグ
tcpdump -i lo port 8080

# 4. システムコール追跡
strace -p $(pgrep wezterm-multi-dev)
```

### 設定検証

```bash
# 1. 設定ファイル構文チェック
wezterm-multi-dev --check-config

# 2. Lua設定チェック
lua -c "dofile('~/.config/wezterm/wezterm.lua')"

# 3. YAML設定検証
python3 -c "import yaml; yaml.safe_load(open('~/.config/wezterm-multi-dev/config.yaml'))"
```

### バックトレース取得

```bash
# 1. コアダンプ有効化
ulimit -c unlimited

# 2. gdbでデバッグ
gdb ./target/release/wezterm-multi-dev core

# 3. バックトレース取得
(gdb) bt
(gdb) thread apply all bt
```

## 📊 システム要件チェック

### 依存関係確認

```bash
# 1. WezTermバージョン
wezterm --version

# 2. Rustバージョン
rustc --version
cargo --version

# 3. システムライブラリ
ldd ./target/release/wezterm-multi-dev

# 4. 利用可能リソース
free -h
df -h
```

### 互換性確認

```bash
# 1. OS確認
uname -a

# 2. ターミナル機能確認
echo $TERM
tput colors

# 3. ロケール確認
locale

# 4. 環境変数確認
env | grep -E "(PATH|RUST|CARGO|WEZTERM)"
```

## 🧹 クリーンアップ手順

### 一時ファイル削除

```bash
# 1. ログファイルクリーンアップ
find ~/.local/share/wezterm-multi-dev/logs/ -name "*.log.*" -mtime +7 -delete

# 2. 一時ファイル削除
rm -rf /tmp/wezterm-multi-dev-*

# 3. キャッシュクリア
rm -rf ~/.cache/wezterm-multi-dev/

# 4. 古いバックアップ削除
find ~/.local/share/wezterm-multi-dev/backups/ -mtime +30 -delete
```

### 設定リセット

```bash
# 1. 設定バックアップ
cp ~/.config/wezterm-multi-dev/config.yaml ~/config-backup.yaml

# 2. デフォルト設定復元
cp config/templates/framework.yaml ~/.config/wezterm-multi-dev/config.yaml

# 3. データベースリセット
rm -f ~/.local/share/wezterm-multi-dev/data.db

# 4. 再起動
wezterm-multi-dev
```

## 📞 サポート情報

### ログ収集スクリプト

```bash
#!/bin/bash
# debug-info.sh - デバッグ情報収集スクリプト

echo "=== System Information ===" > debug-info.txt
uname -a >> debug-info.txt
echo "" >> debug-info.txt

echo "=== WezTerm Version ===" >> debug-info.txt
wezterm --version >> debug-info.txt
echo "" >> debug-info.txt

echo "=== Rust Version ===" >> debug-info.txt
rustc --version >> debug-info.txt
cargo --version >> debug-info.txt
echo "" >> debug-info.txt

echo "=== Process Information ===" >> debug-info.txt
ps aux | grep wezterm >> debug-info.txt
echo "" >> debug-info.txt

echo "=== Configuration ===" >> debug-info.txt
cat ~/.config/wezterm-multi-dev/config.yaml >> debug-info.txt
echo "" >> debug-info.txt

echo "=== Recent Logs ===" >> debug-info.txt
tail -100 ~/.local/share/wezterm-multi-dev/logs/framework.log >> debug-info.txt

echo "Debug information collected in debug-info.txt"
```

### バグ報告テンプレート

**Issue報告時は以下の情報を含めてください：**

1. **環境情報**
   - OS: (例: macOS 14.0, Ubuntu 22.04)
   - WezTermバージョン
   - フレームワークバージョン

2. **発生している問題**
   - 具体的な症状
   - エラーメッセージ
   - 再現手順

3. **設定情報**
   - 変更した設定項目
   - 使用中のテーマ

4. **ログ情報**
   - エラー発生時のログ
   - デバッグモードのログ

### よくある質問

**Q: 複数のWezTermインスタンスで使用できますか？**
A: はい、各インスタンスは独立して動作します。

**Q: リモートサーバーで使用できますか？**
A: SSH経由でも使用可能です。X11フォワーディングが必要な場合があります。

**Q: Windows Subsystem for Linux (WSL) で動作しますか？**
A: はい、WSL2環境で正常に動作します。

**Q: 設定の移行はできますか？**
A: 設定ファイルをコピーするだけで移行可能です。

**Q: プラグイン開発は可能ですか？**
A: はい、Luaベースのプラグインシステムを提供しています。

---

## 🆘 緊急時連絡先

問題が解決しない場合:

- **GitHub Issues**: https://github.com/your-org/wezterm-parallel/issues
- **Discord**: https://discord.gg/wezterm-parallel
- **Email**: support@wezterm-parallel.dev

緊急時は必ず上記のデバッグ情報収集スクリプトの結果を添付してください。