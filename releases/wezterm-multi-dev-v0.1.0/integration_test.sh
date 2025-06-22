#!/bin/bash

# WezTerm Multi-Process Development Framework Integration Test
# 統合テストスクリプト

echo "🧪 WezTerm Multi-Process Development Framework Integration Test"
echo "=============================================================="

# テスト結果カウンタ
TESTS_PASSED=0
TESTS_TOTAL=0

# テスト関数
run_test() {
    local test_name="$1"
    local test_command="$2"
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    
    echo -n "Testing $test_name... "
    if eval "$test_command" > /dev/null 2>&1; then
        echo "✅ PASSED"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo "❌ FAILED"
    fi
}

echo
echo "📋 Environment Check"
echo "--------------------"

# 1. WezTerm インストール確認
run_test "WezTerm installation" "which wezterm"

# 2. Rust ツールチェーン確認
run_test "Rust toolchain" "which cargo"

# 3. Lua インタープリター確認
run_test "Lua interpreter" "which lua"

echo
echo "🏗️  Build Tests"
echo "---------------"

# 4. Cargo ビルドテスト
run_test "Cargo build" "cargo build --release --quiet"

# 5. テスト実行
run_test "Unit tests" "cargo test --quiet"

echo
echo "📁 File Structure Tests"
echo "----------------------"

# 6. フレームワークファイル存在確認
run_test "Lua config files" "test -f lua/config/init.lua"
run_test "Dashboard UI files" "test -f lua/ui/dashboard.lua"
run_test "Workspace manager" "test -f lua/workspace/manager.lua"
run_test "Framework config" "test -f config/templates/framework.yaml"
run_test "WezTerm template" "test -f config/templates/wezterm.lua"

echo
echo "⚙️  Runtime Tests"
echo "-----------------"

# 7. バイナリ実行可能性
if [ -x "target/release/wezterm-multi-dev" ]; then
    run_test "Binary executable" "test -x target/release/wezterm-multi-dev"
elif [ -x "bin/wezterm-multi-dev" ]; then
    run_test "Binary executable" "test -x bin/wezterm-multi-dev"
else
    echo "Testing Binary executable... ❌ FAILED (binary not found)"
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
fi

# 8. 設定ファイル構文チェック
run_test "WezTerm config syntax" "wezterm --config-file test_wezterm_config.lua --help"

# 9. YAML設定ファイル構文チェック
if command -v yq > /dev/null 2>&1; then
    run_test "YAML config syntax" "yq eval '.' config/templates/framework.yaml"
else
    echo "Testing YAML config syntax... ⚠️  SKIPPED (yq not available)"
fi

echo
echo "🔌 IPC Communication Tests"
echo "--------------------------"

# 10. UDSソケット作成テスト
if [ -x "target/release/wezterm-multi-dev" ]; then
    BINARY_PATH="./target/release/wezterm-multi-dev"
elif [ -x "bin/wezterm-multi-dev" ]; then
    BINARY_PATH="./bin/wezterm-multi-dev"
else
    BINARY_PATH=""
fi

if [ -n "$BINARY_PATH" ]; then
    timeout 5s $BINARY_PATH > /tmp/test_server.log 2>&1 &
    SERVER_PID=$!
else
    SERVER_PID=""
fi
sleep 2

if test -S /tmp/wezterm-multi-dev.sock; then
    echo "Testing IPC socket creation... ✅ PASSED"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo "Testing IPC socket creation... ❌ FAILED"
fi
TESTS_TOTAL=$((TESTS_TOTAL + 1))

# サーバー停止
if [ -n "$SERVER_PID" ]; then
    kill $SERVER_PID > /dev/null 2>&1
fi
rm -f /tmp/wezterm-multi-dev.sock /tmp/test_server.log

echo
echo "📊 Test Summary"
echo "==============="
echo "Tests Passed: $TESTS_PASSED/$TESTS_TOTAL"

if [ $TESTS_PASSED -eq $TESTS_TOTAL ]; then
    echo "🎉 All tests passed! Framework is ready for use."
    exit 0
else
    echo "⚠️  Some tests failed. Please check the issues above."
    exit 1
fi