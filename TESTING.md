# テストガイド

## 概要

WezTerm Multi-Process Development Frameworkのテストガイドです。本番環境に影響しない安全なテスト方法と、各コンポーネントのテスト手順を提供します。

## 🧪 安全なテスト環境

### テスト環境の特徴

- ✅ **本番WezTerm設定を自動バックアップ・復元**
- ✅ **独立したテスト用ソケット** (`/tmp/wezterm-parallel-test.sock`)
- ✅ **テスト専用の設定ファイル** (`test_wezterm.lua`)
- ✅ **自動クリーンアップ機能**

### クイックテスト手順

```bash
# 1. テスト環境セットアップ
./test_runner.sh setup

# 2. 新しいWezTermウィンドウを開く（タブタイトルに[TEST]が表示）

# 3. テスト用キーバインドで動作確認
# Ctrl+Shift+D - ダッシュボード切り替え
# Ctrl+Shift+R - ダッシュボード更新
# Ctrl+Shift+T - バックエンド接続テスト
# Ctrl+Shift+H - ヘルプ表示

# 4. テスト環境クリーンアップ
./test_runner.sh cleanup
```

### テストランナーコマンド

```bash
# 環境セットアップ
./test_runner.sh setup

# 現在の状態確認
./test_runner.sh status

# 環境クリーンアップ
./test_runner.sh cleanup

# ヘルプ表示
./test_runner.sh help
```

## 🔬 ユニットテスト

### Rustライブラリテスト

```bash
# 全テスト実行
cargo test

# 特定モジュールのテスト
cargo test workspace
cargo test process
cargo test dashboard

# テスト詳細表示
cargo test -- --nocapture

# 並行実行数制限
cargo test -- --test-threads=1
```

### 現在のテスト状況

- **総テスト数**: 127個
- **テスト結果**: 全て通過 ✅
- **カバレッジ**: コア機能85%以上

#### テストカテゴリ

**ワークスペース管理 (42テスト)**
- ワークスペース作成・削除
- プロセス配置・管理
- 状態同期・永続化

**プロセス管理 (35テスト)**
- プロセス起動・停止
- ヘルスチェック・監視
- リソース制限・エラーハンドリング

**IPC通信 (28テスト)**
- Unix Domain Socket通信
- メッセージシリアライゼーション
- 接続管理・エラー処理

**ダッシュボード (22テスト)**
- WebSocket接続・切断
- リアルタイムデータ更新
- UI状態管理

## 🔧 統合テスト

### WebSocketダッシュボードテスト

```bash
# WebSocket接続テスト
python3 -c "
import socket
sock = socket.socket()
result = sock.connect_ex(('127.0.0.1', 9999))
print('接続成功' if result == 0 else '接続失敗')
sock.close()
"

# ダッシュボード機能テスト（WezTerm内）
# Ctrl+Shift+D でダッシュボード表示
# リアルタイム更新の確認
# プロセス一覧・ワークスペース状態の表示確認
```

### IPC通信テスト

```bash
# Unix Socket接続テスト
ls -la /tmp/wezterm-parallel.sock

# 基本通信テスト
echo '{"WorkspaceList":{}}' | nc -U /tmp/wezterm-parallel.sock
```

## 📊 パフォーマンステスト

### 起動時間測定

```bash
# フレームワーク起動時間
time cargo run -- --help

# 期待値: 100ms以下
```

### メモリ使用量確認

```bash
# プロセス監視
ps aux | grep wezterm-parallel

# メモリ使用量（期待値: 50MB以下）
```

### レスポンス時間測定

```bash
# WebSocket応答時間
time python3 -c "
import socket, time
start = time.time()
sock = socket.socket()
sock.connect(('127.0.0.1', 9999))
sock.close()
print(f'応答時間: {(time.time() - start) * 1000:.1f}ms')
"
```

## 🔍 デバッグ・トラブルシューティング

### ログレベル設定

```bash
# デバッグログ有効化
RUST_LOG=debug cargo run

# 特定モジュールのログ
RUST_LOG=wezterm_parallel::dashboard=debug cargo run
```

### 一般的な問題

**ダッシュボード接続失敗**
```bash
# ポート使用状況確認
netstat -an | grep 9999

# プロセス状況確認
ps aux | grep wezterm-parallel

# ログ確認
tail -f logs/wezterm-parallel.json
```

**WezTerm設定問題**
```bash
# 設定構文チェック
wezterm --config-file test_wezterm.lua show-keys

# 設定リロード
# WezTerm内で Ctrl+Shift+R
```

**バックエンド通信問題**
```bash
# ソケットファイル確認
ls -la /tmp/wezterm-parallel*.sock

# 権限確認
chmod 755 /tmp/wezterm-parallel.sock
```

## 📋 テストチェックリスト

### 基本機能テスト

- [ ] バックエンドサービス起動
- [ ] WebSocketダッシュボード接続
- [ ] WezTerm統合（キーバインド）
- [ ] ワークスペース作成・切り替え
- [ ] プロセス管理（起動・停止）

### ダッシュボード機能テスト

- [ ] リアルタイムデータ更新
- [ ] システムヘルス表示
- [ ] プロセス一覧表示
- [ ] ワークスペース状態表示
- [ ] アラート表示

### WezTerm統合テスト

- [ ] キーバインド動作
- [ ] ペイン分割・管理
- [ ] タブ管理
- [ ] レイアウトテンプレート

### エラーハンドリングテスト

- [ ] 無効な入力に対する適切な応答
- [ ] ネットワーク切断時の復旧
- [ ] リソース不足時の処理
- [ ] 設定エラー時の表示

## 🚀 CI/CD テスト

### GitHub Actions

現在実装中のワークフロー：
- 自動ビルドテスト
- ユニットテスト実行
- 設定ファイル検証
- ドキュメント更新確認

### ローカル事前チェック

```bash
# コミット前チェック
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

## 📚 テストデータ

### モックデータ

テスト環境では以下のモックデータを使用：

**ワークスペース**
- default (active, 2 processes)
- frontend (inactive, 1 process)
- backend (inactive, 3 processes)

**プロセス**
- claude-1 (Running, 25% CPU, 128MB)
- claude-2 (Idle, 5% CPU, 64MB)
- claude-3 (Busy, 60% CPU, 256MB)

**システムヘルス**
- 総プロセス数: 3
- 応答プロセス数: 3
- 平均CPU使用率: 30%
- 総メモリ使用量: 448MB

## ⚠️ 注意事項

### テスト実行時の注意

1. **必ずテスト環境を使用** - 本番設定への影響を避ける
2. **テスト後のクリーンアップ** - `./test_runner.sh cleanup` を必ず実行
3. **並行テスト実行の制限** - ポート競合を避けるため順次実行
4. **ログファイルの確認** - 異常時はログを必ず確認

### 既知の制限事項

- テスト環境では一部の永続化機能が無効
- モックデータのため実際のパフォーマンスと異なる場合あり
- WebSocket接続数は10クライアントまで制限

## 📈 テスト結果の記録

テスト実行後は以下を記録：

```bash
# テスト結果例
Date: 2025-07-08
Environment: Test Environment v0.3.0
Backend Status: ✅ Running
Dashboard Status: ✅ Connected
WezTerm Integration: ✅ Working
Unit Tests: ✅ 127/127 passed
Performance: ✅ Startup < 100ms, Memory < 50MB
```

---

詳細な使用方法については [README.md](README.md) および [QUICKSTART.md](QUICKSTART.md) を参照してください。