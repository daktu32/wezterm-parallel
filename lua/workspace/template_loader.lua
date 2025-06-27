-- WezTerm Multi-Process Development Framework - Template Loader
-- YAMLテンプレートの読み込み、解析、バリデーション機能

local wezterm = require 'wezterm'

local TemplateLoader = {}

-- テンプレートローダー設定
local config = {
  template_directories = {
    "config/templates/layouts",
    "~/.config/wezterm-parallel/templates",
    "~/.local/share/wezterm-parallel/templates"
  },
  cache_enabled = true,
  cache_ttl = 300, -- 5分
  supported_versions = {"1.0.0", "1.1.0"}
}

-- テンプレートキャッシュ
local template_cache = {}
local cache_timestamps = {}

-- 初期化
function TemplateLoader.init(framework_config)
  if framework_config and framework_config.template_loader then
    for k, v in pairs(framework_config.template_loader) do
      config[k] = v
    end
  end
  
  -- テンプレートディレクトリの展開
  for i, dir in ipairs(config.template_directories) do
    if dir:match("^~") then
      config.template_directories[i] = os.getenv("HOME") .. dir:sub(2)
    end
  end
  
  wezterm.log_info("TemplateLoader initialized")
end

-- シンプルなYAML解析器（基本的なキー:値ペアのみ対応）
local function parse_yaml_simple(content)
  local result = {}
  local current_section = result
  local section_stack = {result}
  
  for line in content:gmatch("[^\r\n]+") do
    line = line:gsub("^%s+", "") -- 先頭の空白を削除
    
    if line == "" or line:match("^#") then
      -- 空行またはコメント行をスキップ
    elseif line:match(":$") then
      -- セクション開始
      local section_name = line:match("^([%w_]+):")
      if section_name then
        current_section[section_name] = {}
        current_section = current_section[section_name]
        table.insert(section_stack, current_section)
      end
    elseif line:match(":") then
      -- キー:値ペア
      local key, value = line:match("^([%w_]+):%s*(.*)$")
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
    elseif line:match("^%s*-%s") then
      -- 配列項目
      local value = line:match("^%s*-%s*(.*)$")
      if not current_section[1] then
        -- 配列に変換
        local old_section = {}
        for k, v in pairs(current_section) do
          old_section[k] = v
        end
        current_section = {}
        for k, v in pairs(old_section) do
          current_section[k] = v
        end
      end
      table.insert(current_section, value)
    end
  end
  
  return result
end

-- 改良されたYAML解析（配列とオブジェクトを正しく処理）
local function parse_yaml_advanced(content)
  local lines = {}
  
  -- 行を配列に分割
  for line in content:gmatch("[^\r\n]+") do
    table.insert(lines, line)
  end
  
  local function get_indent_level(line)
    local indent = line:match("^(%s*)")
    return #indent
  end
  
  local function parse_value(value_str)
    if value_str:match("^%d+$") then
      return tonumber(value_str)
    elseif value_str:match("^%d*%.%d+$") then
      return tonumber(value_str)
    elseif value_str:lower() == "true" then
      return true
    elseif value_str:lower() == "false" then
      return false
    elseif value_str:match('^".*"$') or value_str:match("^'.*'$") then
      return value_str:sub(2, -2)
    else
      return value_str
    end
  end
  
  local function parse_structure(start_idx, expected_indent)
    local result = {}
    local i = start_idx
    local is_array = false
    
    while i <= #lines do
      local line = lines[i]
      local content_line = line:gsub("^%s*", "")
      local indent = get_indent_level(line)
      
      -- 空行やコメントをスキップ
      if content_line == "" or content_line:match("^#") then
        i = i + 1
      -- インデントが予期されたレベルより小さい場合は終了
      elseif indent < expected_indent then
        break
      -- 正しいインデントレベル
      elseif indent == expected_indent then
        -- 配列項目の処理
        if content_line:match("^-%s") then
          is_array = true
          local item_content = content_line:match("^-%s*(.*)$")
          
          if item_content == "" then
            -- 複数行配列項目
            local next_i, nested_obj = parse_structure(i + 1, expected_indent + 2)
            table.insert(result, nested_obj)
            i = next_i
          elseif item_content:match("^([%w_%-]+):%s*(.*)$") then
            -- 配列項目の開始がキー:値ペア
            local key, value = item_content:match("^([%w_%-]+):%s*(.*)$")
            local item_obj = {}
            if value ~= "" then
              item_obj[key] = parse_value(value)
            end
            
            -- 続く行を解析して同じオブジェクトに追加
            local j = i + 1
            while j <= #lines do
              local next_line = lines[j]
              local next_indent = get_indent_level(next_line)
              local next_content = next_line:gsub("^%s*", "")
              
              if next_content == "" or next_content:match("^#") then
                j = j + 1
              elseif next_indent > expected_indent then
                -- より深いインデントの場合
                if next_content:match(":") then
                  local next_key, next_value = next_content:match("^([%w_%-]+):%s*(.*)$")
                  if next_key then
                    if next_value == "" then
                      -- ネストしたオブジェクト
                      local nested_i, nested_result = parse_structure(j + 1, next_indent + 2)
                      item_obj[next_key] = nested_result
                      j = nested_i
                    else
                      item_obj[next_key] = parse_value(next_value)
                      j = j + 1
                    end
                  else
                    j = j + 1
                  end
                else
                  j = j + 1
                end
              else
                -- インデントが同じか浅い場合は終了
                break
              end
            end
            
            table.insert(result, item_obj)
            i = j
          else
            -- 単一行配列項目
            if item_content:match(":") then
              -- インライン オブジェクト
              local inline_obj = {}
              for k, v in item_content:gmatch("([%w_%-]+):%s*([^,]+)") do
                inline_obj[k] = parse_value(v:gsub(",%s*$", ""))
              end
              table.insert(result, inline_obj)
            else
              table.insert(result, parse_value(item_content))
            end
            i = i + 1
          end
        -- キー:値ペアの処理
        elseif content_line:match(":") then
          local key, value = content_line:match("^([%w_%-]+):%s*(.*)$")
          if key then
            if value == "" then
              -- 複数行値
              local next_i, nested_obj = parse_structure(i + 1, expected_indent + 2)
              result[key] = nested_obj
              i = next_i
            else
              -- 単一行値
              result[key] = parse_value(value)
              i = i + 1
            end
          else
            i = i + 1
          end
        else
          i = i + 1
        end
      else
        -- インデントが深すぎる場合はスキップ（エラー処理）
        i = i + 1
      end
    end
    
    return i, result
  end
  
  local _, parsed = parse_structure(1, 0)
  return parsed
end

-- テンプレートファイルの読み込み
function TemplateLoader.load_template(file_path)
  if not file_path then
    wezterm.log_error("Template file path is required")
    return nil
  end
  
  -- キャッシュの確認
  if config.cache_enabled and template_cache[file_path] then
    local cached_time = cache_timestamps[file_path]
    if cached_time and (os.time() - cached_time) < config.cache_ttl then
      wezterm.log_info("Loading template from cache: " .. file_path)
      return template_cache[file_path]
    end
  end
  
  -- ファイルの存在確認
  local file = io.open(file_path, "r")
  if not file then
    wezterm.log_error("Template file not found: " .. file_path)
    return nil
  end
  
  local content = file:read("*all")
  file:close()
  
  if not content or content == "" then
    wezterm.log_error("Template file is empty: " .. file_path)
    return nil
  end
  
  -- YAML解析
  local success, template = pcall(parse_yaml_advanced, content)
  if not success then
    wezterm.log_error("Failed to parse YAML template: " .. file_path .. " - " .. tostring(template))
    return nil
  end
  
  -- バリデーション
  if not TemplateLoader.validate_template(template) then
    wezterm.log_error("Template validation failed: " .. file_path)
    return nil
  end
  
  -- キャッシュに保存
  if config.cache_enabled then
    template_cache[file_path] = template
    cache_timestamps[file_path] = os.time()
  end
  
  wezterm.log_info("Successfully loaded template: " .. file_path)
  return template
end

-- テンプレートの妥当性検証
function TemplateLoader.validate_template(template)
  if not template or type(template) ~= "table" then
    return false
  end
  
  -- 必須フィールドの確認
  local required_fields = {"name", "version", "layout"}
  for _, field in ipairs(required_fields) do
    if not template[field] then
      wezterm.log_error("Missing required field: " .. field)
      return false
    end
  end
  
  -- バージョンの確認
  if template.version then
    local version_supported = false
    for _, supported in ipairs(config.supported_versions) do
      if template.version == supported then
        version_supported = true
        break
      end
    end
    if not version_supported then
      wezterm.log_warn("Unsupported template version: " .. template.version)
    end
  end
  
  -- レイアウトの確認
  if template.layout then
    if not template.layout.panes or type(template.layout.panes) ~= "table" then
      wezterm.log_error("Layout must contain panes array")
      return false
    end
    
    -- 各ペインの確認
    for i, pane in ipairs(template.layout.panes) do
      if not pane.id then
        wezterm.log_error("Pane " .. i .. " missing id")
        return false
      end
      
      if pane.size and (type(pane.size) ~= "number" or pane.size <= 0) then
        wezterm.log_error("Pane " .. pane.id .. " has invalid size")
        return false
      end
      
      if pane.position and type(pane.position) ~= "table" then
        wezterm.log_error("Pane " .. pane.id .. " has invalid position")
        return false
      end
    end
  end
  
  return true
