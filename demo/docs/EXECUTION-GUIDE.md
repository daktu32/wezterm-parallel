# 🎬 デモ実行手順書

**Claude Code 並列開発デモ詳細実行ガイド**

このガイドでは、WezTerm Parallel を使った Claude Code 並列開発デモの詳細な実行手順を説明します。

## 📋 前提条件

### システム要件
- **OS**: macOS (推奨) または Linux
- **メモリ**: 8GB 以上推奨
- **ストレージ**: 2GB 以上の空き容量
- **ネットワーク**: インターネット接続 (Claude API アクセス用)

### 必要なツール
```bash
# 必須ツール確認
curl --version    # HTTP リクエスト
jq --version      # JSON 処理
git --version     # バージョン管理

# WezTerm インストール確認
wezterm --version

# Rust ツールチェーン確認
cargo --version
rustc --version
```

### 事前ビルド
```bash
# プロジェクトルートで実行
cd /Users/aiq/work/wezterm-parallel
cargo build --release
```

## 🚀 ステップバイステップ実行

### Step 1: 環境確認・セットアップ

```bash
# 1. プロジェクトディレクトリに移動
cd /Users/aiq/work/wezterm-parallel/demo

# 2. スクリプトに実行権限付与
chmod +x scripts/*.sh

# 3. 前提条件の最終確認
./scripts/setup.sh --check-only
```

**期待される出力:**
```
✅ wezterm-parallel バイナリ確認済み
✅ 必要ツール確認済み (curl, jq)
✅ ポート 8080, 8081 が利用可能
🚀 セットアップ準備完了
```

### Step 2: フレームワーク起動・ワークスペース作成

```bash
# 1. デモ環境セットアップ実行
./scripts/setup.sh
```

**期待される出力:**
```
🚀 WezTerm Parallel デモセットアップ開始
🧹 既存プロセスを停止中...
🚀 WezTerm Parallel フレームワーク起動中...
フレームワーク PID: 12345
⏳ フレームワーク起動待機中...
✅ フレームワーク起動完了
🧹 成果物ディレクトリクリーンアップ中...
🏗️ デモ用ワークスペース作成中...
{
  "workspace_id": "todo-app-demo",
  "status": "created",
  "processes": 4
}
✅ セットアップ完了！
```

### Step 3: デモシナリオ実行

```bash
# 1. デモ開始
./scripts/run-demo.sh
```

**実行フロー:**
1. **人間の要求入力**: "シンプルなToDoアプリを作って"
2. **ディレクター分析**: 要求分析・アーキテクチャ設計
3. **タスク分散**: 3つのエンジニアへのタスク割り当て
4. **並行開発開始**: React、Node.js、テスト実装の並行実行

**期待される出力:**
```
🎬 Claude Code 並列開発デモ開始
👤 人間からの要求: 「シンプルなToDoアプリを作って」
📤 ディレクターに要求を送信中...
📥 ディレクターからの応答:
{
  "analysis": "ToDoアプリ開発を開始します",
  "architecture": "React + Node.js + SQLite",
  "estimated_time": "2-3 hours"
}
🎯 Phase 1: ディレクターによる要求分析・タスク分解
⚡ Phase 2: タスク分散・並行開発開始
📤 タスク分散中...
🎬 デモが開始されました！
```

### Step 4: 並行開発監視

```bash
# 1. 並行開発監視開始
./scripts/coordinate.sh
```

**監視オプション:**
1. **ダッシュボード表示** (推奨): ブラウザで http://localhost:8081
2. **ターミナル監視**: リアルタイム更新表示
3. **状態確認**: 1回だけのスナップショット

**ダッシュボード表示内容:**
- 各プロセスの CPU/メモリ使用率
- タスクの進行状況（進行中/完了）
- リアルタイムログ
- プロセス間通信状況

### Step 5: 統合・報告

```bash
# 1. 統合・報告フェーズ実行
./scripts/integrate.sh
```

**統合プロセス:**
1. **成果物収集**: 各エンジニアの実装結果
2. **統合テスト**: フロントエンド・バックエンド連携確認
3. **品質評価**: コード品質・テストカバレッジ確認
4. **最終報告**: 完成度・課題の人間への報告

**期待される最終出力:**
```
🔄 統合・報告フェーズ開始
📊 開発完了状況確認中...
📈 進捗サマリー:
  総タスク数: 9
  完了: 7
  進行中: 2
  進捗率: 77%
🎊 統合・報告完了！
==================== 最終報告 ====================
✅ プロジェクト状態: 概ね完成
📋 成果サマリー:
  - ToDoアプリプロジェクト
  - 並行開発プロセス: 4つ
  - 技術スタック: React + TypeScript, Node.js + Express, SQLite
  - 進捗率: 77% (7/9 タスク完了)
🎭 Claude Code 並列開発デモが完了しました！
```

