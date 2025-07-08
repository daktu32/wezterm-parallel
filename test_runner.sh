#!/bin/bash
# WezTerm Parallel フレームワーク テストランナー
# 本番環境に影響しない独立したテスト環境を提供

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_CONFIG="$SCRIPT_DIR/test_wezterm.lua"
BACKUP_CONFIG=""
TEST_SOCKET="/tmp/wezterm-parallel-test.sock"

# カラー出力
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 現在のWezTerm設定をバックアップ
backup_current_config() {
    if [[ -f ~/.config/wezterm/wezterm.lua ]]; then
        BACKUP_CONFIG=~/.config/wezterm/wezterm.lua.test-backup-$(date +%Y%m%d-%H%M%S)
        cp ~/.config/wezterm/wezterm.lua "$BACKUP_CONFIG"
        print_info "現在の設定をバックアップ: $BACKUP_CONFIG"
    else
        print_info "既存のWezTerm設定なし"
    fi
}

# テスト設定を適用
apply_test_config() {
    mkdir -p ~/.config/wezterm
    cp "$TEST_CONFIG" ~/.config/wezterm/wezterm.lua
    print_success "テスト設定を適用"
}

# 設定を復元
restore_config() {
    if [[ -n "$BACKUP_CONFIG" && -f "$BACKUP_CONFIG" ]]; then
        cp "$BACKUP_CONFIG" ~/.config/wezterm/wezterm.lua
        rm "$BACKUP_CONFIG"
        print_success "元の設定を復元"
    else
        rm -f ~/.config/wezterm/wezterm.lua
        print_info "テスト設定を削除（元の設定なし）"
    fi
}

# バックエンドサービス起動
start_backend() {
    print_info "バックエンドサービスを起動中..."
    
    # 既存のテストソケットを削除
    rm -f "$TEST_SOCKET"
    
    # バックエンドを起動（バックグラウンド）
    cd "$SCRIPT_DIR"
    cargo run &
    BACKEND_PID=$!
    
    # 起動確認
    sleep 3
    if kill -0 "$BACKEND_PID" 2>/dev/null; then
        print_success "バックエンドサービス起動 (PID: $BACKEND_PID)"
        echo "$BACKEND_PID" > /tmp/wezterm-parallel-test.pid
    else
        print_error "バックエンドサービス起動失敗"
        exit 1
    fi
}

# バックエンドサービス停止
stop_backend() {
    if [[ -f /tmp/wezterm-parallel-test.pid ]]; then
        local pid=$(cat /tmp/wezterm-parallel-test.pid)
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid"
            print_success "バックエンドサービス停止"
        fi
        rm -f /tmp/wezterm-parallel-test.pid
    fi
    
    # 念のため全体的にクリーンアップ
    pkill -f "wezterm-parallel" 2>/dev/null || true
    rm -f "$TEST_SOCKET"
}

# テスト環境セットアップ
setup_test() {
    print_info "=== WezTerm Parallel テスト環境セットアップ ==="
    
    backup_current_config
    apply_test_config
    start_backend
    
    print_success "テスト環境準備完了"
    echo ""
    print_info "次の手順でテストを実行してください："
    echo ""
    echo "1. 新しいWezTermウィンドウを開く"
    echo "2. タブタイトルに [TEST] が表示されることを確認"
    echo "3. 以下のキーバインドをテスト："
    echo "   - Ctrl+Shift+D : ダッシュボード切り替え"
    echo "   - Ctrl+Shift+R : ダッシュボード更新"
    echo "   - Ctrl+Shift+T : バックエンド接続テスト"
    echo "   - Ctrl+Shift+H : ヘルプ表示"
    echo "   - Ctrl+Shift+Q : テスト終了"
    echo ""
    print_warning "テスト完了後は必ず '$0 cleanup' を実行してください"
}

# テスト環境クリーンアップ
cleanup_test() {
    print_info "=== テスト環境クリーンアップ ==="
    
    stop_backend
    restore_config
    
    print_success "テスト環境クリーンアップ完了"
    print_info "WezTermの設定リロード（Ctrl+Shift+R）を推奨"
}

# ヘルプ表示
show_help() {
    echo "WezTerm Parallel テストランナー"
    echo ""
    echo "使用方法:"
    echo "  $0 setup    - テスト環境をセットアップ"
    echo "  $0 cleanup  - テスト環境をクリーンアップ"
    echo "  $0 status   - テスト環境の状態確認"
    echo "  $0 help     - このヘルプを表示"
    echo ""
    echo "テスト手順:"
    echo "  1. $0 setup"
    echo "  2. 新しいWezTermウィンドウでテスト実行"
    echo "  3. $0 cleanup"
}

# ステータス確認
check_status() {
    print_info "=== テスト環境ステータス ==="
    
    # バックエンド確認
    if [[ -f /tmp/wezterm-parallel-test.pid ]]; then
        local pid=$(cat /tmp/wezterm-parallel-test.pid)
        if kill -0 "$pid" 2>/dev/null; then
            print_success "バックエンドサービス: 稼働中 (PID: $pid)"
        else
            print_error "バックエンドサービス: 停止中"
        fi
    else
        print_info "バックエンドサービス: 未起動"
    fi
    
    # ソケット確認
    if [[ -S "$TEST_SOCKET" ]]; then
        print_success "テストソケット: 利用可能"
    else
        print_warning "テストソケット: なし"
    fi
    
    # 設定確認
    if [[ -f ~/.config/wezterm/wezterm.lua ]]; then
        if grep -q "TEST" ~/.config/wezterm/wezterm.lua; then
            print_warning "WezTerm設定: テスト設定適用中"
        else
            print_info "WezTerm設定: 通常設定"
        fi
    else
        print_info "WezTerm設定: なし"
    fi
    
    # ダッシュボード確認
    if curl -s --connect-timeout 2 ws://127.0.0.1:9999/ >/dev/null 2>&1; then
        print_success "WebSocketダッシュボード: 接続可能"
    else
        print_warning "WebSocketダッシュボード: 接続不可"
    fi
}

# メイン処理
case "${1:-help}" in
    setup)
        setup_test
        ;;
    cleanup)
        cleanup_test
        ;;
    status)
        check_status
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        print_error "不明なコマンド: $1"
        show_help
        exit 1
        ;;
esac