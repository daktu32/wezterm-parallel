-- WezTerm Multi-Process Development Framework - Template Bridge
-- Rust TemplateEngine とのIPC通信ブリッジ

local wezterm = require 'wezterm'
local socket_client = require 'utils.socket_client'

local TemplateBridge = {}

-- 設定
local config = {
  socket_path = "/tmp/wezterm-parallel.sock",
  timeout = 5000,
  retry_attempts = 3,
  cache_enabled = true,
  cache_ttl = 300 -- 5分
}

-- キャッシュ
local cache = {
  templates = nil,
  last_update = 0
}

-- 初期化
function TemplateBridge.init(framework_config)
  if framework_config and framework_config.template_bridge then
    for k, v in pairs(framework_config.template_bridge) do
      config[k] = v
    end
  end
  
  wezterm.log_info("TemplateBridge initialized")
end

-- キャッシュ有効性チェック
function TemplateBridge.is_cache_valid()
  if not config.cache_enabled or not cache.templates then
    return false
  end
  
  local now = os.time()
  return (now - cache.last_update) < config.cache_ttl
end

-- キャッシュクリア
function TemplateBridge.clear_cache()
  cache.templates = nil
  cache.last_update = 0
end

-- テンプレート一覧取得（Rust TemplateEngineから）
function TemplateBridge.list_templates()
  -- キャッシュチェック
  if TemplateBridge.is_cache_valid() then
    return cache.templates
  end
  
  wezterm.log_info("Fetching template list from Rust backend")
  
  local message = {
    TemplateList = {}
  }
  
  local response = socket_client.send_message(message)
  if response and response.TemplateListResponse then
    local templates = response.TemplateListResponse.templates
    
    -- テンプレート情報を拡張
    for _, template in ipairs(templates) do
      template.file_path = "rust://builtin/" .. template.name
      template.is_builtin = true
      template.source = "rust_backend"
    end
    
    -- キャッシュ更新
    if config.cache_enabled then
      cache.templates = templates
      cache.last_update = os.time()
    end
    
    wezterm.log_info("Successfully fetched " .. #templates .. " templates from backend")
    return templates
  else
    wezterm.log_error("Failed to fetch templates from Rust backend")
    
    -- フォールバック: ローカルスタブを返す
    return TemplateBridge.get_fallback_templates()
  end
end

-- テンプレート詳細取得
function TemplateBridge.load_template(template_path)
  if not template_path or not template_path:match("^rust://") then
    wezterm.log_warn("Invalid template path for Rust backend: " .. (template_path or "nil"))
    return nil
  end
  
  local template_name = template_path:gsub("rust://builtin/", "")
  
  wezterm.log_info("Loading template from Rust backend: " .. template_name)
  
  local message = {
    TemplateGet = { name = template_name }
  }
  
  local response = socket_client.send_message(message)
  if response and response.TemplateGetResponse and response.TemplateGetResponse.template then
    local template_json = response.TemplateGetResponse.template
    
    -- JSONデコード
    local success, template = pcall(function()
      return wezterm.json_parse and wezterm.json_parse(template_json) or nil
    end)
    
    if success and template then
      wezterm.log_info("Successfully loaded template: " .. template_name)
      return template
    else
      wezterm.log_error("Failed to parse template JSON: " .. template_name)
    end
  else
    wezterm.log_error("Failed to load template from backend: " .. template_name)
  end
  
  return nil
end

-- テンプレート作成（将来実装）
function TemplateBridge.create_template(name, content)
  wezterm.log_info("Creating template: " .. name)
  
  local message = {
    TemplateCreate = { 
      name = name,
      content = content
    }
  }
  
  local response = socket_client.send_message(message)
  if response and response.TemplateCreateResponse then
    local result = response.TemplateCreateResponse
    if result.success then
      wezterm.log_info("Successfully created template: " .. name)
      TemplateBridge.clear_cache()
      return true
    else
      wezterm.log_error("Failed to create template: " .. (result.error or "unknown error"))
      return false, result.error
    end
  end
  
  return false, "Backend communication failed"
end

-- テンプレート削除（将来実装）
function TemplateBridge.delete_template(name)
  wezterm.log_info("Deleting template: " .. name)
  
  local message = {
    TemplateDelete = { name = name }
  }
  
  local response = socket_client.send_message(message)
  if response and response.TemplateDeleteResponse then
    local result = response.TemplateDeleteResponse
    if result.success then
      wezterm.log_info("Successfully deleted template: " .. name)
      TemplateBridge.clear_cache()
      return true
    else
      wezterm.log_error("Failed to delete template: " .. (result.error or "unknown error"))
      return false, result.error
    end
  end
  
  return false, "Backend communication failed"
end

-- テンプレートディレクトリ確保（Rust側で管理）
function TemplateBridge.ensure_template_directories()
  wezterm.log_info("Template directories managed by Rust backend")
  return true
end

-- フォールバックテンプレート
function TemplateBridge.get_fallback_templates()
  return {
    {
      name = "basic",
      description = "Basic single-process workspace",
      author = "System",
      version = "1.0",
      created_at = os.date("!%Y-%m-%dT%H:%M:%SZ"),
      layout_type = "Single",
      pane_count = 1,
      auto_start_processes = true,
      file_path = "rust://builtin/basic",
      is_builtin = true,
      source = "fallback"
    },
    {
      name = "web_dev",
      description = "Web development with frontend/backend separation",
      author = "System", 
      version = "1.0",
      created_at = os.date("!%Y-%m-%dT%H:%M:%SZ"),
      layout_type = "FourPaneGrid",
      pane_count = 4,
      auto_start_processes = true,
      file_path = "rust://builtin/web_dev",
      is_builtin = true,
      source = "fallback"
    }
  }
end

-- バックエンド接続チェック
function TemplateBridge.check_backend_connection()
  local message = { Ping = {} }
  local response = socket_client.send_message(message)
  return response and response.Pong ~= nil
end

-- 統計情報取得
function TemplateBridge.get_statistics()
  return {
    cache_enabled = config.cache_enabled,
    cache_valid = TemplateBridge.is_cache_valid(),
    last_update = cache.last_update,
    backend_connected = TemplateBridge.check_backend_connection()
  }
end

-- 設定取得
function TemplateBridge.get_config()
  return config
end

-- 設定更新
function TemplateBridge.update_config(new_config)
  if new_config then
    for k, v in pairs(new_config) do
      config[k] = v
    end
  end
end

return TemplateBridge