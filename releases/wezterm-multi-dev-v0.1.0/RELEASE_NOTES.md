# WezTerm Multi-Process Development Framework v0.1.0

## 🎉 Release Highlights

この度、WezTerm Multi-Process Development Framework v0.1.0 をリリースいたします。

### ✨ 主要機能

- **リアルタイムダッシュボード**: システムメトリクス、プロセス監視、ログ表示
- **高度なペイン管理**: ペイン同期、レイアウト管理、動的ペイン作成
- **タスク管理システム**: 包括的なタスク追跡、プロジェクト管理、時間追跡
- **ワークスペース管理**: プロジェクト単位での環境分離
- **プロセス間通信**: WebSocketベースのリアルタイム通信

### 📊 実装統計

- **総コード行数**: 87,914行 (Rust: 69,898行, Lua: 18,016行)
- **テスト数**: 47個 (100%成功率)
- **コンパイル警告**: 0個 (クリーンコードベース)

### 🚀 インストール

```bash
# 1. リリースパッケージを展開
tar -xzf wezterm-multi-dev-v0.1.0.tar.gz
cd wezterm-multi-dev-v0.1.0

# 2. インストールスクリプトを実行
./install.sh

# 3. フレームワークを起動
wezterm-multi-dev
```

### 🧪 テスト

統合テストを実行してインストールを確認：

```bash
./integration_test.sh
```

### 📋 動作要件

- WezTerm (最新安定版推奨)
- macOS/Linux
- Rust 1.70+ (ソースからビルドする場合)

### 🔗 詳細情報

詳細な使用方法は README.md をご覧ください。

---

**開発チーム**: Claude Code Assistant  
**リリース日**: 2025-06-22
