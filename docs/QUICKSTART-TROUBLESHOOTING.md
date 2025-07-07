# 🆘 クイックスタート トラブルシューティング

クイックスタートで問題が発生した場合の解決方法です。

## 🔧 よくある問題と解決策

### 1. インストール関連

#### Q1: `cargo build` が失敗する
```bash
# エラー例: "could not find Cargo.toml"
# 解決策: 正しいディレクトリにいることを確認
pwd
ls -la Cargo.toml

# エラー例: "linker error" 
# 解決策: 必要な開発ツールをインストール
# Ubuntu/Debian:
sudo apt update && sudo apt install build-essential

# macOS:
xcode-select --install

# エラー例: "rustc version too old"
# 解決策: Rustを最新版に更新
rustup update
```

#### Q2: バイナリが見つからない
```bash
# パスを確認
echo $PATH
which wezterm-parallel

# 手動でパスを追加
export PATH="$HOME/.cargo/bin:$PATH"
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc

# または絶対パスで実行
./target/release/wezterm-parallel --help
```

### 2. 起動関連

#### Q3: フレームワークが起動しない
```bash
# 詳細ログで原因を確認
RUST_LOG=debug ./target/release/wezterm-parallel

# よくある原因と解決策:

# 原因1: ポートが使用中
lsof -i :8080 -i :8081
# 解決: 他のプロセスを停止するか、設定でポートを変更

# 原因2: 権限エラー
# 解決: ディレクトリ権限を確認
mkdir -p ~/.config/wezterm-parallel
chmod 755 ~/.config/wezterm-parallel

# 原因3: 設定ファイルエラー
# 解決: デフォルト設定を再生成
cp config/quickstart-config.yaml ~/.config/wezterm-parallel/config.yaml
```

#### Q4: "Permission denied" エラー
```bash
# 実行権限を付与
chmod +x ./target/release/wezterm-parallel

# バイナリの場所を確認
ls -la ./target/release/wezterm-parallel

# SELinux有効の場合（Linux）
sudo setsebool -P allow_execheap 1
```

### 3. 接続関連

#### Q5: APIが応答しない
```bash
# サービス状態を確認
ps aux | grep wezterm-parallel

# ポート使用状況を確認
netstat -tlnp | grep :8080
netstat -tlnp | grep :8081

# ファイアウォール確認（必要に応じて）
# Ubuntu:
sudo ufw status
sudo ufw allow 8080
sudo ufw allow 8081

# macOS:
sudo pfctl -sr | grep 8080
```

#### Q6: ダッシュボードにアクセスできない
```bash
# サービス起動確認
curl http://localhost:8080/api/status

# ブラウザで直接確認
# 1. http://localhost:8081
# 2. http://127.0.0.1:8081

# ポート競合確認
lsof -i :8081
```

### 4. WezTerm統合関連

#### Q7: キーバインドが効かない
```bash
# WezTerm設定ファイルの場所確認
ls -la ~/.config/wezterm/wezterm.lua

# 設定ファイルのコピー
cp config/quickstart-wezterm.lua ~/.config/wezterm/wezterm.lua

# WezTerm設定をリロード
# WezTerm内で: Ctrl+Shift+R

# 設定ファイルの構文エラー確認
wezterm show-config
```

#### Q8: WezTermでエラーが表示される
```bash
# WezTermログを確認
# WezTerm内で: Ctrl+Shift+L

# または設定ファイルの構文をチェック
lua -c "dofile('~/.config/wezterm/wezterm.lua')"
```

### 5. Claude Code統合関連

#### Q9: Claude Codeが見つからない
```bash
# Claude Codeの確認
which claude-code
claude-code --version

# Claude Codeがない場合（オプション機能）
# 基本機能は Claude Code なしでも動作します
echo "Claude Code integration is optional"

# Claude Codeを手動で指定
# config.yamlで設定:
# claude_code:
#   binary_path: "/path/to/claude-code"
```

#### Q10: Claude Codeプロセスが起動しない
```bash
# Claude Code統合の無効化（一時的）
# config.yamlで設定:
# claude_code:
#   auto_start: false

# 手動でClaude Codeをテスト
claude-code --help
```

## 🔍 診断コマンド

### システム診断
```bash
# 包括的なシステムチェック
echo "=== System Check ==="
echo "OS: $(uname -a)"
echo "Rust: $(rustc --version)"
echo "WezTerm: $(wezterm --version)"
echo "Claude Code: $(claude-code --version 2>/dev/null || echo 'Not installed')"

echo "=== Network Check ==="
echo "Port 8080: $(lsof -i :8080 || echo 'Available')"
echo "Port 8081: $(lsof -i :8081 || echo 'Available')"

echo "=== File Check ==="
echo "Config dir: $(ls -la ~/.config/wezterm-parallel/ 2>/dev/null || echo 'Not exists')"
echo "WezTerm config: $(ls -la ~/.config/wezterm/wezterm.lua 2>/dev/null || echo 'Not exists')"
```

### ログ確認
```bash
# アプリケーションログ
tail -f ~/.config/wezterm-parallel/logs/application.log

# システムログ（Linux）
journalctl -u wezterm-parallel -f

# 詳細デバッグログ
RUST_LOG=trace ./target/release/wezterm-parallel 2>&1 | tee debug.log
```

## 🚨 緊急時のリセット

### 設定リセット
```bash
# 設定ディレクトリのバックアップ
mv ~/.config/wezterm-parallel ~/.config/wezterm-parallel.backup

# デフォルト設定で再作成
mkdir -p ~/.config/wezterm-parallel
cp config/quickstart-config.yaml ~/.config/wezterm-parallel/config.yaml
```

### 完全リセット
```bash
# 全データの削除（注意: データが失われます）
rm -rf ~/.config/wezterm-parallel
rm -rf ~/.local/share/wezterm-parallel

# WezTerm設定のリセット
mv ~/.config/wezterm/wezterm.lua ~/.config/wezterm/wezterm.lua.backup
```

## 📞 さらなるサポート

### 報告時に含めるべき情報

1. **システム情報**:
   ```bash
   uname -a
   rustc --version
   wezterm --version
   ```

2. **エラーログ**:
   ```bash
   RUST_LOG=debug ./target/release/wezterm-parallel 2>&1
   ```

3. **設定ファイル**:
   ```bash
   cat ~/.config/wezterm-parallel/config.yaml
   ```

### サポートチャンネル

- 🐛 **バグレポート**: [GitHub Issues](https://github.com/daktu32/wezterm-parallel/issues)
- 💡 **機能要求**: [GitHub Discussions](https://github.com/daktu32/wezterm-parallel/discussions)
- 📖 **ドキュメント**: [詳細ガイド](SETUP-GUIDE.md)

---

💡 **ヒント**: 問題が解決しない場合は、[GitHub Issues](https://github.com/daktu32/wezterm-parallel/issues) で詳細な情報（システム情報、エラーログ、設定ファイル）と合わせて報告してください。