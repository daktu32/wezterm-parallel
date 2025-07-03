#!/bin/bash
# WezTermデバッグログを有効にして起動するスクリプト

echo "Starting WezTerm with debug logging enabled..."
echo "Logs will be displayed in the terminal"
echo ""

# デバッグログを標準エラー出力に表示
WEZTERM_LOG=debug wezterm