# WezTerm マルチプロセス開発補助ツール

**WezTermでClaude Codeを複数プロセス実行するための実験的なツール**

個人開発者がClaude Codeを効率的に使うための小さなヘルパーツールです。

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![Status](https://img.shields.io/badge/status-experimental-yellow)

## 🚧 注意事項

- **実験的なツール**です
- **個人開発・学習用途**に限定
- **本格運用には不向き**
- **エラーハンドリングが不完全**

## ✨ 実装されている機能

### 基本機能
- 複数Claude Codeプロセスの起動・停止
- ワークスペース単位でのプロセス整理
- Unix Socket経由のIPC通信
- 基本的なプロセス監視

### 実験的機能
- シンプルなタスク管理
- 基本的な時間追跡
- WebSocketダッシュボード（簡素）
- コンソール監視

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

### 3. WebSocketダッシュボード（実験的）

ブラウザで `ws://localhost:9999` に接続（簡素なWebSocket API）

## 📊 プロジェクト構造

```
wezterm-parallel/
├── src/                    # Rust コード
│   ├── workspace/          # ワークスペース管理
│   ├── process/            # プロセス管理
│   ├── task/               # タスク管理
│   ├── monitoring/         # 監視機能
│   └── main.rs             # エントリポイント
├── lua/                    # WezTerm Lua設定（実験的）
├── config/                 # 設定テンプレート
└── tests/                  # テスト
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

## ⚠️ 制限事項

- **セキュリティ対策は最小限**
- **エラーハンドリングが不完全**
- **パフォーマンス最適化未実装**
- **本格的なUI/UXなし**
- **ドキュメントが不十分**

## 🎯 想定用途

- Claude Code学習・実験
- 個人開発での効率化実験
- マルチプロセス開発の体験
- Rust/WezTerm学習

## 🤝 貢献

小さな改善や修正は歓迎します：

1. バグ報告
2. 簡単な機能改善
3. ドキュメント修正
4. テスト追加

## 📝 ライセンス

MIT License

## 🔗 参考資料

- [WezTerm公式ドキュメント](https://wezfurlong.org/wezterm/)
- [Rust公式ドキュメント](https://doc.rust-lang.org/)

---

**免責事項**: このツールは実験的なものです。重要なプロジェクトでの使用前に十分にテストしてください。