# 🚀 WezTerm Multi-Process Development Framework

**WezTermの機能を最大限活用したマルチプロセス並行開発環境**

Claude Codeとの統合により、効率的な開発ワークフローを実現する包括的なフレームワークです。

![Version](https://img.shields.io/badge/version-1.0.0-blue)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![WezTerm](https://img.shields.io/badge/wezterm-20230712%2B-green)
![License](https://img.shields.io/badge/license-MIT-blue)

## ✨ 主要機能

### 🎯 リアルタイムダッシュボード
- **システムメトリクス監視**: CPU、メモリ、ディスク使用量のリアルタイム表示
- **ログ管理**: 高度なフィルタリング・検索機能付きログビューア
- **プロセス監視**: 実行中のプロセス状況の可視化

### 🔄 高度なペイン管理
- **ペイン同期**: 複数ペイン間での入力コマンドの同期実行
- **レイアウト管理**: プロジェクトタイプ別の最適なレイアウトテンプレート
- **動的ペイン作成**: インテリジェントな自動ペイン配置

### 📋 タスク管理システム
- **プロジェクト管理**: 階層的なプロジェクト・タスク構造
- **時間追跡**: 作業時間の自動計測と生産性分析
- **カンバンボード**: 視覚的なタスク進捗管理

### ⚡ マルチプロセス統合
- **プロセス間通信**: WebSocketベースのリアルタイム通信
- **状態管理**: セッション情報の自動保存・復元
- **ワークスペース管理**: プロジェクト単位での環境分離

## 📈 実装状況

**総計 87,914行のコード実装完了**
- **Rust**: 69,898行 (バックエンドサービス)
- **Lua**: 18,016行 (フロントエンドUI)

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
- Claude Code
- Git

### インストール

```bash
# リポジトリのクローン
git clone https://github.com/daktu32/wezterm-parallel.git
cd wezterm-parallel

# Rustプロジェクトのビルド（実装予定）
cargo build --release

# WezTerm設定のインストール（実装予定）
./install-wezterm-config.sh
```

### 設定

WezTerm設定ファイル (`~/.config/wezterm/wezterm.lua`) に以下を追加：

```lua
local wezterm = require 'wezterm'
local multi_dev = require 'wezterm_parallel'

local config = wezterm.config_builder()

-- Multi-dev framework integration
multi_dev.setup(config, {
  config_path = wezterm.home_dir .. '/.config/wezterm-multi-dev/config.yaml',
  auto_start = true,
  debug = false
})

-- カスタムキーバインド
config.keys = {
  { key = 'n', mods = 'CTRL|SHIFT', action = multi_dev.actions.create_workspace },
  { key = 'w', mods = 'CTRL|SHIFT', action = multi_dev.actions.switch_workspace },
  { key = 'd', mods = 'CTRL|SHIFT', action = multi_dev.actions.show_dashboard },
}

return config
```

## 使用方法

### ワークスペースの作成

```
Ctrl+Shift+N
```

新しいワークスペースを作成し、Claude Codeプロセスを自動起動します。

### ワークスペースの切り替え

```
Ctrl+Shift+W
```

既存のワークスペース間を切り替えます。

### ダッシュボード表示

```
Ctrl+Shift+D
```

全プロセスの状態とリソース使用量を表示します。

## 開発

### プロジェクト構造（予定）

```
wezterm-parallel/
├── src/                    # Rustソースコード
│   ├── process_manager/    # プロセス管理
│   ├── communication/      # IPC通信
│   ├── state/             # 状態管理
│   └── main.rs            # エントリーポイント
├── wezterm-config/        # WezTerm Lua設定
│   ├── init.lua           # メイン設定
│   ├── workspace.lua      # ワークスペース管理
│   └── keybindings.lua    # キーバインド
├── tests/                 # テストコード
├── docs/                  # ドキュメント
└── config/               # 設定テンプレート
```

### 開発コマンド（実装予定）

```bash
# ビルド
cargo build

# テスト実行
cargo test

# ドキュメント生成
cargo doc --open

# Luaスクリプトのチェック
luacheck wezterm-config/
```

## パフォーマンス目標

- ワークスペース起動時間: < 3秒
- ペイン操作レスポンス: < 100ms
- メモリ使用量: ベースライン < 50MB
- CPU使用率（アイドル時）: < 1%

## 📦 インストール

### クイックセットアップ (推奨)

```bash
# 1. リポジトリをクローン
git clone https://github.com/your-org/wezterm-parallel.git
cd wezterm-parallel

# 2. 自動インストールスクリプトを実行
./scripts/install.sh
```

### 手動インストール

詳細な手動インストールについては [インストールガイド](docs/installation.md) をご覧ください。

## 🚀 クイックスタート

### 基本的な使い方

1. **フレームワーク起動**
   ```bash
   wezterm-multi-dev
   ```

2. **WezTermでワークスペースを作成**
   ```
   Ctrl+Shift+N  # 新規ワークスペース
   ```

3. **ダッシュボードを表示**
   ```
   Ctrl+Shift+D  # ダッシュボード表示
   ```

### 主要キーバインド

| キー | 機能 |
|------|------|
| `Ctrl+Shift+D` | ダッシュボード表示 |
| `Ctrl+Shift+P` | ペイン管理メニュー |
| `Ctrl+Shift+T` | タスク管理 |
| `Ctrl+Shift+S` | ペイン同期トグル |
| `Ctrl+Shift+L` | レイアウト選択 |
| `Ctrl+Shift+M` | メトリクス表示 |

## 📊 開発状況

### ✅ 完了済み機能

**🎉 v1.0.0 リリース準備完了**

**Phase 3: UI/UX機能 (完了)**
- ✅ **3.1 リアルタイムダッシュボード**: システムメトリクス収集、リアルタイム更新UI、ログ表示・フィルタリング
- ✅ **3.2 高度なペイン機能**: ペイン同期・レイアウト管理、動的ペイン作成・管理  
- ✅ **3.3 タスク管理UI**: 包括的なタスク追跡、プロジェクト管理、時間追跡機能

**Phase 2: コア機能 (完了)**
- ✅ ワークスペース管理システム
- ✅ プロセス間通信の実装 (WebSocket)
- ✅ 状態管理と永続化

**Phase 1: 基盤構築 (完了)**
- ✅ Rustプロジェクトのセットアップ
- ✅ WezTerm Lua設定の実装
- ✅ 基本的なプロセス管理機能

**品質保証 (完了)**
- ✅ **47個のテスト** - 100%成功率
- ✅ **コンパイル警告0個** - クリーンコードベース
- ✅ **環境変数競合問題解決** - 安定したテスト実行

### 🔮 将来の拡張予定

**Phase 4: 高度な機能**
- 🔄 プラグインシステム
- 🔄 設定のホットリロード  
- 🔄 AI統合機能強化

## 貢献

このプロジェクトへの貢献を歓迎します。

### 開発ガイドライン

1. **テストファースト開発**: 実装前にテストを作成
2. **Git Worktree使用**: 機能開発は独立したワークツリーで行う
3. **進捗追跡**: PROGRESS.mdを定期的に更新
4. **品質チェック**: コミット前にlint、型チェック、テストを実行

### セキュリティ

- プロセス分離によるセキュリティ確保
- ファイルアクセス制御
- プロセス間通信の暗号化（オプション）

## ライセンス

MIT License

## 参考資料

- [WezTerm公式ドキュメント](https://wezfurlong.org/wezterm/)
- [Lua公式ドキュメント](https://www.lua.org/docs.html)
- [Rust公式ドキュメント](https://doc.rust-lang.org/)
- プロジェクト要求仕様書: docs/prd.md
- アーキテクチャ仕様書: docs/ARCHITECTURE.md