end

-- 利用可能なテンプレートの一覧取得
function TemplateLoader.list_templates()
  local templates = {}
  
  for _, dir in ipairs(config.template_directories) do
    local success, result = pcall(function()
      local handle = io.popen("find " .. wezterm.shell_quote_arg(dir) .. " -name '*.yaml' -o -name '*.yml' 2>/dev/null")
      if handle then
        for file in handle:lines() do
          local template = TemplateLoader.load_template(file)
          if template then
            table.insert(templates, {
              file_path = file,
              name = template.name,
              description = template.description,
              version = template.version,
              author = template.author
            })
          end
        end
        handle:close()
      end
    end)
    
    if not success then
      wezterm.log_warn("Failed to scan directory: " .. dir)
    end
  end
  
  return templates
end

-- テンプレートディレクトリの取得
function TemplateLoader.get_template_directories()
  return config.template_directories
end

-- テンプレートの事前読み込み
function TemplateLoader.preload_templates()
  wezterm.log_info("Preloading templates...")
  
  local templates = TemplateLoader.list_templates()
  local loaded_count = 0
  
  for _, template_info in ipairs(templates) do
    if TemplateLoader.load_template(template_info.file_path) then
      loaded_count = loaded_count + 1
    end
  end
  
  wezterm.log_info("Preloaded " .. loaded_count .. " templates")
  return loaded_count
end

-- テンプレートキャッシュの取得
function TemplateLoader.get_template_cache()
  return template_cache
end

-- テンプレートキャッシュのクリア
function TemplateLoader.clear_cache()
  template_cache = {}
  cache_timestamps = {}
  wezterm.log_info("Template cache cleared")
end

-- テンプレートの検索
function TemplateLoader.find_template(name_or_pattern)
  local templates = TemplateLoader.list_templates()
  local matches = {}
  
  for _, template in ipairs(templates) do
    if template.name == name_or_pattern or 
       template.name:match(name_or_pattern) or
       template.file_path:match(name_or_pattern) then
      table.insert(matches, template)
    end
  end
  
  return matches
end

-- テンプレートディレクトリの作成
function TemplateLoader.ensure_template_directories()
  for _, dir in ipairs(config.template_directories) do
    local expanded_dir = dir
    if dir:match("^~") then
      expanded_dir = os.getenv("HOME") .. dir:sub(2)
    end
    
    -- ディレクトリ作成の試行
    local success = pcall(function()
      os.execute("mkdir -p " .. wezterm.shell_quote_arg(expanded_dir))
    end)
    
    if success then
      wezterm.log_info("Ensured template directory: " .. expanded_dir)
    else
      wezterm.log_warn("Failed to create template directory: " .. expanded_dir)
    end
  end
end

-- テンプレートの保存
function TemplateLoader.save_template(template, file_path)
  if not TemplateLoader.validate_template(template) then
    wezterm.log_error("Cannot save invalid template")
    return false
  end
  
  -- 簡単なYAML出力（基本的な構造のみ）
  local function serialize_yaml(data, indent)
    local result = {}
    indent = indent or 0
    local spaces = string.rep(" ", indent)
    
    for key, value in pairs(data) do
      if type(value) == "table" then
        if #value > 0 then
          -- 配列
          table.insert(result, spaces .. key .. ":")
          for _, item in ipairs(value) do
            if type(item) == "table" then
              table.insert(result, spaces .. "  - " .. serialize_yaml(item, 0):gsub("\n", "\n" .. spaces .. "    "))
            else
              table.insert(result, spaces .. "  - " .. tostring(item))
            end
          end
        else
          -- オブジェクト
          table.insert(result, spaces .. key .. ":")
          table.insert(result, serialize_yaml(value, indent + 2))
        end
      else
        if type(value) == "string" then
          table.insert(result, spaces .. key .. ': "' .. value .. '"')
        else
          table.insert(result, spaces .. key .. ": " .. tostring(value))
        end
      end
    end
    
    return table.concat(result, "\n")
  end
  
  local yaml_content = serialize_yaml(template)
  
  local file = io.open(file_path, "w")
  if not file then
    wezterm.log_error("Cannot write to template file: " .. file_path)
    return false
  end
  
  file:write(yaml_content)
  file:close()
  
  wezterm.log_info("Template saved: " .. file_path)
  return true
end

-- 設定の取得
function TemplateLoader.get_config()
  return config
end

return TemplateLoader