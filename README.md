# WezTerm マルチプロセス開発補助ツール

**WezTermでClaude Codeを複数プロセス実行するためのマルチプロセス管理ツール**

個人開発者がClaude Codeを効率的に使うためのマルチプロセス管理フレームワークです。

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
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

### 前提条件

- WezTerm
- Rust 1.70+
- Git

### インストール

```bash
# リポジトリのクローン
git clone https://github.com/daktu32/wezterm-parallel.git
cd wezterm-parallel

# ビルド
cargo build --release

# 起動
cargo run
```

## 🚀 基本的な使い方

### 1. フレームワーク起動

```bash
cargo run
```

### 2. 基本テスト

```bash
# Ping テスト
echo '{"Ping": null}' | nc -U /tmp/wezterm-parallel.sock

# ワークスペース作成
echo '{"WorkspaceCreate":{"name":"test","template":"default"}}' | nc -U /tmp/wezterm-parallel.sock

# タスク作成
echo '{"TaskQueue":{"id":"task-1","priority":5,"command":"テストタスク"}}' | nc -U /tmp/wezterm-parallel.sock
```

### 3. WebSocketダッシュボード

ブラウザで `ws://localhost:9999` に接続（メトリクス、タスク管理、カンバンボード）

## 📊 プロジェクト構造

```
wezterm-parallel/
├── src/                    # Rust コード
│   ├── workspace/          # ワークスペース管理
│   ├── process/            # プロセス管理
│   ├── task/               # タスク管理システム
│   ├── monitoring/         # 監視・メトリクス機能
│   ├── dashboard/          # WebSocketダッシュボード
│   └── main.rs             # エントリポイント
├── lua/                    # WezTerm Lua設定
├── config/                 # 設定テンプレート
└── tests/                  # ライブラリテスト
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

## 🤝 貢献

個人利用での機能改善や拡張に興味がある方は歓迎します：

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

**品質について**: 堅牢なテスト基盤により基本機能の品質を確認しています。詳細は[docs/TESTING.md](docs/TESTING.md)を参照してください。
