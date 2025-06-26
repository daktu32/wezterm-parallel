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

2. **バックエンドレイヤー (Rust)**
   - Process Manager: Claude Codeプロセスの管理
   - Communication Hub: プロセス間通信の仲介
   - State Management: アプリケーション状態の永続化

### 技術スタック
- **フロントエンド**: WezTerm、Lua (3,239行実装済み)
- **バックエンド**: Rust (6,734行実装済み)
- **IPC**: Unix Domain Socket
- **設定**: YAML/TOML
- **状態管理**: JSON/YAML、SQLite（オプション）

## 実装状況

### ✅ Phase 1: 基盤構築 (完了)
- Rustプロジェクトのセットアップ (6,734行)
- 完全なワークスペース管理システム (CRUD操作、テンプレート、永続化)
- プロセス管理・監視・再起動機能
- メトリクス収集・保存システム
- YAML設定管理・ホットリロード基盤
- Unix Domain Socket IPC通信システム
- 包括的テストスイート (47個のテスト、全て通過)

### 🔄 Phase 2: UI/UX機能 (準備完了)
- WezTerm Lua統合 (基盤実装済み)
- WebSocketダッシュボード (設計完了)
- ペイン管理システム (設計完了)

### 📋 Phase 3: 高度機能 (計画中)
- タスク管理・カンバンボード
- プラグインシステム
- 運用監視・分析機能

## 開発ワークフロー

### 開発プラクティス
1. **テストファースト開発**: 実装前にテストを作成する (現在47個実装済み)
2. **進捗追跡**: PROGRESS.mdとDEVELOPMENT_ROADMAP.mdを更新する
3. **品質チェック**: コミット前にlint、型チェック、テストを通す
4. **GitHub Issues**: [Issue #8-16](https://github.com/daktu32/wezterm-parallel/issues) で進捗管理

### ブランチ戦略
- `main`: 本番環境相当（Phase 1完了）
- `develop`: 開発統合ブランチ
- `feature/*`: 機能開発ブランチ
- `hotfix/*`: 緊急修正ブランチ

## コマンド

### 開発用コマンド（実装済み）
```bash
# Rustプロジェクトのビルド
cargo build

# 全テストの実行 (47個のテスト)
cargo test

# フレームワークの起動
cargo run

# リリースビルド
cargo build --release

# ドキュメント生成
cargo doc --open
```

### WezTerm設定関連（Phase 2で実装予定）
```bash
# WezTerm設定のリロード
# Ctrl+Shift+R (設定内で定義予定)

# 新規ワークスペース作成
# Ctrl+Shift+N (設定内で定義予定)

# ワークスペース切り替え
# Ctrl+Shift+W (設定内で定義予定)
```

## ディレクトリ構造（実装済み）

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

## 開発進捗管理

### GitHub Issues（実装済み）
- [Issue #8](https://github.com/daktu32/wezterm-parallel/issues/8): ProcessManager-WorkspaceManager統合
- [Issue #9](https://github.com/daktu32/wezterm-parallel/issues/9): WezTerm Lua統合実装
- [Issue #11](https://github.com/daktu32/wezterm-parallel/issues/11): WebSocketダッシュボード
- [Issue #12](https://github.com/daktu32/wezterm-parallel/issues/12): ペイン管理システム

### 次のマイルストーン
1. **Issue #8**: ProcessManager-WorkspaceManager 統合強化
2. **Issue #9**: 基本的なWezTerm Lua統合
3. **Issue #11**: WebSocketダッシュボード実装

## パフォーマンス目標

- ワークスペース起動時間: < 3秒
- ペイン操作レスポンス: < 100ms
- メモリ使用量: ベースライン < 50MB
- CPU使用率（アイドル時）: < 1%

## セキュリティ考慮事項

- プロセス分離によるセキュリティ確保
- ファイルアクセス制御
- プロセス間通信の暗号化（オプション）

## 進捗管理ルール

### 必須ファイル更新
AIエージェントは以下のファイルを最新に保つ必要があります：

1. **PROGRESS.md** - 開発進捗の追跡
   - 各タスク完了後に更新
   - 完了したタスク、現在の作業、次のタスクを文書化
   - 日付とタイムスタンプを含める

2. **DEVELOPMENT_ROADMAP.md** - 開発ロードマップ
   - フェーズの進行に応じて更新
   - 完了したマイルストーンにチェックマークを付ける
   - 新しい課題や変更を反映

### 更新タイミング
- 機能実装完了時
- 重要な設定変更後
- フェーズ移行時
- バグ修正や改善後
- 新しい技術的決定時

### 更新方法
1. 作業完了直後に該当ファイルを更新
2. 具体的な成果物と変更を文書化
3. 次のステップを明確化
4. コミットメッセージに進捗更新を含める

### 実装後チェックリスト
- [ ] すべてのテストが通過 (現在47個実装済み)
- [ ] Rustコンパイルが成功
- [ ] `cargo clippy` が通過
- [ ] ドキュメントが更新済み
- [ ] PROGRESS.mdが完了したタスクと次のタスクで更新済み
- [ ] DEVELOPMENT_ROADMAP.mdに進捗反映済み

## プロジェクト固有の開発ルール

### Gitワークフロー

#### ブランチ戦略
- **メインブランチ**: `main` (Phase 1完了)
- **機能ブランチ**: `feature/task-description`
- **バグ修正ブランチ**: `fix/bug-description`

#### 必須作業手順
すべての開発作業で以下の手順に従ってください：

1. 機能要件を定義し、GitHub Issueで管理
2. **作業ブランチを作成し、git worktreeで分離**
3. 期待される入力と出力に基づいてテストを作成
4. テストを実行し、失敗を確認
5. テストを通過するコードを実装
6. すべてのテストが通過したらリファクタリング
7. 進捗ファイル（PROGRESS.md、DEVELOPMENT_ROADMAP.md）を更新

### 品質チェックリスト
実装完了前に以下を確認：
- `cargo build` (Rustコンパイル)
- `cargo test` (全テスト実行)
- `cargo clippy` (リンティング)
- ドキュメントの更新

### 禁止される実践

以下の実践は厳禁です：
- テストなしでの機能実装
- メインブランチでの直接作業
- シークレットや認証情報のハードコーディング
- 既存テストの削除・破壊的変更
- 承認なしでの外部依存関係の追加
- ドキュメント更新のスキップ
- PROGRESS.mdとDEVELOPMENT_ROADMAP.md更新の無視

## 参考資料

### プロジェクト文書
- [開発ロードマップ](DEVELOPMENT_ROADMAP.md) - 段階的開発計画
- [プロジェクト要求仕様書](docs/prd.md) - 機能要求定義
- [アーキテクチャ仕様書](docs/ARCHITECTURE.md) - 技術設計
- [進捗レポート](PROGRESS.md) - 開発進捗状況

### 外部リソース
- [WezTerm公式ドキュメント](https://wezfurlong.org/wezterm/)
- [Lua公式ドキュメント](https://www.lua.org/docs.html)
- [Rust公式ドキュメント](https://doc.rust-lang.org/)
- [GitHub Issues](https://github.com/daktu32/wezterm-parallel/issues) - 開発進捗

## 重要な実装ガイドライン
- 新機能は必ずテストファーストで開発
- 既存の47個のテストを維持・拡張
- Unix Domain Socket IPCインターフェースを維持
- 段階的開発アプローチを遵守（Phase 1完了、Phase 2実装中）