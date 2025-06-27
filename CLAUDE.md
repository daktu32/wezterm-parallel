# CLAUDE.md

このファイルは、このリポジトリでのコード作業時にClaude Code (claude.ai/code) へのガイダンスを提供します。

## プロジェクト概要

**WezTerm マルチプロセス開発補助ツール** - WezTermでClaude Codeを複数プロセス実行するための実験的なツールです。

### 主要機能
- 複数Claude Codeプロセスの管理
- ワークスペース単位でのプロセス整理
- 基本的なタスク管理
- シンプルな監視機能

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

### ✅ 実装済み機能
- 基本的なプロセス管理 (起動・停止・監視)
- ワークスペース作成・切り替え
- Unix Socket経由のIPC通信
- シンプルなタスク管理
- 基本的な時間追跡
- コンソール監視機能

### 🔄 実験的機能
- WebSocketダッシュボード (動作するが簡素)
- カンバンボード風UI (プロトタイプレベル)
- メトリクス収集 (基本的な情報のみ)

### ⚠️ 制限事項
- 個人開発・実験用途のみ
- 本格運用には不向き
- エラーハンドリングが不完全
- テストカバレッジが限定的

## 開発ワークフロー

### 開発プラクティス
1. **テストファースト開発**: 実装前にテストを作成する (現在47個実装済み)
2. **進捗追跡**: PROGRESS.mdとDEVELOPMENT_ROADMAP.mdを更新する
3. **品質チェック**: コミット前にlint、型チェック、テストを通す
4. **GitHub Issues**: [Issue #8-16](https://github.com/daktu32/wezterm-parallel/issues) で進捗管理

### ブランチ戦略
- `main`: 動作する最新バージョン
- `develop`: 開発中の機能
- `feature/*`: 新機能ブランチ
- `fix/*`: バグ修正ブランチ

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

## 想定される性能

- ワークスペース起動: 数秒
- 基本操作: レスポンシブ
- メモリ使用量: 軽量（プロセス数に依存）
- CPU使用率: 低負荷

## セキュリティ注意事項

- ローカル環境での使用を想定
- プロセス間通信は平文
- 本格的なセキュリティ対策は未実装

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
- [ ] 基本テストが通過
- [ ] Rustコンパイルが成功
- [ ] 基本動作確認
- [ ] ドキュメント更新
- [ ] 進捗記録更新

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

### 避けるべき実践

以下は避けましょう：
- 動作確認なしでの機能追加
- メインブランチでの直接作業
- 認証情報のハードコーディング
- 既存機能の破壊
- 重い外部依存の追加
- ドキュメント更新の怠慢

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

## 実装ガイドライン
- 基本的な動作テストを実行
- 既存の基本機能を維持
- Unix Domain Socket IPCを維持
- 簡素で理解しやすいコードを心がける