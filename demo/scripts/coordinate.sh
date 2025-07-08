#!/bin/bash
# coordinate.sh - 並行開発協調・監視スクリプト

set -e

echo "🎯 並行開発協調・監視開始"

# 前提条件チェック
if ! curl -s http://localhost:8080/api/health > /dev/null; then
    echo "❌ WezTerm Parallel フレームワークが起動していません"
    exit 1
fi

# 1. 現在の状態確認
echo "📊 現在のワークスペース状態:"
curl -s http://localhost:8080/api/workspaces/todo-app-demo/status | jq "."

echo ""
echo "📋 現在のタスク状態:"
curl -s http://localhost:8080/api/workspaces/todo-app-demo/tasks | jq "."

echo ""
echo "🤖 プロセス状態:"
curl -s http://localhost:8080/api/processes/status | jq "."

# 2. リアルタイム監視の選択肢提示
echo ""
echo "📊 リアルタイム監視オプション:"
echo "  1. ダッシュボード表示 (推奨)"
echo "  2. ターミナルでの継続監視"
echo "  3. 1回だけ状態確認"
echo ""
read -p "選択してください [1-3]: " choice

case $choice in
    1)
        echo "🌐 ダッシュボードを開きます..."
        if command -v open &> /dev/null; then
            open http://localhost:8081
        else
            echo "ブラウザで http://localhost:8081 を開いてください"
        fi
        echo "ダッシュボードでリアルタイム監視を開始してください"
        ;;
    2)
        echo "📊 ターミナルでの継続監視を開始 (Ctrl+C で終了)"
        echo ""
        while true; do
            clear
            echo "🕒 $(date '+%Y-%m-%d %H:%M:%S') - WezTerm Parallel 並行開発監視"
            echo "========================================================"
            
            echo ""
            echo "📊 ワークスペース状態:"
            curl -s http://localhost:8080/api/workspaces/todo-app-demo/status | jq -r '.status // "Unknown"'
            
            echo ""
            echo "📋 アクティブタスク:"
            curl -s http://localhost:8080/api/workspaces/todo-app-demo/tasks | \
                jq -r '.[] | select(.status == "in_progress") | "- \(.title) (\(.assignee))"'
            
            echo ""
            echo "🤖 プロセス状態:"
            curl -s http://localhost:8080/api/processes/status | \
                jq -r '.[] | "\(.id): \(.status) (CPU: \(.cpu_usage // 0)%, Mem: \(.memory_usage // 0)MB)"'
            
            echo ""
            echo "📈 進捗状況:"
            TOTAL_TASKS=$(curl -s http://localhost:8080/api/workspaces/todo-app-demo/tasks | jq 'length')
            COMPLETED_TASKS=$(curl -s http://localhost:8080/api/workspaces/todo-app-demo/tasks | jq '[.[] | select(.status == "completed")] | length')
            if [ "$TOTAL_TASKS" -gt 0 ]; then
                PROGRESS=$((COMPLETED_TASKS * 100 / TOTAL_TASKS))
                echo "完了: $COMPLETED_TASKS/$TOTAL_TASKS タスク ($PROGRESS%)"
            else
                echo "タスク情報を取得中..."
            fi
            
            echo ""
            echo "========================================================"
            echo "Ctrl+C で監視終了、Enter でダッシュボード表示"
            
            sleep 5
        done
        ;;
    3)
        echo "📊 現在の状態確認:"
        echo ""
        
        echo "📋 タスク詳細:"
        curl -s http://localhost:8080/api/workspaces/todo-app-demo/tasks | jq "."
        
        echo ""
        echo "🤖 プロセス詳細:"
        curl -s http://localhost:8080/api/processes/status | jq "."
        
        echo ""
        echo "📊 メトリクス:"
        curl -s http://localhost:8080/api/metrics | jq "."
        ;;
    *)
        echo "❌ 無効な選択です"
        exit 1
        ;;
esac

echo ""
echo "📊 協調監視完了"
echo ""
echo "次のステップ:"
echo "  ./integrate.sh   # 統合・報告フェーズ"
echo "  kill \$PID       # フレームワーク終了"