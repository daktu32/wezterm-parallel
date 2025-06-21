#!/bin/bash
# WezTerm Multi-Process Development Framework - インストールスクリプト
# このスクリプトは自動的にフレームワークをセットアップします

set -e  # エラー時に終了

# カラー定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ロゴ表示
print_logo() {
    echo -e "${BLUE}"
    echo "🚀 WezTerm Multi-Process Development Framework"
    echo "   インストールスクリプト v1.0.0"
    echo -e "${NC}"
}

# ログ関数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 前提条件チェック
check_prerequisites() {
    log_info "前提条件をチェックしています..."
    
    # OS確認
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        OS="linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        OS="windows"
    else
        log_error "サポートされていないOS: $OSTYPE"
        exit 1
    fi
    log_info "OS: $OS"
    
    # WezTerm確認
    if ! command -v wezterm &> /dev/null; then
        log_error "WezTermがインストールされていません"
        echo "以下のコマンドでインストールしてください:"
        if [[ "$OS" == "macos" ]]; then
            echo "  brew install --cask wezterm"
        elif [[ "$OS" == "linux" ]]; then
            echo "  https://wezfurlong.org/wezterm/install/linux.html を参照"
        fi
        exit 1
    fi
    
    # Rust確認
    if ! command -v cargo &> /dev/null; then
        log_error "Rustがインストールされていません"
        echo "以下のコマンドでインストールしてください:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    # Git確認
    if ! command -v git &> /dev/null; then
        log_error "Gitがインストールされていません"
        exit 1
    fi
    
    log_info "前提条件チェック完了"
}

# ディレクトリ作成
create_directories() {
    log_info "必要なディレクトリを作成しています..."
    
    mkdir -p ~/.config/wezterm
    mkdir -p ~/.config/wezterm-multi-dev
    mkdir -p ~/.local/share/wezterm-multi-dev/logs
    mkdir -p ~/.local/share/wezterm-multi-dev/backups
    mkdir -p ~/.local/share/wezterm-multi-dev/data
    
    log_info "ディレクトリ作成完了"
}

# Rustプロジェクトビルド
build_project() {
    log_info "Rustプロジェクトをビルドしています..."
    
    # 依存関係チェック
    if ! cargo check; then
        log_error "依存関係の問題があります"
        exit 1
    fi
    
    # リリースビルド
    if ! cargo build --release; then
        log_error "ビルドに失敗しました"
        exit 1
    fi
    
    log_info "ビルド完了"
}

# 設定ファイルセットアップ
setup_config() {
    log_info "設定ファイルをセットアップしています..."
    
    # WezTerm設定
    if [[ ! -f ~/.config/wezterm/wezterm.lua ]]; then
        cp config/templates/wezterm.lua ~/.config/wezterm/
        log_info "WezTerm設定ファイルをコピーしました"
    else
        log_warn "WezTerm設定ファイルが既に存在します (スキップ)"
        cp config/templates/wezterm.lua ~/.config/wezterm/wezterm.lua.new
        log_info "新しい設定ファイルを wezterm.lua.new として保存しました"
    fi
    
    # フレームワーク設定
    if [[ ! -f ~/.config/wezterm-multi-dev/config.yaml ]]; then
        cp config/templates/framework.yaml ~/.config/wezterm-multi-dev/config.yaml
        log_info "フレームワーク設定ファイルをコピーしました"
    else
        log_warn "フレームワーク設定ファイルが既に存在します (スキップ)"
    fi
    
    # Luaモジュールのシンボリックリンク
    if [[ ! -L ~/.config/wezterm-multi-dev/lua ]]; then
        ln -sf "$(pwd)/wezterm-config" ~/.config/wezterm-multi-dev/lua
        log_info "Luaモジュールのシンボリックリンクを作成しました"
    fi
}

# バイナリのインストール
install_binary() {
    log_info "バイナリをインストールしています..."
    
    local install_dir="/usr/local/bin"
    
    # インストールディレクトリが存在しない場合
    if [[ ! -d "$install_dir" ]]; then
        install_dir="$HOME/.local/bin"
        mkdir -p "$install_dir"
    fi
    
    # バイナリをコピー
    if [[ -w "$install_dir" ]]; then
        cp target/release/wezterm-multi-dev "$install_dir/"
        chmod +x "$install_dir/wezterm-multi-dev"
        log_info "バイナリを $install_dir にインストールしました"
    else
        log_warn "管理者権限が必要です"
        if sudo cp target/release/wezterm-multi-dev "$install_dir/"; then
            sudo chmod +x "$install_dir/wezterm-multi-dev"
            log_info "バイナリを $install_dir にインストールしました"
        else
            log_error "バイナリのインストールに失敗しました"
            exit 1
        fi
    fi
    
    # PATHの確認
    if ! echo "$PATH" | grep -q "$install_dir"; then
        log_warn "$install_dir がPATHに含まれていません"
        echo "以下をシェル設定ファイル (~/.bashrc, ~/.zshrc等) に追加してください:"
        echo "export PATH=\"\$PATH:$install_dir\""
    fi
}

