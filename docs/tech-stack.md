# WezTerm マルチプロセス並行開発フレームワーク - Technology Stack

このドキュメントはプロジェクトの技術スタックを定義します。他のドキュメントはこれを技術選択の信頼できる情報源として参照します。

## フロントエンド技術

### UI Framework
- **Primary**: WezTerm
- **Version**: 最新安定版 (20240203-110809-5046fc22)
- **Rationale**: 高度にカスタマイズ可能なターミナルで、Luaスクリプトによる豊富な機能拡張が可能

### 設定・スクリプト言語
- **Primary**: Lua 5.4
- **Version**: ≥5.4.0
- **Configuration**: WezTerm組み込みLua環境
- **Rationale**: WezTermのネイティブスクリプト言語で、パフォーマンスが良く軽量

### ユーザーインターフェース
- **Framework**: WezTerm GUI API + カスタムLuaモジュール
- **Component**: ペイン管理、タブ管理、ステータス表示
- **Theming**: WezTerm Color Scheme API

## バックエンド技術

### Runtime Environment
- **Platform**: Rust
- **Version**: ≥1.70.0
- **Edition**: 2021
- **Rationale**: メモリ安全性、高パフォーマンス、並行処理に優秀

### Framework & Libraries
- **Async Runtime**: Tokio 1.x
- **Serialization**: Serde + serde_json
- **Process Management**: tokio::process
- **IPC**: Unix Domain Sockets (tokio::net::UnixListener)
- **Logging**: tracing + tracing-subscriber
- **Configuration**: serde + toml/yaml

### API Design
- **Style**: カスタムIPC プロトコル (JSON over Unix Socket)
- **Message Format**: JSON
- **Validation**: Serde derive macros

## データストレージ

### 設定ストレージ
- **Type**: ファイルベース (YAML/TOML)
- **Location**: `~/.config/wezterm-parallel/`
- **Rationale**: シンプルで人間が読みやすく、バージョン管理可能

### 状態管理
- **Primary**: JSON ファイル
- **Backup**: SQLite (オプション)
- **Location**: `~/.local/share/wezterm-parallel/`
- **Rationale**: 軽量で依存関係が少ない

### セッション永続化
- **Format**: JSON
- **Scope**: ワークスペース単位
- **Auto-save**: 30秒間隔

## プロセス間通信

### IPC Protocol
- **Transport**: Unix Domain Socket
- **Format**: JSON messages
- **Authentication**: ファイルシステム権限ベース
- **Rationale**: 高速、セキュア、クロスプラットフォーム対応

### Message Types
```rust
pub enum Message {
    WorkspaceCreate { name: String, template: String },
    ProcessSpawn { workspace: String, command: String },
    StatusUpdate { process_id: String, status: ProcessStatus },
    TaskQueue { id: String, priority: u8, command: String },
}
```

## 開発・ビルドツール

### コード品質
- **Linting**: Clippy (Rust), luacheck (Lua)
- **Formatting**: rustfmt, stylua
- **Type Checking**: Rust compiler (rustc)

### テスト
- **Unit Testing**: Rust built-in test framework
- **Integration Testing**: tokio-test
- **Lua Testing**: busted (オプション)
- **Performance Testing**: criterion.rs

### ドキュメント
- **Code Docs**: rustdoc
- **API Docs**: 自動生成 (cargo doc)
- **Project Docs**: Markdown

## セキュリティ

### プロセス分離
- **Method**: OS-level process isolation
- **Sandboxing**: ファイルアクセス制限
- **User permissions**: 実行ユーザーの権限内で動作

### 通信セキュリティ
- **IPC**: Unix socket file permissions (600)
- **Data validation**: Serde type safety
- **Access control**: プロセスID ベース認証

### データ保護
- **Configuration**: ファイルシステム権限 (600)
- **Secrets**: 環境変数または外部キーマネージャー
- **Logging**: 機密情報のフィルタリング

## 外部依存関係

### Claude Code Integration
- **Interface**: プロセス実行 + stdin/stdout
- **Communication**: 標準入出力
- **Process monitoring**: プロセスID追跡
- **Rationale**: Claude Codeの標準的な利用方法

### System Dependencies
- **Required**: WezTerm
- **Optional**: luacheck (開発時)
- **Platform**: macOS, Linux, Windows

## バージョン要件

| Technology | Minimum Version | Recommended Version | Notes |
|------------|----------------|-------------------|-------|
| Rust | 1.70.0 | Latest stable | async/await, const generics |
| WezTerm | 20240203 | Latest stable | Lua API compatibility |
| Lua | 5.4 | 5.4.x | WezTerm embedded |
| Claude Code | Latest | Latest | External dependency |

## 技術選択の根拠

### Why These Technologies?

1. **Rust**: メモリ安全性と高パフォーマンスを両立。並行プロセス管理に最適
2. **WezTerm + Lua**: 強力なカスタマイズ性とスクリプト機能。ターミナル環境の完全制御が可能
3. **Unix Domain Socket**: ローカルIPC通信で最高のパフォーマンスとセキュリティ
4. **ファイルベース設定**: シンプルで透明性が高く、バージョン管理やバックアップが容易

### 代替案検討

| Primary Choice | Alternative Considered | Why Not Chosen |
|----------------|----------------------|----------------|
| Rust | Go | ガベージコレクションによる予測不能な停止が懸念 |
| Unix Socket | TCP Socket | ローカル通信でオーバーヘッドが不要 |
| JSON | MessagePack | 可読性とデバッグのしやすさを優先 |
| WezTerm | Tmux + 外部UI | 統合されたUI/UX体験を実現するため |

## アーキテクチャ決定

### ADR-001: Rust for Backend
- **Status**: Accepted
- **Rationale**: システムレベルプログラミング、メモリ安全性、並行処理性能
- **Alternatives**: Go, C++
- **Decision**: Rustの採用により、安全で高性能なプロセス管理を実現

### ADR-002: Unix Domain Socket for IPC
- **Status**: Accepted  
- **Rationale**: ローカル通信の最適化、セキュリティ、クロスプラットフォーム対応
- **Alternatives**: TCP/HTTP, Named Pipes
- **Decision**: パフォーマンスとセキュリティの最適バランス

### ADR-003: WezTerm as Primary UI
- **Status**: Accepted
- **Rationale**: ターミナル環境での完全な制御、Luaスクリプトによる柔軟性
- **Alternatives**: 独自GUI、Web UI
- **Decision**: 開発者体験とカスタマイズ性を最大化

## 依存関係

### Critical Dependencies
- **WezTerm**: フロントエンドUI環境
- **Rust toolchain**: バックエンド開発・ビルド環境
- **Claude Code**: 統合対象の外部プロセス

### Optional Dependencies
- **luacheck**: Luaコードの静的解析
- **stylua**: Luaコードフォーマッター
- **criterion**: パフォーマンステスト

## 開発フェーズ別技術導入

### Phase 1: 基盤構築 (✅ 完了)
- ✅ Rust basic structure (6,734行実装済み)
- ✅ WezTerm Lua basic integration (3,239行準備済み)
- ✅ Simple IPC implementation (Unix Domain Socket実装済み)

### Phase 2: コア機能 (🔄 実装中)
- ✅ Full IPC protocol (実装完了)
- ✅ Process management (実装完了)
- ✅ Workspace management (実装完了)

### Phase 3: 高度機能 (📅 計画中)
- 📅 Performance optimization
- 📅 Advanced monitoring
- 📅 Plugin system

---

**Last Updated**: 2025-06-26  
**Reviewed By**: Claude Code Assistant  
**Next Review**: Phase 2完了時