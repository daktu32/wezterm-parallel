# CLAUDE.md

このファイルは、このリポジトリでのコード作業時にClaude Code (claude.ai/code) へのガイダンスを提供します。

## プロジェクト概要

**WezTerm マルチプロセス並行開発フレームワーク** - WezTermの機能を活用し、Claude Codeによるマルチプロセス並行開発環境を提供するフレームワークです。

### 主要機能
- WezTerm単体での完全なマルチプロセス開発環境
- Claude Codeプロセスの統合管理
- プロジェクト単位でのワークスペース管理
- セッション永続化とプロセス間通信

## アーキテクチャ概要

### コンポーネント構成
1. **フロントエンドレイヤー (WezTerm + Lua)**
   - WezTerm Terminal: ユーザーインターフェース
   - Lua Configuration: 設定管理とイベントハンドリング
   - Workspace Management: ワークスペースとペインの管理

2. **バックエンドレイヤー (Rust/Go)**
   - Process Manager: Claude Codeプロセスの管理
   - Communication Hub: プロセス間通信の仲介
   - State Management: アプリケーション状態の永続化

### 技術スタック
- **フロントエンド**: WezTerm、Lua
- **バックエンド**: Rust または Go
- **IPC**: Unix Domain Socket
- **設定**: YAML/TOML
- **状態管理**: JSON/YAML、SQLite（オプション）

## 開発ワークフロー

### 開発プラクティス
1. **Git Worktree使用**: 機能開発は独立したワークツリーで行う
2. **進捗追跡**: PROGRESS.mdとDEVELOPMENT_ROADMAP.mdを更新する
3. **テストファースト開発**: 実装前にテストを作成する
4. **品質チェック**: コミット前にlint、型チェック、テストを通す

### ブランチ戦略
- `main`: 本番環境相当
- `develop`: 開発統合ブランチ
- `feature/*`: 機能開発ブランチ
- `hotfix/*`: 緊急修正ブランチ

## コマンド

### 開発用コマンド（実装予定）
```bash
# Rustプロジェクトのビルド
cargo build

# テストの実行
cargo test

# Luaスクリプトのチェック
luacheck wezterm-config/

# ドキュメント生成
cargo doc --open
```

### WezTerm設定関連
```bash
# WezTerm設定のリロード
# Ctrl+Shift+R (設定内で定義予定)

# 新規ワークスペース作成
# Ctrl+Shift+N (設定内で定義予定)

# ワークスペース切り替え
# Ctrl+Shift+W (設定内で定義予定)
```

## ディレクトリ構造（計画）

```
wezterm-parallel/
├── src/                    # Rustソースコード
│   ├── process_manager/    # プロセス管理モジュール
│   ├── communication/      # IPC通信モジュール
│   ├── state/             # 状態管理モジュール
│   └── main.rs            # エントリーポイント
├── wezterm-config/        # WezTerm Lua設定
│   ├── init.lua           # メイン設定ファイル
│   ├── workspace.lua      # ワークスペース管理
│   └── keybindings.lua    # キーバインド定義
├── tests/                 # テストコード
├── docs/                  # ドキュメント
└── config/               # 設定テンプレート
```

## 実装フェーズ

### Phase 1: 基盤構築
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

## パフォーマンス目標

- ワークスペース起動時間: < 3秒
- ペイン操作レスポンス: < 100ms
- メモリ使用量: ベースライン < 50MB
- CPU使用率（アイドル時）: < 1%

## セキュリティ考慮事項

- プロセス分離によるセキュリティ確保
- ファイルアクセス制御
- プロセス間通信の暗号化（オプション）

## 参考資料

- [WezTerm公式ドキュメント](https://wezfurlong.org/wezterm/)
- [Lua公式ドキュメント](https://www.lua.org/docs.html)
- [Rust公式ドキュメント](https://doc.rust-lang.org/)
- プロジェクト要求仕様書: docs/prd.md
- アーキテクチャ仕様書: docs/ARCHITECTURE.md