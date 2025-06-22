#!/bin/bash

# WezTerm Multi-Process Development Framework Installer
# インストールスクリプト v1.0.0

set -e

echo "🚀 WezTerm Multi-Process Development Framework Installer v1.0.0"
echo "=================================================================="

# カラー定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ログ関数
info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; exit 1; }

# 設定
FRAMEWORK_NAME="wezterm-multi-dev"
INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/wezterm-multi-dev"
WEZTERM_CONFIG_DIR="$HOME/.config/wezterm"

echo
info "インストール先ディレクトリの確認と作成..."

# ディレクトリ作成
mkdir -p "$INSTALL_DIR"
mkdir -p "$CONFIG_DIR"
mkdir -p "$CONFIG_DIR/lua"
mkdir -p "$CONFIG_DIR/logs"
mkdir -p "$CONFIG_DIR/templates"
mkdir -p "$WEZTERM_CONFIG_DIR"

success "ディレクトリ作成完了"

echo
info "前提条件の確認..."

# WezTerm確認
if ! command -v wezterm &> /dev/null; then
    warning "WezTermがインストールされていません"
    info "https://wezfurlong.org/wezterm/ からインストールしてください"
else
    success "WezTerm: $(wezterm --version)"
fi

# Rust確認（オプション）
if command -v cargo &> /dev/null; then
    success "Rust: $(rustc --version)"
else
    warning "Rust未インストール（ソースからビルドする場合に必要）"
fi

echo
info "バイナリファイルのインストール..."

# バイナリコピー
if [ -f "target/release/$FRAMEWORK_NAME" ]; then
    cp "target/release/$FRAMEWORK_NAME" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$FRAMEWORK_NAME"
    success "バイナリファイルをインストール: $INSTALL_DIR/$FRAMEWORK_NAME"
else
    error "リリースバイナリが見つかりません。先に 'cargo build --release' を実行してください"
fi

echo
info "設定ファイルのインストール..."

# 設定ファイルコピー
if [ -f "config/templates/framework.yaml" ]; then
    cp "config/templates/framework.yaml" "$CONFIG_DIR/config.yaml"
    success "フレームワーク設定ファイルをインストール"
else
    warning "設定テンプレートが見つかりません"
fi

# Luaファイルコピー
if [ -d "lua" ]; then
    cp -r lua/* "$CONFIG_DIR/lua/"
    success "Luaモジュールをインストール"
else
    warning "Luaモジュールが見つかりません"
fi

# WezTerm設定テンプレート
if [ -f "config/templates/wezterm.lua" ]; then
    if [ ! -f "$WEZTERM_CONFIG_DIR/wezterm.lua" ]; then
        cp "config/templates/wezterm.lua" "$WEZTERM_CONFIG_DIR/wezterm.lua"
        success "WezTerm設定テンプレートをインストール"
    else
        cp "config/templates/wezterm.lua" "$WEZTERM_CONFIG_DIR/wezterm-multi-dev.lua"
        warning "既存のWezTerm設定を保護し、テンプレートを wezterm-multi-dev.lua として保存"
    fi
fi

echo
info "統合テストスクリプトのインストール..."

if [ -f "integration_test.sh" ]; then
    cp "integration_test.sh" "$CONFIG_DIR/"
    chmod +x "$CONFIG_DIR/integration_test.sh"
    success "統合テストスクリプトをインストール"
fi

echo
info "PATH環境変数の確認..."

# PATH確認
if [[ ":$PATH:" == *":$INSTALL_DIR:"* ]]; then
    success "PATH環境変数に $INSTALL_DIR が含まれています"
else
    warning "PATH環境変数に $INSTALL_DIR が含まれていません"
    info "以下のコマンドをシェル設定ファイル（~/.bashrc, ~/.zshrc等）に追加してください："
    echo "export PATH=\"\$PATH:$INSTALL_DIR\""
fi

echo
info "インストール後の確認..."

# インストール確認
if [ -x "$INSTALL_DIR/$FRAMEWORK_NAME" ]; then
    success "バイナリ実行可能: $($INSTALL_DIR/$FRAMEWORK_NAME --version 2>/dev/null || echo 'インストール完了')"
else
    error "バイナリが正しくインストールされていません"
fi

echo
success "🎉 インストール完了！"
echo
echo "📋 次のステップ:"
echo "1. WezTermを起動"
echo "2. 以下のコマンドでフレームワークを開始:"
echo "   $FRAMEWORK_NAME"
echo ""
echo "3. または、統合テストを実行:"
echo "   $CONFIG_DIR/integration_test.sh"
echo ""
echo "📁 インストール場所:"
echo "   バイナリ: $INSTALL_DIR/$FRAMEWORK_NAME"
echo "   設定: $CONFIG_DIR/"
echo "   Lua: $CONFIG_DIR/lua/"
echo ""
echo "📖 詳細は README.md をご覧ください"