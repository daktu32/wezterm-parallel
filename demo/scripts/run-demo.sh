#!/bin/bash
# run-demo.sh - デモ実行スクリプト

set -e

echo "🎬 Claude Code 並列開発デモ開始"

# 前提条件チェック
if ! curl -s http://localhost:8080/api/health > /dev/null; then
    echo "❌ WezTerm Parallel フレームワークが起動していません"
    echo "   先に ./setup.sh を実行してください"
    exit 1
fi

# 1. 人間からの要求入力
echo "👤 人間からの要求: 「シンプルなToDoアプリを作って」"
echo ""

REQUEST_PAYLOAD='{
  "type": "human_request",
  "content": "シンプルなToDoアプリを作って。フロントエンドはReact、バックエンドはNode.js/Expressで実装してください。",
  "requirements": {
    "frontend": {
      "framework": "React",
      "language": "TypeScript",
      "styling": "Tailwind CSS",
      "components": ["TodoList", "TodoItem", "AddTodo", "Filter"]
    },
    "backend": {
      "framework": "Express",
      "language": "Node.js",
      "database": "SQLite",
      "endpoints": ["GET /todos", "POST /todos", "PUT /todos/:id", "DELETE /todos/:id"]
    },
    "testing": {
      "unit": "Jest + React Testing Library",
      "api": "Supertest",
      "e2e": "Cypress"
    }
  },
  "coordinator": "director"
}'

echo "📤 ディレクターに要求を送信中..."
RESPONSE=$(curl -s -X POST http://localhost:8080/api/human-request \
  -H "Content-Type: application/json" \
  -d "$REQUEST_PAYLOAD")

echo "📥 ディレクターからの応答:"
echo "$RESPONSE" | jq "."

# 2. ディレクターによる分析・計画フェーズ
echo ""
echo "🎯 Phase 1: ディレクターによる要求分析・タスク分解"
echo "⏳ 分析中..."

# タスク分解指示をディレクターに送信
ANALYSIS_PAYLOAD='{
  "type": "analyze_and_plan",
  "content": "受け取った要求を分析し、フロントエンド、バックエンド、テスターに分散するタスクを計画してください。",
  "target_process": "director"
}'

curl -s -X POST http://localhost:8080/api/processes/director/task \
  -H "Content-Type: application/json" \
  -d "$ANALYSIS_PAYLOAD" | jq "."

sleep 3

# 3. タスク分散指示
echo ""
echo "⚡ Phase 2: タスク分散・並行開発開始"

# ディレクターからエンジニアへのタスク分散
DISTRIBUTION_PAYLOAD='{
  "coordinator_id": "director",
  "tasks": [
    {
      "id": "frontend-setup",
      "title": "React TypeScript プロジェクト初期化",
      "assignee": "frontend",
      "priority": "high",
      "description": "React + TypeScript + Tailwind CSS プロジェクトを初期化し、基本的なコンポーネント構造を作成する"
    },
    {
      "id": "backend-setup", 
      "title": "Node.js Express API 初期化",
      "assignee": "backend",
      "priority": "high",
      "description": "Express サーバー、SQLite データベース、CRUD エンドポイントを実装する"
    },
    {
      "id": "test-setup",
      "title": "テスト環境・テストケース実装",
      "assignee": "tester",
      "priority": "medium",
      "description": "Jest、React Testing Library、Supertest を使ったテスト環境を構築し、テストケースを実装する"
    }
  ]
}'

echo "📤 タスク分散中..."
curl -s -X POST http://localhost:8080/api/tasks/distribute \
  -H "Content-Type: application/json" \
  -d "$DISTRIBUTION_PAYLOAD" | jq "."

# 4. 並行開発状況表示
echo ""
echo "📊 並行開発状況監視"
echo "   リアルタイム監視: http://localhost:8081"
echo "   API での状態確認: curl -s http://localhost:8080/api/workspaces/todo-app-demo/status"
echo ""

# 初期状態表示
echo "📋 初期タスク状態:"
curl -s http://localhost:8080/api/workspaces/todo-app-demo/tasks | jq "."

echo ""
echo "🎬 デモが開始されました！"
echo ""
echo "次のステップ:"
echo "  1. ダッシュボードを開く: open http://localhost:8081"
echo "  2. 並行開発を監視: ./coordinate.sh"
echo "  3. 統合・報告: ./integrate.sh"
echo ""
echo "📊 現在のプロセス状態:"
curl -s http://localhost:8080/api/processes/status | jq "."