# システムサービスセットアップ
setup_service() {
    log_info "システムサービスをセットアップしています..."
    
    if [[ "$OS" == "linux" ]]; then
        # systemd サービス
        local service_file="/etc/systemd/system/wezterm-multi-dev.service"
        
        if [[ -f "config/systemd/wezterm-multi-dev.service" ]]; then
            if sudo cp config/systemd/wezterm-multi-dev.service "$service_file"; then
                sudo systemctl daemon-reload
                sudo systemctl enable wezterm-multi-dev
                log_info "systemdサービスを設定しました"
                log_info "以下のコマンドで開始できます:"
                log_info "  sudo systemctl start wezterm-multi-dev"
            else
                log_warn "systemdサービスの設定に失敗しました"
            fi
        fi
        
    elif [[ "$OS" == "macos" ]]; then
        # LaunchAgent
        local plist_file="$HOME/Library/LaunchAgents/com.wezterm-multi-dev.plist"
        
        if [[ -f "config/launchd/com.wezterm-multi-dev.plist" ]]; then
            cp config/launchd/com.wezterm-multi-dev.plist "$plist_file"
            
            # パスを現在のパスに更新
            sed -i '' "s|/path/to/wezterm-multi-dev|$(pwd)/target/release/wezterm-multi-dev|g" "$plist_file"
            
            if launchctl load "$plist_file"; then
                log_info "LaunchAgentを設定しました"
            else
                log_warn "LaunchAgentの設定に失敗しました"
            fi
        fi
    fi
}

# インストール検証
verify_installation() {
    log_info "インストールを検証しています..."
    
    # バイナリの確認
    if command -v wezterm-multi-dev &> /dev/null; then
        local version=$(wezterm-multi-dev --version 2>/dev/null || echo "unknown")
        log_info "wezterm-multi-dev バージョン: $version"
    else
        log_error "wezterm-multi-dev コマンドが見つかりません"
        return 1
    fi
    
    # 設定ファイルの確認
    if [[ -f ~/.config/wezterm-multi-dev/config.yaml ]]; then
        log_info "フレームワーク設定ファイル: OK"
    else
        log_error "フレームワーク設定ファイルが見つかりません"
        return 1
    fi
    
    # WezTerm設定の確認
    if [[ -f ~/.config/wezterm/wezterm.lua ]]; then
        log_info "WezTerm設定ファイル: OK"
    else
        log_error "WezTerm設定ファイルが見つかりません"
        return 1
    fi
    
    # 設定ファイルの構文チェック
    if wezterm-multi-dev --check-config &> /dev/null; then
        log_info "設定ファイル構文: OK"
    else
        log_warn "設定ファイルに問題がある可能性があります"
    fi
    
    log_info "インストール検証完了"
}

# 使用方法の表示
show_usage() {
    echo -e "${GREEN}"
    echo "🎉 インストールが完了しました！"
    echo -e "${NC}"
    echo
    echo "次のステップ:"
    echo "1. 新しいターミナルセッションを開始するか、以下を実行してください:"
    echo "   source ~/.bashrc  # または ~/.zshrc"
    echo
    echo "2. フレームワークサービスを開始:"
    echo "   wezterm-multi-dev"
    echo
    echo "3. WezTermを起動:"
    echo "   wezterm"
    echo
    echo "4. フレームワーク機能を使用:"
    echo "   Ctrl+Shift+D : ダッシュボード表示"
    echo "   Ctrl+Shift+P : ペイン管理"
    echo "   Ctrl+Shift+T : タスク管理"
    echo "   Ctrl+Shift+H : ヘルプ表示"
    echo
    echo "詳細なドキュメント:"
    echo "   https://github.com/your-org/wezterm-parallel/docs"
    echo
    echo "問題がある場合:"
    echo "   https://github.com/your-org/wezterm-parallel/issues"
}

# メイン処理
main() {
    print_logo
    
    echo "このスクリプトはWezTerm Multi-Process Development Frameworkを"
    echo "自動的にインストール・セットアップします。"
    echo
    
    read -p "続行しますか？ (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "インストールをキャンセルしました"
        exit 0
    fi
    
    echo
    
    # インストール手順
    check_prerequisites
    create_directories
    build_project
    setup_config
    install_binary
    setup_service
    
    if verify_installation; then
        show_usage
    else
        log_error "インストールの検証に失敗しました"
        echo "トラブルシューティングガイドを確認してください:"
        echo "docs/troubleshooting.md"
        exit 1
    fi
}

# スクリプト実行
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi