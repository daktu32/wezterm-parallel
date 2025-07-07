#!/bin/bash

# WezTerm Parallel - 簡易セットアップスクリプト
# このスクリプトは初回セットアップを自動化します

set -e

echo "🚀 WezTerm Parallel セットアップスクリプト"
echo "========================================"

# カラー定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ヘルパー関数
info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
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

# 前提条件チェック
check_prerequisites() {
    info "前提条件をチェック中..."
    
    # WezTerm チェック
    if command -v wezterm &> /dev/null; then
        WEZTERM_VERSION=$(wezterm --version | cut -d' ' -f2)
        success "WezTerm が見つかりました: $WEZTERM_VERSION"
    else
        error "WezTerm が見つかりません"
        echo "  インストール: https://wezfurlong.org/wezterm/installation.html"
        exit 1
    fi
    
    # Rust チェック
    if command -v rustc &> /dev/null; then
        RUST_VERSION=$(rustc --version | cut -d' ' -f2)
        success "Rust が見つかりました: $RUST_VERSION"
    else
        error "Rust が見つかりません"
        echo "  インストール: https://rustup.rs/"
        exit 1
    fi
    
    # Claude Code チェック (オプション)
    if command -v claude-code &> /dev/null; then
        CLAUDE_VERSION=$(claude-code --version 2>/dev/null || echo "unknown")
        success "Claude Code が見つかりました: $CLAUDE_VERSION"
    else
        warning "Claude Code が見つかりません (オプション機能)"
        echo "  基本機能は Claude Code なしでも動作します"
    fi
}

# ビルド実行
build_project() {
    info "プロジェクトをビルド中..."
    
    if cargo build --release; then
        success "ビルドが完了しました"
    else
        error "ビルドに失敗しました"
        exit 1
    fi
}

# 設定ファイルセットアップ
setup_config() {
    info "設定ファイルをセットアップ中..."
    
    # ディレクトリ作成
    mkdir -p ~/.config/wezterm-parallel
    mkdir -p ~/.config/wezterm
    mkdir -p ~/.local/share/wezterm-parallel/logs
    
    # 設定ファイルコピー
    if [ -f "config/quickstart-config.yaml" ]; then
        cp config/quickstart-config.yaml ~/.config/wezterm-parallel/config.yaml
        success "WezTerm Parallel設定ファイルを配置しました"
    else
        error "quickstart-config.yaml が見つかりません"
        exit 1
    fi
    
    # WezTerm設定（オプション）
    if [ -f "config/quickstart-wezterm.lua" ]; then
        if [ -f ~/.config/wezterm/wezterm.lua ]; then
            warning "既存のWezTerm設定が見つかりました"
            echo -n "上書きしますか？ (y/N): "
            read -r response
            if [[ "$response" =~ ^[Yy]$ ]]; then
                cp ~/.config/wezterm/wezterm.lua ~/.config/wezterm/wezterm.lua.backup
                cp config/quickstart-wezterm.lua ~/.config/wezterm/wezterm.lua
                success "WezTerm設定ファイルを更新しました（バックアップ作成済み）"
            else
                info "WezTerm設定はスキップしました"
            fi
        else
            cp config/quickstart-wezterm.lua ~/.config/wezterm/wezterm.lua
            success "WezTerm設定ファイルを配置しました"
        fi
    fi
}

# 動作確認
test_installation() {
    info "動作確認を実行中..."
    
    # バイナリテスト
    if ./target/release/wezterm-parallel --help &> /dev/null; then
        success "バイナリが正常に動作します"
    else
        error "バイナリの動作確認に失敗しました"
        exit 1
    fi
    
    # 設定ファイルテスト
    if [ -f ~/.config/wezterm-parallel/config.yaml ]; then
        success "設定ファイルが配置されています"
    else
        error "設定ファイルが見つかりません"
        exit 1
    fi
}

# 次のステップ案内
show_next_steps() {
    echo ""
    echo "🎉 セットアップが完了しました！"
    echo "==============================="
    echo ""
    echo "📋 次のステップ:"
    echo ""
    echo "1. フレームワークを起動:"
    echo "   ./target/release/wezterm-parallel"
    echo ""
    echo "2. ダッシュボードにアクセス:"
    echo "   http://localhost:8081"
    echo ""
    echo "3. API テスト:"
    echo "   curl http://localhost:8080/api/status"
    echo ""
    echo "4. WezTerm キーバインド（WezTerm内で）:"
    echo "   Ctrl+Shift+N: 新しいワークスペース作成"
    echo "   Ctrl+Shift+D: ダッシュボードを開く"
    echo "   Ctrl+Alt+S:   フレームワーク状態確認"
    echo ""
    echo "📚 詳細な使い方:"
    echo "   - QUICKSTART.md   : 基本的な使い方"
    echo "   - SETUP-GUIDE.md  : 詳細設定とカスタマイズ"
    echo ""
    echo "🆘 問題が発生した場合:"
    echo "   - docs/QUICKSTART-TROUBLESHOOTING.md"
    echo "   - GitHub Issues: https://github.com/daktu32/wezterm-parallel/issues"
}

# メイン実行
main() {
    check_prerequisites
    build_project
    setup_config
    test_installation
    show_next_steps
}

# スクリプト実行
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi