#!/bin/bash
# integrate.sh - 統合・報告スクリプト

set -e

echo "🔄 統合・報告フェーズ開始"

# 前提条件チェック
if ! curl -s http://localhost:8080/api/health > /dev/null; then
    echo "❌ WezTerm Parallel フレームワークが起動していません"
    exit 1
fi

# 1. 現在の開発状況確認
echo "📊 開発完了状況確認中..."
echo ""

echo "📋 全タスク状況:"
TASKS_RESPONSE=$(curl -s http://localhost:8080/api/workspaces/todo-app-demo/tasks)
echo "$TASKS_RESPONSE" | jq "."

TOTAL_TASKS=$(echo "$TASKS_RESPONSE" | jq 'length')
COMPLETED_TASKS=$(echo "$TASKS_RESPONSE" | jq '[.[] | select(.status == "completed")] | length')
IN_PROGRESS_TASKS=$(echo "$TASKS_RESPONSE" | jq '[.[] | select(.status == "in_progress")] | length')

echo ""
echo "📈 進捗サマリー:"
echo "  総タスク数: $TOTAL_TASKS"
echo "  完了: $COMPLETED_TASKS"
echo "  進行中: $IN_PROGRESS_TASKS"
echo "  残り: $((TOTAL_TASKS - COMPLETED_TASKS))"

if [ "$TOTAL_TASKS" -gt 0 ]; then
    PROGRESS=$((COMPLETED_TASKS * 100 / TOTAL_TASKS))
    echo "  進捗率: $PROGRESS%"
fi

# 2. 各エンジニアの成果物収集
echo ""
echo "📦 各エンジニアの成果物収集中..."

echo ""
echo "🎨 フロントエンド成果物:"
curl -s http://localhost:8080/api/processes/frontend/artifacts 2>/dev/null | jq "." || echo "  成果物情報を取得中..."

echo ""
echo "🔧 バックエンド成果物:"
curl -s http://localhost:8080/api/processes/backend/artifacts 2>/dev/null | jq "." || echo "  成果物情報を取得中..."

echo ""
echo "🧪 テスト成果物:"
curl -s http://localhost:8080/api/processes/tester/artifacts 2>/dev/null | jq "." || echo "  成果物情報を取得中..."

# 3. ディレクターによる統合指示
echo ""
echo "🎯 ディレクターによる統合・品質確認指示"

INTEGRATION_PAYLOAD='{
  "type": "integrate_and_review",
  "content": "各エンジニアの成果物を統合し、動作確認とコードレビューを実行してください。品質、完成度、課題を評価して報告してください。",
  "target_process": "director"
}'

echo "📤 ディレクターに統合指示を送信中..."
DIRECTOR_RESPONSE=$(curl -s -X POST http://localhost:8080/api/processes/director/task \
  -H "Content-Type: application/json" \
  -d "$INTEGRATION_PAYLOAD")

echo "📥 ディレクターからの応答:"
echo "$DIRECTOR_RESPONSE" | jq "."

# 4. 統合テスト実行
echo ""
echo "🧪 統合テスト実行中..."

TEST_PAYLOAD='{
  "type": "integration_test",
  "target_workspace": "todo-app-demo",
  "test_types": ["unit", "api", "integration"]
}'

curl -s -X POST http://localhost:8080/api/workspaces/todo-app-demo/test \
  -H "Content-Type: application/json" \
  -d "$TEST_PAYLOAD" | jq "." || echo "統合テスト情報を取得中..."

# 5. 最終成果物の確認
echo ""
echo "📊 最終成果物確認"

# 成果物ディレクトリに保存
echo "💾 成果物をローカルに保存中..."
mkdir -p artifacts/final

# ワークスペース全体の状態をレポートとして保存
curl -s http://localhost:8080/api/workspaces/todo-app-demo/report > artifacts/final/project-report.json
echo "  📄 プロジェクトレポート: artifacts/final/project-report.json"

# メトリクス保存
curl -s http://localhost:8080/api/metrics > artifacts/final/metrics.json
echo "  📊 メトリクス: artifacts/final/metrics.json"

# 6. 人間への最終報告
echo ""
echo "🎊 統合・報告完了！"
echo ""
echo "==================== 最終報告 ===================="

# プロジェクト完成度評価
if [ "$PROGRESS" -ge 80 ]; then
    echo "✅ プロジェクト状態: 概ね完成"
elif [ "$PROGRESS" -ge 50 ]; then
    echo "🔄 プロジェクト状態: 実装中"
else
    echo "🚧 プロジェクト状態: 初期段階"
fi

echo ""
echo "📋 成果サマリー:"
echo "  - ToDoアプリプロジェクト"
echo "  - 並行開発プロセス: 4つ（ディレクター、フロントエンド、バックエンド、テスター）"
echo "  - 技術スタック: React + TypeScript, Node.js + Express, SQLite"
echo "  - 進捗率: $PROGRESS% ($COMPLETED_TASKS/$TOTAL_TASKS タスク完了)"

echo ""
echo "📁 成果物:"
echo "  - artifacts/final/project-report.json"
echo "  - artifacts/final/metrics.json"
echo "  - http://localhost:8081 (ダッシュボード)"

echo ""
echo "🎯 デモ完了項目:"
echo "  ✅ 人間の要求受付・分析"
echo "  ✅ タスク分解・分散"
echo "  ✅ 複数プロセス並行開発"
echo "  ✅ リアルタイム監視"
echo "  ✅ 成果物統合・品質確認"
echo "  ✅ 最終報告"

echo ""
echo "=================================================="
echo ""
echo "🎭 Claude Code 並列開発デモが完了しました！"
echo ""
echo "📊 詳細結果: cat artifacts/final/project-report.json | jq '.'"
echo "🌐 ダッシュボード: http://localhost:8081"
echo "🧹 クリーンアップ: pkill wezterm-parallel"