## 📊 結果の確認方法

### 1. ダッシュボードでの確認
```bash
# ブラウザで開く
open http://localhost:8081
```

### 2. API での状態確認
```bash
# ワークスペース状態
curl -s http://localhost:8080/api/workspaces/todo-app-demo/status | jq "."

# プロセス状態
curl -s http://localhost:8080/api/processes/status | jq "."

# タスク状態
curl -s http://localhost:8080/api/workspaces/todo-app-demo/tasks | jq "."
```

### 3. 成果物の確認
```bash
# 成果物ディレクトリ
ls -la artifacts/final/

# プロジェクトレポート
cat artifacts/final/project-report.json | jq "."

# メトリクス
cat artifacts/final/metrics.json | jq "."
```

## 🛠️ トラブルシューティング

### 一般的な問題と解決方法

#### 1. フレームワーク起動失敗
```bash
# 問題: ポート 8080 が使用中
# 解決: 既存プロセスを停止
sudo lsof -i :8080
kill -9 <PID>

# 問題: wezterm-parallel バイナリが見つからない
# 解決: ビルドの実行
cd /Users/aiq/work/wezterm-parallel
cargo build --release
export PATH="$PATH:$(pwd)/target/release"
```

#### 2. ワークスペース作成失敗
```bash
# API レスポンスエラーの確認
curl -v http://localhost:8080/api/health

# ログ確認
tail -f logs/wezterm-parallel.log
```

#### 3. プロセス間通信エラー
```bash
# Unix socket 確認
ls -la /tmp/wezterm-parallel-*.sock

# プロセス状態確認
curl -s http://localhost:8080/api/processes/status
```

#### 4. タスク実行停止
```bash
# 個別プロセスの再起動
curl -X POST http://localhost:8080/api/processes/frontend/restart

# ワークスペース全体の再起動
curl -X POST http://localhost:8080/api/workspaces/todo-app-demo/restart
```

### ログ確認

```bash
# システムログ
tail -f logs/system.log

# プロセス固有ログ
tail -f logs/process-director.log
tail -f logs/process-frontend.log
tail -f logs/process-backend.log
tail -f logs/process-tester.log
```

## 🧹 クリーンアップ

### デモ終了後のクリーンアップ

```bash
# 1. ワークスペース削除
curl -X DELETE http://localhost:8080/api/workspaces/todo-app-demo

# 2. フレームワーク停止
pkill wezterm-parallel

# 3. 一時ファイル削除
rm -rf /tmp/demo/
rm -rf artifacts/

# 4. ログファイル削除（オプション）
rm -rf logs/demo-*
```

## 📈 成功指標

このデモが成功した場合、以下が確認できます：

### 技術的成功指標
- ✅ 4つの Claude プロセスが正常に起動・通信
- ✅ タスクが適切に分散・実行
- ✅ リアルタイム監視ダッシュボードが動作
- ✅ 成果物（React アプリ、Node.js API、テスト）が生成
- ✅ 統合テストが実行・結果報告

### 実用的成功指標
- ✅ 人間の要求が自動的に分析・分解
- ✅ 並行開発による開発時間短縮効果
- ✅ プロセス間協調による品質向上
- ✅ 進捗・品質の透明性確保

## 📞 サポート・FAQ

### よくある質問

**Q: デモ実行にどのくらい時間がかかりますか？**
A: セットアップ 5分、デモ実行 15-30分、統合・報告 10分程度です。

**Q: 実際のコードは生成されますか？**
A: はい。React コンポーネント、Node.js API、テストコードが実際に生成されます。

**Q: 複数回実行できますか？**
A: はい。クリーンアップ後に何度でも実行可能です。

**Q: カスタマイズできますか？**
A: `config/director-instructions.yaml` を編集することで要求内容をカスタマイズできます。

### サポート

追加サポートが必要な場合は、以下を確認してください：

1. [QUICKSTART.md](../../QUICKSTART.md) - 基本的なセットアップ
2. [USER-GUIDE.md](../../USER-GUIDE.md) - 詳細な使用方法
3. [FAQ.md](../../FAQ.md) - よくある質問
4. [GitHub Issues](https://github.com/daktu32/wezterm-parallel/issues) - 問題報告

---

**作成日**: 2025-07-08  
**更新日**: 2025-07-08  
**バージョン**: v1.0.0