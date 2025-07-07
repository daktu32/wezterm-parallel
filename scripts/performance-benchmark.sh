#!/bin/bash

# WezTerm Parallel - パフォーマンスベンチマークスクリプト
# 起動時間・メモリ使用量・応答性の包括的測定

set -e

# カラー定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# 設定
BENCHMARK_RUNS=5
WARMUP_RUNS=2
BINARY_PATH="${1:-./target/release/wezterm-parallel}"
RESULTS_DIR="./benchmark-results"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
RESULTS_FILE="$RESULTS_DIR/benchmark-$TIMESTAMP.json"

# ヘルパー関数
info() {
    echo -e "${BLUE}📊 $1${NC}"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
}

header() {
    echo -e "${PURPLE}=== $1 ===${NC}"
}

# 結果ディレクトリ作成
mkdir -p "$RESULTS_DIR"

# ベンチマーク結果を格納するJSON構造を初期化
cat > "$RESULTS_FILE" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "binary_path": "$BINARY_PATH",
  "system_info": {},
  "benchmarks": {
    "startup_time": [],
    "memory_usage": [],
    "api_response": [],
    "stress_test": {}
  }
}
EOF

# システム情報収集
collect_system_info() {
    header "システム情報収集"
    
    local system_info=$(cat <<EOF
{
  "os": "$(uname -s)",
  "kernel": "$(uname -r)",
  "architecture": "$(uname -m)",
  "cpu_cores": $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "unknown"),
  "total_memory_mb": $(free -m 2>/dev/null | awk '/^Mem:/{print $2}' || sysctl -n hw.memsize 2>/dev/null | awk '{print int($1/1024/1024)}' || echo "unknown"),
  "rust_version": "$(rustc --version 2>/dev/null || echo "unknown")",
  "build_type": "release"
}
EOF
    )
    
    # JSONファイルにシステム情報を追加
    echo "$system_info" | jq '.' > /tmp/system_info.json
    jq '.system_info = input' "$RESULTS_FILE" /tmp/system_info.json > /tmp/results_tmp.json
    mv /tmp/results_tmp.json "$RESULTS_FILE"
    
    info "OS: $(uname -s) $(uname -r)"
    info "CPU: $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "unknown") cores"
    info "Memory: $(free -m 2>/dev/null | awk '/^Mem:/{print $2}' || sysctl -n hw.memsize 2>/dev/null | awk '{print int($1/1024/1024)}' || echo "unknown") MB"
}

# バイナリ存在確認
check_binary() {
    if [[ ! -f "$BINARY_PATH" ]]; then
        error "バイナリが見つかりません: $BINARY_PATH"
        echo "使用方法: $0 [binary_path]"
        exit 1
    fi
    
    if [[ ! -x "$BINARY_PATH" ]]; then
        error "バイナリに実行権限がありません: $BINARY_PATH"
        exit 1
    fi
    
    success "バイナリ確認完了: $BINARY_PATH"
}

# プロセス終了
cleanup() {
    info "クリーンアップ中..."
    pkill -f wezterm-parallel 2>/dev/null || true
    sleep 2
}

