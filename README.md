# WezTerm マルチプロセス並行開発フレームワーク

WezTermの機能を活用し、Claude Codeによるマルチプロセス並行開発環境を提供するフレームワークです。

## 概要

このフレームワークは、WezTerm単体で完全なマルチプロセス開発環境を実現し、Claude Codeプロセスの統合管理、プロジェクト単位でのワークスペース管理、セッション永続化とプロセス間通信を提供します。

### 主要機能

- **ワークスペース管理**: プロジェクト単位でのワークスペース自動生成・切り替え
- **Claude Codeプロセス管理**: 複数のClaude Codeインスタンスの自動起動・監視
- **タスク管理**: 並行実行タスクのキューイングと優先度制御
- **プロセス間通信**: Unix Domain Socketによる効率的な通信
- **状態永続化**: ワークスペース状態の保存と復元

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

## 開発フェーズ

### Phase 1: 基盤構築 (進行中)
- [ ] Rustプロジェクトのセットアップ
- [ ] 基本的なプロセス管理機能
- [ ] WezTerm Lua設定の基礎実装

### Phase 2: コア機能
- [ ] ワークスペース管理システム
- [ ] Claude Codeプロセスの自動起動
- [ ] プロセス間通信の実装

### Phase 3: UI/UX機能
- [ ] ペイン管理機能
- [ ] ダッシュボード表示
- [ ] キーボードショートカット

### Phase 4: 高度な機能
- [ ] プラグインシステム
- [ ] 設定のホットリロード
- [ ] パフォーマンス最適化

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