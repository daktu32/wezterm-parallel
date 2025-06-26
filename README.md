# 🚀 WezTerm Multi-Process Development Framework

**WezTermの機能を最大限活用したマルチプロセス並行開発環境**

Claude Codeとの統合により、効率的な開発ワークフローを実現する包括的なフレームワークです。

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![WezTerm](https://img.shields.io/badge/wezterm-20230712%2B-green)
![License](https://img.shields.io/badge/license-MIT-blue)

## ✨ 実装済み機能

### ⚡ マルチプロセス基盤
- **プロセス間通信**: Unix Domain Socket による安定した通信
- **ワークスペース管理**: 完全なCRUD操作、テンプレートシステム
- **状態管理**: セッション情報の自動保存・復元
- **プロセス管理**: Claude Code プロセスの起動・監視・再起動

### 📊 監視・メトリクス
- **システム監視**: CPU、メモリ、ディスク使用量の収集
- **プロセス監視**: 実行中プロセスの状態追跡
- **メトリクス保存**: 履歴データの永続化

### 🔧 設定管理
- **YAML設定**: 柔軟な設定ファイルシステム
- **テンプレートエンジン**: ワークスペース作成の自動化
- **環境変数**: プロセス固有の環境設定

## 🚀 開発中・予定機能

### 📱 UI/UX機能 (Phase 2)
- **WebSocketダッシュボード**: リアルタイム監視UI
- **WezTerm統合**: Lua による操作インターフェース
- **ペイン管理**: 高度なレイアウト・同期機能

### 📋 高度機能 (Phase 3)
- **タスク管理**: カンバンボード、時間追跡
- **プラグインシステム**: 外部ツール統合
- **運用監視**: 詳細ログ・分析機能

## 📈 実装状況

**現在の実装: Phase 1基盤完成**
- **Rust**: 6,734行 (完全なワークスペース・プロセス管理)
- **Lua**: 3,239行 (WezTerm統合準備済み)
- **テスト**: 47個のユニットテスト (全て通過)
- **品質**: 型安全、エラーハンドリング、ログ完備

### アーキテクチャ

```
┌─────────────────────────────────────────────────────────────┐
│                     WezTerm Terminal                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │  Workspace A    │  │  Workspace B    │  │ Control Pane │ │
│  │ ┌─────┬─────┬───┐│  │ ┌─────┬─────┐   │  │ ┌──────────┐ │ │
│  │ │Pane1│Pane2│...││  │ │Pane1│Pane2│   │  │ │Dashboard │ │ │
│  │ └─────┴─────┴───┘│  │ └─────┴─────┘   │  │ └──────────┘ │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Lua Configuration Layer                  │
└─────────────────────────────────────────────────────────────┘
            │                    │                    │
            ▼                    ▼                    ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Workspace      │    │  Process        │    │  Communication │
│  Manager        │    │  Manager        │    │  Hub            │
└─────────────────┘    └─────────────────┘    └─────────────────┘
            │                    │                    │
            └────────────────────┼────────────────────┘
                                 ▼
                    ┌─────────────────────────┐
                    │   Backend Services      │
                    │ ┌─────────────────────┐ │
                    │ │ Claude Code         │ │
                    │ │ Process Pool        │ │
                    │ └─────────────────────┘ │
                    │ ┌─────────────────────┐ │
                    │ │ State Management    │ │
                    │ └─────────────────────┘ │
                    │ ┌─────────────────────┐ │
                    │ │ IPC Server          │ │
                    │ └─────────────────────┘ │
                    └─────────────────────────┘
```

## 技術スタック

- **フロントエンド**: WezTerm、Lua
- **バックエンド**: Rust
- **IPC**: Unix Domain Socket
- **設定管理**: YAML/TOML
- **状態管理**: JSON/YAML、SQLite（オプション）

## セットアップ

### 前提条件

- WezTerm（最新安定版）
- Rust 1.70+
- Git

### インストール（プロトタイプ版）

```bash
# リポジトリのクローン
git clone https://github.com/daktu32/wezterm-parallel.git
cd wezterm-parallel

# Rustプロジェクトのビルド
cargo build

# 基本的なIPCサーバーの起動
cargo run
```

**注意**: Phase 1基盤は完成。WezTerm統合UI (Phase 2) は開発中です。

### 設定ファイル

フレームワークは以下の設定ファイルを自動生成・管理します：

```yaml
# ~/.config/wezterm-parallel/workspaces.json
# ワークスペース状態の永続化

# config/templates/framework.yaml  
# デフォルト設定テンプレート

# config/templates/wezterm.lua
# WezTerm統合設定テンプレート (Phase 2で有効化予定)
```

## API仕様

### 現在利用可能な機能

**実装済みAPIエンドポイント:**
- ワークスペース作成・削除・切り替え
- プロセス起動・監視・終了
- メトリクス収集・保存
- 設定ファイル管理

**開発者向け機能:**
- 47個の包括的テストスイート
- 型安全なRust実装
- 詳細なログ・エラーハンドリング
- Unix Domain Socket IPC

## 開発

### プロジェクト構造（実装済み）

```
wezterm-parallel/
├── src/                    # Rustソースコード (6,734行)
│   ├── workspace/          # ワークスペース管理 (完全実装)
│   ├── process/            # プロセス管理 (完全実装)
│   ├── config/             # 設定管理 (完全実装)
│   ├── metrics/            # メトリクス収集 (完全実装)
│   ├── dashboard/          # ダッシュボード基盤
│   ├── lib.rs              # ライブラリエントリ
│   └── main.rs             # メインエントリ
├── lua/                    # WezTerm Lua統合 (3,239行)
│   ├── config/             # 基本設定・キーバインド
│   ├── ui/                 # UI機能 (ダッシュボード等)
│   ├── utils/              # ユーティリティ
│   └── workspace/          # ワークスペース統合
├── config/                 # 設定テンプレート
├── tests/                  # テスト (47個実装済み)
└── docs/                   # ドキュメント
```

### 開発コマンド（利用可能）

```bash
# プロジェクトビルド
cargo build

# 全テスト実行 (47個のテスト)
cargo test

# フレームワーク起動
cargo run

# リリースビルド
cargo build --release

# ドキュメント生成
cargo doc --open
```

## パフォーマンス目標

- ワークスペース起動時間: < 3秒
- ペイン操作レスポンス: < 100ms
- メモリ使用量: ベースライン < 50MB
- CPU使用率（アイドル時）: < 1%

## 🚀 クイックスタート

### インストール

```bash
# リポジトリクローン
git clone https://github.com/daktu32/wezterm-parallel.git
cd wezterm-parallel

# プロジェクトビルド
cargo build --release

# フレームワーク起動
cargo run
```

### 基本操作

```bash
# バックグラウンドでフレームワーク起動
cargo run &

# Unix Domain Socket経由でテスト
echo '{"Ping":{}}' | nc -U /tmp/wezterm-parallel.sock

# ワークスペース作成テスト
echo '{"WorkspaceCreate":{"name":"test","template":"basic"}}' | nc -U /tmp/wezterm-parallel.sock
```

### 開発状況確認

```bash
# 全テスト実行
cargo test  # 47個のテスト全て通過

# プロジェクト統計
find src/ -name "*.rs" -exec wc -l {} + | tail -1  # Rust: 6,734行
find lua/ -name "*.lua" -exec wc -l {} + | tail -1  # Lua: 3,239行
```

## 📊 開発状況

### ✅ 完了済み機能

**Phase 1: 基盤構築 (完了)**
- ✅ Rustプロジェクト完全セットアップ (6,734行)
- ✅ Unix Domain Socket IPC通信システム
- ✅ 完全ワークスペース管理 (CRUD、テンプレート、永続化)
- ✅ 高度プロセス管理 (起動・監視・再起動・ヘルスチェック)
- ✅ メトリクス収集・保存システム
- ✅ YAML設定管理・ホットリロード基盤
- ✅ 包括的テストスイート (47個、全て通過)

### 🔄 開発中

**Phase 2: UI/UX機能 (準備完了)**
- 🔄 WezTerm Lua統合 (基盤実装済み)
- 🔄 WebSocketダッシュボード (設計完了)
- 🔄 ペイン管理システム (設計完了)

### 📋 計画中

**Phase 3: 高度機能**
- 📋 タスク管理・カンバンボード
- 📋 プラグインシステム
- 📋 運用監視・分析機能

### 🔮 将来の拡張予定

**Phase 4: 高度な機能**
- 🔄 プラグインシステム  
- 🔄 AI統合機能強化
- 🔄 クラウド統合・分散ワークスペース

## 貢献

このプロジェクトへの貢献を歓迎します。

### 開発ガイドライン

1. **テストファースト開発**: 実装前にテストを作成 (現在47個実装済み)
2. **段階的開発**: [開発ロードマップ](DEVELOPMENT_ROADMAP.md)に従った実装
3. **品質チェック**: 型安全性・エラーハンドリング・テストカバレッジ
4. **GitHub Issues**: [Issue #8-16](https://github.com/daktu32/wezterm-parallel/issues) で進捗管理

### セキュリティ

- プロセス分離によるセキュリティ確保
- ファイルアクセス制御
- プロセス間通信の暗号化（オプション）

## ライセンス

MIT License

## 参考資料

### プロジェクト文書
- [開発ロードマップ](DEVELOPMENT_ROADMAP.md) - 段階的開発計画
- [プロジェクト要求仕様書](docs/prd.md) - 機能要求定義
- [アーキテクチャ仕様書](docs/ARCHITECTURE.md) - 技術設計

### 外部リソース
- [WezTerm公式ドキュメント](https://wezfurlong.org/wezterm/)
- [Rust公式ドキュメント](https://doc.rust-lang.org/)
- [GitHub Issues](https://github.com/daktu32/wezterm-parallel/issues) - 開発進捗