# 起動時間ベンチマーク
benchmark_startup_time() {
    header "起動時間ベンチマーク"
    
    local startup_times=()
    
    for i in $(seq 1 $BENCHMARK_RUNS); do
        info "起動時間測定 $i/$BENCHMARK_RUNS"
        
        cleanup
        
        # 起動時間測定
        local start_time=$(date +%s.%N)
        
        # バックグラウンドで起動
        timeout 30s "$BINARY_PATH" &
        local pid=$!
        
        # APIが応答するまで待機
        local ready=false
        local timeout_counter=0
        while [[ $ready == false && $timeout_counter -lt 300 ]]; do # 30秒タイムアウト
            if curl -s http://localhost:8080/api/status >/dev/null 2>&1; then
                ready=true
            else
                sleep 0.1
                ((timeout_counter++))
            fi
        done
        
        local end_time=$(date +%s.%N)
        local startup_time=$(echo "$end_time - $start_time" | bc)
        
        if [[ $ready == true ]]; then
            startup_times+=("$startup_time")
            success "起動時間: ${startup_time}秒"
        else
            error "起動タイムアウト (30秒)"
            kill $pid 2>/dev/null || true
            startup_times+=("null")
        fi
        
        cleanup
        sleep 1
    done
    
    # 結果をJSONに保存
    local startup_json=$(printf '%s\n' "${startup_times[@]}" | jq -R 'if . == "null" then null else tonumber end' | jq -s '.')
    jq ".benchmarks.startup_time = $startup_json" "$RESULTS_FILE" > /tmp/results_tmp.json
    mv /tmp/results_tmp.json "$RESULTS_FILE"
    
    # 統計計算
    local valid_times=($(printf '%s\n' "${startup_times[@]}" | grep -v "null"))
    if [[ ${#valid_times[@]} -gt 0 ]]; then
        local avg=$(echo "${valid_times[@]}" | tr ' ' '\n' | awk '{sum+=$1} END {print sum/NR}')
        local min=$(echo "${valid_times[@]}" | tr ' ' '\n' | sort -n | head -1)
        local max=$(echo "${valid_times[@]}" | tr ' ' '\n' | sort -n | tail -1)
        
        success "起動時間統計:"
        echo "  平均: ${avg}秒"
        echo "  最小: ${min}秒" 
        echo "  最大: ${max}秒"
        echo "  成功率: ${#valid_times[@]}/$BENCHMARK_RUNS"
    else
        error "有効な起動時間測定結果がありません"
    fi
}

# メモリ使用量ベンチマーク
benchmark_memory_usage() {
    header "メモリ使用量ベンチマーク"
    
    cleanup
    
    info "フレームワーク起動中..."
    "$BINARY_PATH" &
    local pid=$!
    
    # 起動完了まで待機
    local ready=false
    local timeout_counter=0
    while [[ $ready == false && $timeout_counter -lt 300 ]]; do
        if curl -s http://localhost:8080/api/status >/dev/null 2>&1; then
            ready=true
        else
            sleep 0.1
            ((timeout_counter++))
        fi
    done
    
    if [[ $ready == false ]]; then
        error "起動タイムアウト - メモリ測定をスキップ"
        kill $pid 2>/dev/null || true
        return
    fi
    
    success "起動完了 - メモリ測定開始"
    
    local memory_samples=()
    
    for i in $(seq 1 60); do # 60秒間測定
        local memory_kb=$(ps -o rss= -p $pid 2>/dev/null || echo "0")
        local memory_mb=$(echo "scale=2; $memory_kb / 1024" | bc)
        memory_samples+=("$memory_mb")
        
        if [[ $((i % 10)) -eq 0 ]]; then
            info "メモリ使用量 (${i}秒): ${memory_mb}MB"
        fi
        
        sleep 1
    done
    
    cleanup
    
    # 統計計算
    local avg_memory=$(echo "${memory_samples[@]}" | tr ' ' '\n' | awk '{sum+=$1} END {print sum/NR}')
    local min_memory=$(echo "${memory_samples[@]}" | tr ' ' '\n' | sort -n | head -1)
    local max_memory=$(echo "${memory_samples[@]}" | tr ' ' '\n' | sort -n | tail -1)
    
    # 結果をJSONに保存
    local memory_json=$(printf '%s\n' "${memory_samples[@]}" | jq -R 'tonumber' | jq -s '{
        samples: .,
        avg_mb: (add / length),
        min_mb: min,
        max_mb: max,
        sample_count: length
    }')
    
    jq ".benchmarks.memory_usage = $memory_json" "$RESULTS_FILE" > /tmp/results_tmp.json
    mv /tmp/results_tmp.json "$RESULTS_FILE"
    
    success "メモリ使用量統計:"
    echo "  平均: ${avg_memory}MB"
    echo "  最小: ${min_memory}MB"
    echo "  最大: ${max_memory}MB"
    echo "  サンプル数: ${#memory_samples[@]}"
}

# API応答性能ベンチマーク
benchmark_api_response() {
    header "API応答性能ベンチマーク"
    
    cleanup
    
    info "フレームワーク起動中..."
    "$BINARY_PATH" &
    local pid=$!
    
    # 起動完了まで待機
    local ready=false
    local timeout_counter=0
    while [[ $ready == false && $timeout_counter -lt 300 ]]; do
        if curl -s http://localhost:8080/api/status >/dev/null 2>&1; then
            ready=true
        else
            sleep 0.1
            ((timeout_counter++))
        fi
    done
    
    if [[ $ready == false ]]; then
        error "起動タイムアウト - API測定をスキップ"
        kill $pid 2>/dev/null || true
        return
    fi
    
    success "起動完了 - API応答性能測定開始"
    
    # 各エンドポイントの応答時間を測定
    local endpoints=(
        "/api/status"
        "/api/workspaces"
        "/api/processes"
        "/api/system/metrics"
    )
    
    local api_results=()
    
    for endpoint in "${endpoints[@]}"; do
        info "エンドポイント測定: $endpoint"
        
        local response_times=()
        local success_count=0
        
        for i in $(seq 1 10); do # 各エンドポイント10回測定
            local start_time=$(date +%s.%N)
            
            if curl -s "http://localhost:8080$endpoint" >/dev/null; then
                local end_time=$(date +%s.%N)
                local response_time=$(echo "$end_time - $start_time" | bc)
                response_times+=("$response_time")
                ((success_count++))
            else
                response_times+=("null")
            fi
        done
        
        # 統計計算
        local valid_times=($(printf '%s\n' "${response_times[@]}" | grep -v "null"))
        if [[ ${#valid_times[@]} -gt 0 ]]; then
            local avg=$(echo "${valid_times[@]}" | tr ' ' '\n' | awk '{sum+=$1} END {print sum/NR}')
            local min=$(echo "${valid_times[@]}" | tr ' ' '\n' | sort -n | head -1)
            local max=$(echo "${valid_times[@]}" | tr ' ' '\n' | sort -n | tail -1)
            
            success "  平均応答時間: ${avg}秒"
            echo "  最小: ${min}秒"
            echo "  最大: ${max}秒"
            echo "  成功率: ${success_count}/10"
            
            api_results+=("{\"endpoint\": \"$endpoint\", \"avg_response_time\": $avg, \"min_response_time\": $min, \"max_response_time\": $max, \"success_rate\": $(echo "scale=2; $success_count / 10" | bc)}")
        else
            warning "  エンドポイント $endpoint の測定に失敗"
            api_results+=("{\"endpoint\": \"$endpoint\", \"error\": \"no_valid_responses\"}")
        fi
    done
    
    cleanup
    
    # 結果をJSONに保存
    local api_json=$(printf '%s\n' "${api_results[@]}" | jq -s '.')
    jq ".benchmarks.api_response = $api_json" "$RESULTS_FILE" > /tmp/results_tmp.json
    mv /tmp/results_tmp.json "$RESULTS_FILE"
}

# ストレステスト
benchmark_stress_test() {
    header "ストレステスト"
    
    cleanup
    
    info "フレームワーク起動中..."
    "$BINARY_PATH" &
    local pid=$!
    
    # 起動完了まで待機
    local ready=false
    local timeout_counter=0
    while [[ $ready == false && $timeout_counter -lt 300 ]]; do
        if curl -s http://localhost:8080/api/status >/dev/null 2>&1; then
            ready=true
        else
            sleep 0.1
            ((timeout_counter++))
        fi
    done
    
    if [[ $ready == false ]]; then
        error "起動タイムアウト - ストレステストをスキップ"
        kill $pid 2>/dev/null || true
        return
    fi
    
    success "起動完了 - ストレステスト開始"
    
    # 複数ワークスペースの同時作成
    info "複数ワークスペース同時作成テスト"
    local start_time=$(date +%s.%N)
    local workspace_count=10
    local success_count=0
    
    for i in $(seq 1 $workspace_count); do
        if curl -s -X POST http://localhost:8080/api/workspaces \
           -H "Content-Type: application/json" \
           -d "{\"name\": \"stress-test-$i\", \"template\": \"basic\"}" >/dev/null; then
            ((success_count++))
        fi &
    done
    
    wait # 全ての並列処理完了を待機
    
    local end_time=$(date +%s.%N)
    local total_time=$(echo "$end_time - $start_time" | bc)
    
    success "ワークスペース作成: $success_count/$workspace_count 成功"
    echo "  合計時間: ${total_time}秒"
    echo "  スループット: $(echo "scale=2; $success_count / $total_time" | bc) ops/sec"
    
    # 高負荷API呼び出し
    info "高負荷API呼び出しテスト (30秒間)"
    local api_start_time=$(date +%s.%N)
    local api_success_count=0
    local api_total_requests=0
    
    timeout 30s bash -c '
        while true; do
            curl -s http://localhost:8080/api/status >/dev/null
            echo $?
        done
    ' | while read result; do
        ((api_total_requests++))
        if [[ $result -eq 0 ]]; then
            ((api_success_count++))
        fi
    done
    
    local api_end_time=$(date +%s.%N)
    local api_duration=$(echo "$api_end_time - $api_start_time" | bc)
    
    # ストレステスト結果をJSONに保存
    local stress_json=$(cat <<EOF
{
  "workspace_creation": {
    "target_count": $workspace_count,
    "success_count": $success_count,
    "total_time_seconds": $total_time,
    "throughput_ops_per_second": $(echo "scale=2; $success_count / $total_time" | bc)
  },
  "api_load_test": {
    "duration_seconds": $api_duration,
    "total_requests": $api_total_requests,
    "success_rate": $(echo "scale=2; $api_success_count / $api_total_requests" | bc 2>/dev/null || echo "0")
  }
}
EOF
    )
    
    echo "$stress_json" | jq '.' > /tmp/stress_results.json
    jq '.benchmarks.stress_test = input' "$RESULTS_FILE" /tmp/stress_results.json > /tmp/results_tmp.json
    mv /tmp/results_tmp.json "$RESULTS_FILE"
    
    cleanup
}

# レポート生成
generate_report() {
    header "ベンチマークレポート生成"
    
    local report_file="$RESULTS_DIR/report-$TIMESTAMP.md"
    
    cat > "$report_file" << 'EOF'
# WezTerm Parallel パフォーマンスベンチマーク レポート

EOF

    # システム情報
    echo "## システム情報" >> "$report_file"
    echo "" >> "$report_file"
    jq -r '.system_info | to_entries[] | "- **\(.key)**: \(.value)"' "$RESULTS_FILE" >> "$report_file"
    echo "" >> "$report_file"
    
    # 起動時間
    echo "## 起動時間" >> "$report_file"
    echo "" >> "$report_file"
    local startup_stats=$(jq -r '
        .benchmarks.startup_time 
        | map(select(. != null)) 
        | if length > 0 then 
            "- **平均**: " + (add / length | tostring) + "秒\n" +
            "- **最小**: " + (min | tostring) + "秒\n" +
            "- **最大**: " + (max | tostring) + "秒\n" +
            "- **成功率**: " + (length | tostring) + "/" + (length | tostring)
          else 
            "測定データなし" 
          end
    ' "$RESULTS_FILE")
    echo "$startup_stats" >> "$report_file"
    echo "" >> "$report_file"
    
    # メモリ使用量
    echo "## メモリ使用量" >> "$report_file"
    echo "" >> "$report_file"
    jq -r '.benchmarks.memory_usage | 
        if . then
            "- **平均**: " + (.avg_mb | tostring) + "MB\n" +
            "- **最小**: " + (.min_mb | tostring) + "MB\n" +
            "- **最大**: " + (.max_mb | tostring) + "MB\n" +
            "- **サンプル数**: " + (.sample_count | tostring)
        else
            "測定データなし"
        end
    ' "$RESULTS_FILE" >> "$report_file"
    echo "" >> "$report_file"
    
    # API応答性能
    echo "## API応答性能" >> "$report_file"
    echo "" >> "$report_file"
    jq -r '.benchmarks.api_response[]? | 
        if .error then
            "- **" + .endpoint + "**: エラー (" + .error + ")"
        else
            "- **" + .endpoint + "**: " + (.avg_response_time | tostring) + "秒 (成功率: " + (.success_rate | tostring) + ")"
        end
    ' "$RESULTS_FILE" >> "$report_file"
    echo "" >> "$report_file"
    
    # ストレステスト
    echo "## ストレステスト" >> "$report_file"
    echo "" >> "$report_file"
    jq -r '.benchmarks.stress_test | 
        if . then
            "### ワークスペース作成\n" +
            "- **成功率**: " + (.workspace_creation.success_count | tostring) + "/" + (.workspace_creation.target_count | tostring) + "\n" +
            "- **スループット**: " + (.workspace_creation.throughput_ops_per_second | tostring) + " ops/sec\n" +
            "\n### API負荷テスト\n" +
            "- **成功率**: " + (.api_load_test.success_rate | tostring) + "\n" +
            "- **継続時間**: " + (.api_load_test.duration_seconds | tostring) + "秒"
        else
            "測定データなし"
        end
    ' "$RESULTS_FILE" >> "$report_file"
    
    success "レポート生成完了: $report_file"
    success "詳細データ: $RESULTS_FILE"
}

# メイン実行
main() {
    header "WezTerm Parallel パフォーマンスベンチマーク"
    echo "開始時間: $(date)"
    echo "バイナリ: $BINARY_PATH"
    echo "実行回数: $BENCHMARK_RUNS"
    echo ""
    
    collect_system_info
    check_binary
    
    benchmark_startup_time
    benchmark_memory_usage
    benchmark_api_response
    benchmark_stress_test
    
    generate_report
    
    header "ベンチマーク完了"
    echo "結果ファイル: $RESULTS_FILE"
    echo "レポート: $RESULTS_DIR/report-$TIMESTAMP.md"
}

# トラップで確実にクリーンアップ
trap cleanup EXIT

# メイン実行
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi