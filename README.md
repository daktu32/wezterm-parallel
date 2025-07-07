# WezTerm マルチプロセス開発補助ツール

**WezTermでClaude Codeを複数プロセス実行するためのマルチプロセス管理ツール**

個人開発者がClaude Codeを効率的に使うためのマルチプロセス管理フレームワークです。

![Version](https://img.shields.io/badge/version-0.3.0-blue)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![CI](https://github.com/daktu32/wezterm-parallel/workflows/CI%2FCD%20Pipeline/badge.svg)
![Tests](https://img.shields.io/badge/tests-251/251-green)
![License](https://img.shields.io/badge/license-MIT-blue)
![Status](https://img.shields.io/badge/status-stable-green)

## 🎆 特徴

- **基本機能が動作するシステム**です
- **個人利用専用**の生産性向上ツール
- **実用的な日常使用**に対応
- **基本的なエラーハンドリング**を実装

## ✨ 実装済み機能

### 🎯 MVP機能 (Issue #17 & #18)
- **Claude Code複数プロセス協調システム**: プロセス間通信・タスク分散・ファイル同期・成果物統合
- **WezTermペイン分割・レイアウトテンプレート**: YAMLテンプレート・動的レイアウト・4種実用テンプレート
- **協調開発基盤**: 負荷分散・依存関係管理・競合検出・自動マージ

### コア機能
- 複数Claude Codeプロセスの管理（起動・停止・監視・再起動）
- ワークスペース単位でのプロセス整理とテンプレート管理
- Unix Socket経由のIPC通信
- システム監視とメトリクス収集

### 高度機能
- カンバンボード式タスク管理システム
- 時間追跡と生産性分析
- WebSocketベースのダッシュボード
- WezTerm Lua統合とキーボードショートカット
- ログ・アラート・障害検知機能

## 🛠️ セットアップ

### 🚀 新規ユーザー向け

**5分で動作確認してみませんか？**

➡️ **[クイックスタートガイド](QUICKSTART.md)** - 最小設定で即座に体験

### 📖 本格利用向け

**カスタマイズしてガッツリ使いたい方は：**

➡️ **[詳細セットアップガイド](SETUP-GUIDE.md)** - 設定・カスタマイズ・高度機能

### ⚡ インストール概要

```bash
# 方法1: GitHubからビルド (推奨)
git clone https://github.com/daktu32/wezterm-parallel.git
cd wezterm-parallel
cargo build --release

# 方法2: Cargoから直接インストール
cargo install --git https://github.com/daktu32/wezterm-parallel
```

### 📋 必要なもの

| 必須 | オプション |
|------|------------|
| WezTerm 20240203+ | Claude Code |
| Rust 1.70+ | Git |
| 512MB+ RAM | 1GB+ RAM (推奨) |

## 🚀 使い方概要

### 🎯 3ステップで開始

1. **フレームワーク起動**: `wezterm-parallel`
2. **ワークスペース作成**: API または WezTerm キーバインド
3. **ダッシュボード確認**: `http://localhost:8081`

**詳しい手順は**:
- 📚 [クイックスタートガイド](QUICKSTART.md) - 5分でお試し
- 📖 [詳細セットアップガイド](SETUP-GUIDE.md) - 本格利用

### 💡 主な操作方法

| 操作 | WezTermキーバインド | WebAPI | ダッシュボード |
|------|-------------------|--------|---------------|
| ワークスペース作成 | `Ctrl+Shift+N` | `POST /api/workspaces` | ✅ |
| ワークスペース切替 | `Ctrl+Shift+W` | `GET /api/workspaces` | ✅ |
| プロセス管理 | `Ctrl+Alt+P` | `POST /api/processes` | ✅ |
| ダッシュボード表示 | `Ctrl+Shift+D` | `http://localhost:8081` | - |

## 📚 ドキュメント

### 🚀 はじめに
- **[クイックスタートガイド](QUICKSTART.md)** - 5分で動作確認
- **[詳細セットアップガイド](SETUP-GUIDE.md)** - 本格設定・カスタマイズ

### 📖 利用ガイド
- **[ユーザーガイド](docs/USER-GUIDE.md)** - 実用例・ベストプラクティス
- **[API Documentation](https://daktu32.github.io/wezterm-parallel/)** - プログラム的操作
- **[FAQ](docs/FAQ.md)** - よくある質問

### 🔧 開発・保守
- **[開発ロードマップ](DEVELOPMENT_ROADMAP.md)** - 機能計画・進捗
- **[CI/CDガイド](docs/CI-CD.md)** - 品質保証・リリースプロセス
- **[コントリビューションガイド](docs/CONTRIBUTING.md)** - 開発参加方法

## 📊 プロジェクト構造

```
wezterm-parallel/
├── src/                    # Rust コア実装 (19,335行)
│   ├── room/               # ワークスペース (Room) 管理
│   ├── process/            # プロセス管理・Claude Code統合
│   ├── task/               # タスク管理・分散システム
│   ├── sync/               # ファイル同期・競合解決
│   ├── dashboard/          # WebSocketダッシュボード
│   ├── logging/            # 統一ログシステム
│   └── main.rs             # エントリポイント
├── lua/                    # WezTerm統合 (7,175行)
│   ├── room/               # Room管理Lua統合
│   ├── ui/                 # ダッシュボード・ペイン管理
│   └── config/             # 設定・キーバインド
├── .github/workflows/      # CI/CD自動化
├── docs/                   # ドキュメント
├── config/templates/       # 設定テンプレート
└── tests/                  # 251個のテスト
```

## 🧪 開発コマンド

```bash
# ビルド
cargo build

# テスト実行
cargo test

# 起動
cargo run

# リリースビルド
cargo build --release
```

## 📊 システム情報

- **実装規模**: 19,335行 (Rust) + 7,175行 (Lua) = 約26,500行
- **テストカバレッジ**: 堅牢なテスト基盤。詳細は[docs/TESTING.md](docs/TESTING.md)を参照
- **パフォーマンス**: 起動 < 2秒、メモリ < 30MB
- **安定性**: 日常的な使用に適している
- **ドキュメント**: 基本的な実装ガイドとコマンドリファレンス

## 🎯 想定用途

- **個人のClaude Codeマルチプロセス管理**
- **個人の開発ワークフロー最適化**
- **個人のタスク管理と時間追跡**
- **個人のWezTerm開発環境の整備**

## 📚 詳細情報

### アーキテクチャ・設計
- **🏗️ システム設計**: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - コンポーネント設計・アーキテクチャ詳細
- **🔧 API仕様**: [docs/API.md](docs/API.md) - IPC・WebSocket・Lua API の完全仕様
- **⚙️ 機能仕様**: [docs/FEATURE-SPEC.md](docs/FEATURE-SPEC.md) - 全機能の詳細仕様・実装状況

### 開発・貢献
- **🤝 貢献ガイド**: [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) - 開発環境構築・貢献方法
- **🧪 テスト詳細**: [docs/TESTING.md](docs/TESTING.md) - テスト戦略・実行方法
- **🔒 セキュリティ**: [docs/SECURITY.md](docs/SECURITY.md) - セキュリティポリシー・脆弱性報告

### プロジェクト管理
- **📋 ドキュメント体系**: [docs/DOCUMENTATION-MAP.md](docs/DOCUMENTATION-MAP.md) - 全ドキュメントの役割・関係
- **⚡ 技術スタック**: [docs/tech-stack.md](docs/tech-stack.md) - 採用技術・バージョン情報
- **🎯 プロジェクト要求仕様**: [docs/prd.md](docs/prd.md) - プロダクト要求・制約事項

## 🤝 貢献

個人利用での機能改善や拡張に興味がある方は歓迎します。詳細な貢献方法は[docs/CONTRIBUTING.md](docs/CONTRIBUTING.md)を参照してください：

1. 新機能の提案と実装
2. パフォーマンス最適化
3. UI/UXの改善
4. コミュニティプラグインの開発

## 📝 ライセンス

MIT License

## 🔗 参考資料

- [WezTerm公式ドキュメント](https://wezfurlong.org/wezterm/)
- [Rust公式ドキュメント](https://doc.rust-lang.org/)

---

## 関連ドキュメント

- **📋 ドキュメント体系**: [docs/DOCUMENTATION-MAP.md](docs/DOCUMENTATION-MAP.md) - 全ドキュメントの概要・役割
- **🏗️ アーキテクチャ**: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - システム設計詳細
- **🔧 API仕様**: [docs/API.md](docs/API.md) - API完全リファレンス
- **🤝 貢献ガイド**: [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) - 開発・貢献方法

---

**Last Updated**: 2025-07-03  
**Version**: v0.1.0  
**Quality**: 堅牢なテスト基盤により基本機能の品質を確認しています。詳細は[docs/TESTING.md](docs/TESTING.md)を参照してください。
