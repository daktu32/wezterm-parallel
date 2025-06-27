-- WezTerm テンプレート統合テスト
-- Issue #18 Phase 3 - 実用テンプレートとテンプレート管理機能のテスト

local wezterm = require 'wezterm'

-- テスト対象モジュールの読み込み
package.path = package.path .. ";lua/?.lua;lua/workspace/?.lua;lua/ui/?.lua"
local TemplateLoader = require('template_loader')
local LayoutEngine = require('layout_engine')

-- テストフレームワーク
local TestFramework = {}

function TestFramework.run_test(name, test_func)
  local success, result = pcall(test_func)
  if success then
    print("✓ PASS: " .. name)
    return true
  else
    print("✗ FAIL: " .. name .. " - " .. tostring(result))
    return false
  end
end

function TestFramework.assert_equal(actual, expected, message)
  if actual ~= expected then
    error((message or "Assertion failed") .. ": expected " .. tostring(expected) .. ", got " .. tostring(actual))
  end
end

function TestFramework.assert_not_nil(value, message)
  if value == nil then
    error(message or "Expected non-nil value")
  end
end

function TestFramework.assert_table(value, message)
  if type(value) ~= "table" then
    error((message or "Expected table") .. ", got " .. type(value))
  end
end

-- テストケース群
local tests = {}

