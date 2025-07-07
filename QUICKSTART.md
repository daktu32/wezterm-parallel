# 🚀 クイックスタートガイド

**5分でWezTerm Parallelを動かしてみよう！**

このガイドでは、最小限の設定でWezTerm Parallelを体験できます。

## 📋 事前確認 (1分)

以下がインストール済みか確認してください：

```bash
# WezTermの確認
wezterm --version
# 期待値: wezterm 20240203-110809-5046fc22 (またはそれ以降)

# Rustの確認
rustc --version
# 期待値: rustc 1.70.0 (またはそれ以降)

# Claude Codeの確認 (オプション)
claude-code --version
# ※ Claude Codeがない場合でも基本機能は動作します
```

❌ **まだインストールしていない場合**:
- [WezTerm](https://wezfurlong.org/wezterm/installation.html)
- [Rust](https://rustup.rs/)
- [Claude Code](https://claude.ai/code) (オプション)

## ⚡ インストール (2分)

### 方法1: 自動セットアップスクリプト (最も簡単！)

```bash
# 1. プロジェクトをクローン
git clone https://github.com/daktu32/wezterm-parallel.git
cd wezterm-parallel

# 2. 自動セットアップ実行
./setup.sh
```

✅ **この方法なら**: ビルド・設定ファイル配置・動作確認まで自動実行されます

### 方法2: GitHubから手動インストール

```bash
# 1. プロジェクトをクローン
git clone https://github.com/daktu32/wezterm-parallel.git
cd wezterm-parallel

# 2. ビルド & インストール
cargo build --release

# 3. バイナリの確認
./target/release/wezterm-parallel --help
```

### 方法2: Cargoからインストール

```bash
# Cargo経由でインストール
cargo install --git https://github.com/daktu32/wezterm-parallel

# インストール確認
wezterm-parallel --help
```

### 📂 最小設定ファイルのセットアップ

```bash
# 設定ディレクトリ作成
mkdir -p ~/.config/wezterm-parallel

# クイックスタート用最小設定をコピー
cp config/quickstart-config.yaml ~/.config/wezterm-parallel/config.yaml

# WezTerm用最小設定をコピー（オプション）
cp config/quickstart-wezterm.lua ~/.config/wezterm/wezterm.lua
```

✅ **成功の確認**: ヘルプメッセージが表示され、設定ファイルが配置されればOK

## 🎯 基本動作確認 (2分)

### Step 1: フレームワーク起動

```bash
# 設定ファイルを確認して起動（プロジェクトディレクトリから）
./target/release/wezterm-parallel

# または、Cargo経由でインストールした場合
wezterm-parallel

# バックグラウンドで起動する場合
./target/release/wezterm-parallel &
```

✅ **期待する結果**: 
```
WezTerm Parallel Framework v0.3.0
Loading config from: ~/.config/wezterm-parallel/config.yaml
Starting services...
✓ Process Manager started on localhost:8080
✓ WebSocket Dashboard started on localhost:8081
✓ Ready for connections
```

**設定確認**:
```bash
# 設定が正しく読み込まれているか確認
curl -s http://localhost:8080/api/status | jq
```

### Step 2: 基本的なワークスペース作成

別のターミナルで：

```bash
# テスト用ワークスペースを作成
curl -X POST http://localhost:8080/api/workspaces \
  -H "Content-Type: application/json" \
  -d '{"name": "test-workspace", "template": "basic"}'
```

✅ **期待する結果**: 
```json
{"status": "success", "workspace": "test-workspace"}
```

### Step 3: ダッシュボード確認

ブラウザで以下にアクセス：
```
http://localhost:8081
```

✅ **期待する結果**: 
- WebSocketダッシュボードが表示される
- `test-workspace`が一覧に表示される
- リアルタイムメトリクスが更新される

## 🎉 成功！次のステップ

### すぐに試せること

1. **WezTerm統合** (クイックスタート版):
   ```bash
   # 最小構成のWezTerm設定を適用 (既にコピー済みの場合はスキップ)
   cp config/quickstart-wezterm.lua ~/.config/wezterm/wezterm.lua
   
   # WezTerm再起動後、以下のキーバインドが使用可能：
   # Ctrl+Shift+N: 新しいワークスペース作成
   # Ctrl+Shift+D: ダッシュボードを開く
   # Ctrl+Alt+S: フレームワーク状態確認
   ```
   
   **フル機能版** (後でアップグレード可能):
   ```bash
   # 完全機能のWezTerm設定を適用
   cp config/templates/wezterm.lua ~/.config/wezterm/wezterm.lua
   ```

2. **レイアウトテンプレート試用**:
   ```bash
   # 開発用レイアウトを適用
   curl -X POST http://localhost:8080/api/workspaces/test-workspace/apply-template \
     -H "Content-Type: application/json" \
     -d '{"template": "claude-dev"}'
   ```

3. **Claude Code統合** (Claude Codeがある場合):
   ```bash
   # Claude Codeプロセスを自動起動
   curl -X POST http://localhost:8080/api/workspaces/test-workspace/start-claude
   ```

## 🆘 困ったときは

### よくある問題

**Q1: ポートが使用中というエラーが出る**
```bash
# 他のプロセスを確認
lsof -i :8080
lsof -i :8081

# 必要に応じて他のプロセスを停止
kill -9 <PID>
```

**Q2: フレームワークが起動しない**
```bash
# ログを確認
./target/release/wezterm-parallel --verbose

# または設定ファイルをチェック
ls -la ~/.config/wezterm-parallel/
```

**Q3: WezTermで設定が反映されない**
```bash
# WezTerm設定をリロード
# WezTerm内で: Ctrl+Shift+R

# 設定ファイルの場所を確認
ls -la ~/.config/wezterm/wezterm.lua
```

### サポート

- 📖 [詳細セットアップガイド](SETUP-GUIDE.md)
- 📚 [ユーザーガイド](docs/USER-GUIDE.md)
- 🐛 [Issues (GitHub)](https://github.com/daktu32/wezterm-parallel/issues)
- 💡 [FAQ](docs/FAQ.md)

## 📈 次に学ぶこと

1. **[詳細セットアップガイド](SETUP-GUIDE.md)**: カスタマイズと高度な設定
2. **[ユーザーガイド](docs/USER-GUIDE.md)**: 実用的な使い方とベストプラクティス
3. **[API Documentation](https://daktu32.github.io/wezterm-parallel/)**: 詳細なAPI仕様

---

🎊 **お疲れ様でした！** WezTerm Parallelの基本機能が動作確認できました。

より詳しい使い方は [SETUP-GUIDE.md](SETUP-GUIDE.md) をご覧ください。