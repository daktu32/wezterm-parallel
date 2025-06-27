-- WezTerm テンプレート独立テスト
-- Issue #18 Phase 3 - 作成したテンプレートファイルの構造検証

-- 簡易YAML解析器
local function parse_yaml_basic(content)
  local result = {}
  local current_section = result
  
  for line in content:gmatch("[^\r\n]+") do
    line = line:gsub("^%s+", "") -- 先頭の空白を削除
    
    if line == "" or line:match("^#") then
      -- 空行またはコメント行をスキップ
    elseif line:match(":$") then
      -- セクション開始
      local section_name = line:match("^([%w_%-]+):")
      if section_name then
        current_section[section_name] = {}
        current_section = current_section[section_name]
      end
    elseif line:match(":") then
      -- キー:値ペア
      local key, value = line:match("^([%w_%-]+):%s*(.*)$")
      if key and value then
        -- 値の型を推定
        if value:match("^%d+$") then
          current_section[key] = tonumber(value)
        elseif value:match("^%d*%.%d+$") then
          current_section[key] = tonumber(value)
        elseif value:lower() == "true" then
          current_section[key] = true
        elseif value:lower() == "false" then
          current_section[key] = false
        elseif value:match('^".*"$') or value:match("^'.*'$") then
          current_section[key] = value:sub(2, -2) -- 引用符を削除
        else
          current_section[key] = value
        end
      end
    end
  end
  
  return result
end

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

-- テンプレートファイルの読み込み
local function load_template_file(file_path)
  local file = io.open(file_path, "r")
  if not file then
    return nil, "File not found: " .. file_path
  end
  
  local content = file:read("*all")
  file:close()
  
  if not content or content == "" then
    return nil, "File is empty: " .. file_path
  end
  
  local success, template = pcall(parse_yaml_basic, content)
  if not success then
    return nil, "Failed to parse YAML: " .. tostring(template)
  end
  
  return template, nil
end

-- テストケース群
local tests = {}

-- テスト1: Web開発用テンプレートの検証
function tests.test_web_dev_template()
  local template, err = load_template_file("config/templates/layouts/web-dev.yaml")
  TestFramework.assert_not_nil(template, "Web開発テンプレートの読み込み失敗: " .. (err or "unknown"))
  TestFramework.assert_equal(template.name, "Web Development Layout", "テンプレート名が不正")
  TestFramework.assert_equal(template.version, "1.0.0", "バージョンが不正")
  TestFramework.assert_not_nil(template.description, "説明が未設定")
  TestFramework.assert_table(template.layout, "レイアウト構造が不正")
end

-- テスト2: Rust開発用テンプレートの検証
function tests.test_rust_dev_template()
  local template, err = load_template_file("config/templates/layouts/rust-dev.yaml")
  TestFramework.assert_not_nil(template, "Rust開発テンプレートの読み込み失敗: " .. (err or "unknown"))
  TestFramework.assert_equal(template.name, "Rust Development Layout", "テンプレート名が不正")
  TestFramework.assert_equal(template.version, "1.0.0", "バージョンが不正")
  TestFramework.assert_not_nil(template.description, "説明が未設定")
  TestFramework.assert_table(template.layout, "レイアウト構造が不正")
end

-- テスト3: リサーチ用テンプレートの検証
function tests.test_research_template()
  local template, err = load_template_file("config/templates/layouts/research.yaml")
  TestFramework.assert_not_nil(template, "リサーチテンプレートの読み込み失敗: " .. (err or "unknown"))
  TestFramework.assert_equal(template.name, "Research Layout", "テンプレート名が不正")
  TestFramework.assert_equal(template.version, "1.0.0", "バージョンが不正")
  TestFramework.assert_not_nil(template.description, "説明が未設定")
  TestFramework.assert_table(template.layout, "レイアウト構造が不正")
end

-- テスト4: 既存のClaude開発テンプレートの検証
function tests.test_claude_dev_template()
  local template, err = load_template_file("config/templates/layouts/claude-dev.yaml")
  TestFramework.assert_not_nil(template, "Claude開発テンプレートの読み込み失敗: " .. (err or "unknown"))
  TestFramework.assert_not_nil(template.name, "テンプレート名が未設定")
  TestFramework.assert_not_nil(template.version, "バージョンが未設定")
end

-- テスト5: テンプレートの必須フィールド検証
function tests.test_template_required_fields()
  local templates = {
    {"config/templates/layouts/web-dev.yaml", "Web Development"},
    {"config/templates/layouts/rust-dev.yaml", "Rust Development"},
    {"config/templates/layouts/research.yaml", "Research"}
  }
  
  for _, template_info in ipairs(templates) do
    local file_path, expected_type = template_info[1], template_info[2]
    local template, err = load_template_file(file_path)
    
    TestFramework.assert_not_nil(template, expected_type .. " テンプレート読み込み失敗: " .. (err or "unknown"))
    
    -- 必須フィールドの確認
    TestFramework.assert_not_nil(template.name, expected_type .. " - nameフィールドが未設定")
    TestFramework.assert_not_nil(template.version, expected_type .. " - versionフィールドが未設定")
    TestFramework.assert_not_nil(template.description, expected_type .. " - descriptionフィールドが未設定")
    TestFramework.assert_not_nil(template.author, expected_type .. " - authorフィールドが未設定")
    TestFramework.assert_not_nil(template.created, expected_type .. " - createdフィールドが未設定")
    
    -- レイアウト構造の確認
    TestFramework.assert_table(template.layout, expected_type .. " - layout構造が不正")
    
    -- ワークスペース設定の確認
    TestFramework.assert_table(template.workspace, expected_type .. " - workspace設定が不正")
    
    -- オプション設定の確認
    TestFramework.assert_table(template.options, expected_type .. " - options設定が不正")
  end
end

-- テスト6: ファイルサイズと構造の妥当性
function tests.test_template_file_structure()
  local template_files = {
    "config/templates/layouts/web-dev.yaml",
    "config/templates/layouts/rust-dev.yaml", 
    "config/templates/layouts/research.yaml"
  }
  
  for _, file_path in ipairs(template_files) do
    -- ファイルサイズの確認
    local file = io.open(file_path, "r")
    TestFramework.assert_not_nil(file, "ファイルが開けません: " .. file_path)
    
    local size = file:seek("end")
    file:close()
    
    -- 最小サイズ（500バイト）以上であることを確認
    TestFramework.assert_equal(size > 500, true, file_path .. " - ファイルサイズが小さすぎます")
    
    -- 最大サイズ（50KB）以下であることを確認
    TestFramework.assert_equal(size < 50000, true, file_path .. " - ファイルサイズが大きすぎます")
  end
end

-- メイン テスト実行
function main()
  print("=== WezTerm テンプレート独立テスト開始 ===")
  print()
  
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
    return true
  else
    print("✗ 一部テストが失敗しました")
    return false
  end
end

-- テスト実行
return main()