-- テスト1: Web開発用テンプレートの検証
function tests.test_web_dev_template_structure()
  local template_path = "config/templates/layouts/web-dev.yaml"
  local template = TemplateLoader.load_template(template_path)
  
  TestFramework.assert_not_nil(template, "Web開発テンプレートの読み込み失敗")
  TestFramework.assert_equal(template.name, "Web Development Layout", "テンプレート名が不正")
  TestFramework.assert_table(template.layout, "レイアウト構造が不正")
  TestFramework.assert_table(template.layout.panes, "ペイン構造が不正")
  
  -- ペイン数の確認（Web開発用は4ペイン想定）
  TestFramework.assert_equal(#template.layout.panes, 4, "Web開発テンプレートのペイン数が不正")
  
  -- 各ペインのID確認
  local expected_panes = {"editor", "terminal", "browser", "dev_server"}
  for i, pane in ipairs(template.layout.panes) do
    TestFramework.assert_not_nil(pane.id, "ペイン" .. i .. "のIDが未設定")
  end
end

-- テスト2: Rust開発用テンプレートの検証
function tests.test_rust_dev_template_structure()
  local template_path = "config/templates/layouts/rust-dev.yaml"
  local template = TemplateLoader.load_template(template_path)
  
  TestFramework.assert_not_nil(template, "Rust開発テンプレートの読み込み失敗")
  TestFramework.assert_equal(template.name, "Rust Development Layout", "テンプレート名が不正")
  TestFramework.assert_table(template.layout.panes, "ペイン構造が不正")
  
  -- Rust特有のペイン構成の確認
  local has_cargo_watch = false
  local has_test_runner = false
  
  for _, pane in ipairs(template.layout.panes) do
    if pane.command and pane.command:match("cargo") then
      has_cargo_watch = true
    end
    if pane.id and pane.id:match("test") then
      has_test_runner = true
    end
  end
  
  TestFramework.assert_equal(has_cargo_watch, true, "Rust開発環境にCargoコマンドが含まれていない")
end

-- テスト3: リサーチ用テンプレートの検証
function tests.test_research_template_structure()
  local template_path = "config/templates/layouts/research.yaml"
  local template = TemplateLoader.load_template(template_path)
  
  TestFramework.assert_not_nil(template, "リサーチテンプレートの読み込み失敗")
  TestFramework.assert_equal(template.name, "Research Layout", "テンプレート名が不正")
  TestFramework.assert_table(template.layout.panes, "ペイン構造が不正")
  
  -- リサーチ用の特徴的なペイン（ノート、ブラウザ、ターミナル等）の確認
  local has_notes = false
  local has_browser = false
  
  for _, pane in ipairs(template.layout.panes) do
    if pane.id and (pane.id:match("note") or pane.id:match("document")) then
      has_notes = true
    end
    if pane.id and pane.id:match("browser") then
      has_browser = true
    end
  end
  
  TestFramework.assert_equal(has_notes, true, "リサーチ環境にノート機能が含まれていない")
end

-- テスト4: テンプレートからレイアウト生成の検証
function tests.test_template_to_layout_generation()
  local template_path = "config/templates/layouts/claude-dev.yaml"
  local template = TemplateLoader.load_template(template_path)
  
  TestFramework.assert_not_nil(template, "テンプレート読み込み失敗")
  
  -- レイアウトエンジンでレイアウト生成
  local layout = LayoutEngine.generate_from_template(template)
  
  TestFramework.assert_not_nil(layout, "レイアウト生成失敗")
  TestFramework.assert_table(layout, "生成されたレイアウトがテーブルでない")
  TestFramework.assert_equal(#layout, #template.layout.panes, "ペイン数が一致しない")
  
  -- 各ペインの必要な属性が設定されているか確認
  for i, pane in ipairs(layout) do
    TestFramework.assert_not_nil(pane.id, "ペイン" .. i .. "のIDが未設定")
    TestFramework.assert_not_nil(pane.size, "ペイン" .. i .. "のサイズが未設定")
  end
end

-- テスト5: レイアウトの妥当性検証
function tests.test_layout_validation()
  local template_path = "config/templates/layouts/claude-dev.yaml"
  local template = TemplateLoader.load_template(template_path)
  local layout = LayoutEngine.generate_from_template(template)
  
  -- レイアウトの妥当性検証
  local is_valid = LayoutEngine.validate_layout(layout)
  TestFramework.assert_equal(is_valid, true, "生成されたレイアウトが無効")
  
  -- 競合検出テスト
  local conflicts = LayoutEngine.detect_conflicts(layout)
  TestFramework.assert_equal(type(conflicts), "table", "競合検出結果がテーブルでない")
end

-- テスト6: テンプレート検索機能の検証
function tests.test_template_search()
  -- 利用可能なテンプレートの一覧取得
  local templates = TemplateLoader.list_templates()
  TestFramework.assert_table(templates, "テンプレート一覧が取得できない")
  
  -- 特定のテンプレートの検索
  local claude_templates = TemplateLoader.find_template("claude")
  TestFramework.assert_table(claude_templates, "テンプレート検索結果がテーブルでない")
  
  -- 最低1つのテンプレートが見つかることを期待
  TestFramework.assert_equal(#claude_templates > 0, true, "Claude関連テンプレートが見つからない")
end

-- テスト7: テンプレートキャッシュ機能の検証
function tests.test_template_caching()
  local template_path = "config/templates/layouts/claude-dev.yaml"
  
  -- キャッシュクリア
  TemplateLoader.clear_cache()
  
  -- 初回読み込み
  local template1 = TemplateLoader.load_template(template_path)
  TestFramework.assert_not_nil(template1, "初回テンプレート読み込み失敗")
  
  -- 2回目読み込み（キャッシュから）
  local template2 = TemplateLoader.load_template(template_path)
  TestFramework.assert_not_nil(template2, "キャッシュからのテンプレート読み込み失敗")
  
  -- キャッシュ状態の確認
  local cache = TemplateLoader.get_template_cache()
  TestFramework.assert_table(cache, "テンプレートキャッシュが取得できない")
end

-- メイン テスト実行
function main()
  print("=== WezTerm テンプレート統合テスト開始 ===")
  print()
  
  -- テンプレートローダー初期化
  TemplateLoader.init()
  LayoutEngine.init()
  
  local total_tests = 0
  local passed_tests = 0
  
  -- 全テストを実行
  for test_name, test_func in pairs(tests) do
    total_tests = total_tests + 1
    if TestFramework.run_test(test_name, test_func) then
      passed_tests = passed_tests + 1
    end
  end
  
  print()
  print("=== テスト結果 ===")
  print("実行: " .. total_tests)
  print("成功: " .. passed_tests)
  print("失敗: " .. (total_tests - passed_tests))
  
  if passed_tests == total_tests then
    print("✓ 全テスト成功!")
  else
    print("✗ 一部テストが失敗しました")
  end
  
  return passed_tests == total_tests
end

-- テスト実行（このスクリプトが直接実行された場合）
if not package.loaded['test_template_integration'] then
  main()
end

return {
  TestFramework = TestFramework,
  tests = tests,
  main = main
}