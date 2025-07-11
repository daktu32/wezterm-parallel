#!/bin/bash
# setup.sh - WezTerm Parallel デモセットアップスクリプト

set -e

echo "🚀 WezTerm Parallel デモセットアップ開始"

# プロジェクトルートディレクトリを特定
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# wezterm-parallel バイナリのパスを決定
BINARY_PATH="${PROJECT_ROOT}/target/release/wezterm-parallel"

# 前提条件チェック
if [[ ! -x "${BINARY_PATH}" ]]; then
    echo "❌ wezterm-parallel が見つかりません。先にビルドしてください："
    echo "   cd ${PROJECT_ROOT}"
    echo "   cargo build --release"
    exit 1
fi

if ! command -v curl &> /dev/null; then
    echo "❌ curl が見つかりません"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo "❌ jq が見つかりません"
    exit 1
fi

# 1. 既存プロセスの停止
echo "🧹 既存プロセスを停止中..."
pkill wezterm-parallel || true
sleep 2

# 2. フレームワーク起動
echo "🚀 WezTerm Parallel フレームワーク起動中..."
"${BINARY_PATH}" &
FRAMEWORK_PID=$!
echo "フレームワーク PID: $FRAMEWORK_PID"

# 3. フレームワーク起動待機
echo "⏳ フレームワーク起動待機中..."
for i in {1..30}; do
    if curl -s http://localhost:8080/api/health > /dev/null 2>&1; then
        echo "✅ フレームワーク起動完了"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ フレームワーク起動がタイムアウトしました"
        kill $FRAMEWORK_PID || true
        exit 1
    fi
    sleep 1
done

# 4. 成果物ディレクトリクリーンアップ
echo "🧹 成果物ディレクトリクリーンアップ中..."
rm -rf artifacts/*
mkdir -p artifacts/{frontend,backend,tests}

# 5. デモ用ワークスペース作成
echo "🏗️ デモ用ワークスペース作成中..."
curl -X POST http://localhost:8080/api/workspaces \
  -H "Content-Type: application/json" \
  -d '{
    "name": "todo-app-demo",
    "template": "web-dev-4pane",
    "processes": [
      {"id": "director", "role": "director", "claude_config": {"model": "claude-3-5-sonnet"}},
      {"id": "frontend", "role": "frontend", "claude_config": {"model": "claude-3-5-sonnet"}},
      {"id": "backend", "role": "backend", "claude_config": {"model": "claude-3-5-sonnet"}},
      {"id": "tester", "role": "tester", "claude_config": {"model": "claude-3-5-sonnet"}}
    ]
  }' | jq "."

if [ $? -ne 0 ]; then
    echo "❌ ワークスペース作成に失敗しました"
    kill $FRAMEWORK_PID || true
    exit 1
fi

# 6. 初期化完了確認
echo "🔍 ワークスペース状態確認中..."
sleep 3
curl -s http://localhost:8080/api/workspaces/todo-app-demo/status | jq "."

# 7. セットアップ完了
echo ""
echo "✅ セットアップ完了！"
echo ""
echo "📊 ダッシュボード: http://localhost:8081"
echo "🎯 API エンドポイント: http://localhost:8080/api"
echo "🏗️ ワークスペース: todo-app-demo"
echo ""
echo "次のステップ:"
echo "  ./run-demo.sh    # デモ実行"
echo "  ./coordinate.sh  # 並行開発監視"
echo "  ./integrate.sh   # 統合・報告"
echo ""
echo "フレームワーク PID: $FRAMEWORK_PID (終了時は kill $FRAMEWORK_PID)"