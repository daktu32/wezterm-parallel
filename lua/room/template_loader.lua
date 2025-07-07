-- WezTerm Multi-Process Development Framework - Template Manager
-- テンプレート管理統合システム（一覧表示、検索、削除、デフォルト設定）

local wezterm = require 'wezterm'
local LayoutEngine = require('ui.layout_engine')
local TemplateBridge = require('room.template_loader_bridge')

local TemplateManager = {}

-- テンプレートマネージャー設定
local config = {
  default_template = "claude-dev",
  favorites = {},
  recently_used = {},
  max_recent_items = 10,
  auto_cleanup = true,
  cleanup_interval = 86400, -- 24時間
  template_validation = true,
  performance_tracking = true
}

-- 内部状態
local template_cache = {}
local template_stats = {}
local last_cleanup = 0

-- 初期化
function TemplateManager.init(framework_config)
  if framework_config and framework_config.template_manager then
    for k, v in pairs(framework_config.template_manager) do
      config[k] = v
    end
  end
  
  -- レイアウトエンジンの初期化
  if LayoutEngine.init then
    LayoutEngine.init(framework_config)
  end
  
  -- テンプレートブリッジの初期化
  TemplateBridge.init(framework_config)
  
  -- テンプレートディレクトリの確保
  TemplateBridge.ensure_template_directories()
  
  -- 統計情報の初期化
  TemplateManager.init_statistics()
  
  wezterm.log_info("TemplateManager initialized")
end

-- 統計情報の初期化
function TemplateManager.init_statistics()
  template_stats = {
    total_templates = 0,
    successful_loads = 0,
    failed_loads = 0,
    usage_count = {},
    last_updated = os.time()
  }
end

-- 利用可能なテンプレートの一覧取得（強化版）
function TemplateManager.list_templates(options)
  options = options or {}
  local include_details = options.details or false
  local sort_by = options.sort_by or "name"  -- name, date, usage
  local filter = options.filter or nil
  
  local templates = TemplateBridge.list_templates()
  local enhanced_templates = {}
  
  -- テンプレート情報の強化
  for _, template_info in ipairs(templates) do
    local enhanced = {
      file_path = template_info.file_path,
      name = template_info.name,
      description = template_info.description,
      version = template_info.version,
      author = template_info.author,
      is_favorite = TemplateManager.is_favorite(template_info.name),
      usage_count = template_stats.usage_count[template_info.name] or 0,
      last_used = TemplateManager.get_last_used(template_info.name)
    }
    
    -- 詳細情報を含める場合
    if include_details then
      local template = TemplateBridge.load_template(template_info.file_path)
      if template then
        enhanced.pane_count = template.layout and template.layout.panes and #template.layout.panes or 0
        enhanced.workspace_config = template.workspace and true or false
        enhanced.keybindings_count = template.suggested_keybindings and #template.suggested_keybindings or 0
        enhanced.file_size = TemplateManager.get_file_size(template_info.file_path)
        enhanced.validation_result = TemplateManager.validate_template_quick(template)
      end
    end
    
    -- フィルタリング
    if not filter or TemplateManager.match_filter(enhanced, filter) then
      table.insert(enhanced_templates, enhanced)
    end
  end
  
  -- ソート
  table.sort(enhanced_templates, function(a, b)
    if sort_by == "name" then
      return a.name < b.name
    elseif sort_by == "date" then
      return (a.last_used or 0) > (b.last_used or 0)
    elseif sort_by == "usage" then
      return a.usage_count > b.usage_count
    else
      return a.name < b.name
    end
  end)
  
  -- 統計情報の更新
  template_stats.total_templates = #enhanced_templates
  template_stats.last_updated = os.time()
  
  return enhanced_templates
end

-- テンプレート検索（強化版）
function TemplateManager.search_templates(query, options)
  options = options or {}
  local case_sensitive = options.case_sensitive or false
  local search_fields = options.fields or {"name", "description", "author"}
  local exact_match = options.exact_match or false
  
  if not case_sensitive then
    query = string.lower(query)
  end
  
  local all_templates = TemplateManager.list_templates({details = true})
  local matches = {}
  
  for _, template in ipairs(all_templates) do
    local match_found = false
    
    for _, field in ipairs(search_fields) do
      local field_value = template[field]
      if field_value then
        if not case_sensitive then
          field_value = string.lower(field_value)
        end
        
        if exact_match then
          if field_value == query then
            match_found = true
            break
          end
        else
          if field_value:match(query) then
            match_found = true
            break
          end
        end
      end
    end
    
    if match_found then
      table.insert(matches, template)
    end
  end
  
  return matches
end

-- テンプレートの適用
function TemplateManager.apply_template(template_name, window, pane, options)
  options = options or {}
  local auto_start_processes = options.auto_start_processes ~= false
  local custom_working_dir = options.working_directory
  
  -- テンプレートの検索
  local template_matches = TemplateManager.search_templates(template_name, {exact_match = true})
  if #template_matches == 0 then
    wezterm.log_error("Template not found: " .. template_name)
    return false
  end
  
  local template_info = template_matches[1]
  local template = TemplateBridge.load_template(template_info.file_path)
  
  if not template then
    wezterm.log_error("Failed to load template: " .. template_name)
    return false
  end
  
  -- 使用統計の更新
  TemplateManager.update_usage_stats(template_name)
  
  -- レイアウト生成
  local layout = LayoutEngine.generate_from_template(template)
  if not layout then
    wezterm.log_error("Failed to generate layout from template: " .. template_name)
    return false
  end
  
  -- レイアウトの妥当性検証
  if not LayoutEngine.validate_layout(layout) then
    wezterm.log_error("Invalid layout generated from template: " .. template_name)
    return false
  end
  
  -- 作業ディレクトリのカスタマイズ
  if custom_working_dir then
    for _, pane_config in ipairs(layout) do
      pane_config.working_directory = custom_working_dir
    end
  end
  
  -- レイアウトの適用
  local success = TemplateManager.apply_layout(window, pane, layout)
  if not success then
    wezterm.log_error("Failed to apply layout: " .. template_name)
    return false
  end
  
  -- プロセス自動起動
  if auto_start_processes and template.workspace and template.workspace.processes then
    TemplateManager.start_workspace_processes(template.workspace.processes)
  end
  
  -- 最近使用したテンプレートに追加
  TemplateManager.add_to_recent(template_name)
  
  wezterm.log_info("Successfully applied template: " .. template_name)
  return true
end

-- レイアウトの適用（WezTerm API統合）
function TemplateManager.apply_layout(window, pane, layout)
  if not window or not pane or not layout then
    return false
  end
  
  -- 現在のペインから開始
  local current_pane = pane
  local created_panes = {}
  
  for i, pane_config in ipairs(layout) do
    if i == 1 then
      -- 最初のペインは既存のペインを使用
      created_panes[pane_config.id] = current_pane
      
      -- コマンドの実行
      if pane_config.command then
        current_pane:send_text(pane_config.command .. "\n")
      end
    else
      -- 新しいペインを作成
      local split_direction = LayoutEngine.determine_split_direction(pane_config.position)
      local new_pane = current_pane:split({direction = split_direction})
      
      if new_pane then
        created_panes[pane_config.id] = new_pane
        
        -- 作業ディレクトリの設定
        if pane_config.working_directory then
          new_pane:send_text("cd " .. wezterm.shell_quote_arg(pane_config.working_directory) .. "\n")
        end
        
        -- コマンドの実行
        if pane_config.command then
          new_pane:send_text(pane_config.command .. "\n")
        end
        
        current_pane = new_pane
      end
    end
  end
  
  return true
end

-- ワークスペースプロセスの起動
function TemplateManager.start_workspace_processes(processes)
  if not processes then
    return
  end
  
  for _, process in ipairs(processes) do
    if process.auto_start then
      wezterm.log_info("Starting workspace process: " .. (process.name or "unnamed"))
      -- 実際のプロセス起動ロジックはここに実装
      -- 例: workspace_manager.start_process(process)
    end
  end
end

-- お気に入りテンプレートの管理
function TemplateManager.add_to_favorites(template_name)
  if not TemplateManager.is_favorite(template_name) then
    table.insert(config.favorites, template_name)
    wezterm.log_info("Added to favorites: " .. template_name)
    return true
  end
  return false
end

function TemplateManager.remove_from_favorites(template_name)
  for i, favorite in ipairs(config.favorites) do
    if favorite == template_name then
      table.remove(config.favorites, i)
      wezterm.log_info("Removed from favorites: " .. template_name)
      return true
    end
  end
  return false
end

function TemplateManager.is_favorite(template_name)
  for _, favorite in ipairs(config.favorites) do
    if favorite == template_name then
      return true
    end
  end
  return false
end

function TemplateManager.get_favorites()
  return config.favorites
end

-- 最近使用したテンプレートの管理
function TemplateManager.add_to_recent(template_name)
  -- 既存のエントリを削除
  for i, recent in ipairs(config.recently_used) do
    if recent.name == template_name then
      table.remove(config.recently_used, i)
      break
    end
  end
  
  -- 先頭に追加
  table.insert(config.recently_used, 1, {
    name = template_name,
    timestamp = os.time()
  })
  
  -- 最大数を超えた場合は古いものを削除
  if #config.recently_used > config.max_recent_items then
    table.remove(config.recently_used)
  end
end

function TemplateManager.get_recent_templates()
  return config.recently_used
end

function TemplateManager.clear_recent_templates()
  config.recently_used = {}
end

-- 使用統計の管理
function TemplateManager.update_usage_stats(template_name)
  template_stats.usage_count[template_name] = (template_stats.usage_count[template_name] or 0) + 1
  template_stats.successful_loads = template_stats.successful_loads + 1
end

function TemplateManager.get_usage_stats()
  return template_stats
end

-- テンプレートの削除
function TemplateManager.delete_template(template_name)
  local template_matches = TemplateManager.search_templates(template_name, {exact_match = true})
  if #template_matches == 0 then
    wezterm.log_error("Template not found for deletion: " .. template_name)
    return false
  end
  
  local template_info = template_matches[1]
  local file_path = template_info.file_path
  
  -- ファイルの削除
  local success = pcall(function()
    os.remove(file_path)
  end)
  
  if success then
    -- キャッシュからも削除
    TemplateManager.clear_cache()
    
    -- お気に入りから削除
    TemplateManager.remove_from_favorites(template_name)
    
    -- 最近使用したリストから削除
    for i, recent in ipairs(config.recently_used) do
      if recent.name == template_name then
        table.remove(config.recently_used, i)
        break
      end
    end
    
    wezterm.log_info("Template deleted: " .. template_name)
    return true
  else
    wezterm.log_error("Failed to delete template file: " .. file_path)
    return false
  end
end

-- デフォルトテンプレートの管理
function TemplateManager.set_default_template(template_name)
  -- テンプレートの存在確認
  local template_matches = TemplateManager.search_templates(template_name, {exact_match = true})
  if #template_matches == 0 then
    wezterm.log_error("Template not found: " .. template_name)
    return false
  end
  
  config.default_template = template_name
  wezterm.log_info("Default template set to: " .. template_name)
  return true
end

function TemplateManager.get_default_template()
  return config.default_template
end

function TemplateManager.apply_default_template(window, pane)
  return TemplateManager.apply_template(config.default_template, window, pane)
end

-- テンプレートの妥当性検証（クイック版）
function TemplateManager.validate_template_quick(template)
  if not template then
    return {valid = false, error = "Template is nil"}
  end
  
  -- 基本的な妥当性検証
  local validation_result = {valid = true, warnings = {}, errors = {}}
  
  -- 必須フィールドの確認
  if not template.name then
    validation_result.valid = false
    table.insert(validation_result.errors, "Missing template name")
  end
  
  if not template.layout or not template.layout.panes then
    validation_result.valid = false
    table.insert(validation_result.errors, "Missing layout or panes")
  end
  
  -- ペインの検証
  if template.layout and template.layout.panes then
    for i, pane in ipairs(template.layout.panes) do
      if not pane.id then
        table.insert(validation_result.warnings, "Pane " .. i .. " missing ID")
      end
      
      if pane.size and (pane.size <= 0 or pane.size > 1) then
        table.insert(validation_result.warnings, "Pane " .. (pane.id or i) .. " has invalid size")
      end
    end
  end
  
  return validation_result
end

-- ファイルサイズの取得
function TemplateManager.get_file_size(file_path)
  local file = io.open(file_path, "r")
  if not file then
    return 0
  end
  
  local size = file:seek("end")
  file:close()
  return size or 0
end

-- 最後に使用した日時の取得
function TemplateManager.get_last_used(template_name)
  for _, recent in ipairs(config.recently_used) do
    if recent.name == template_name then
      return recent.timestamp
    end
  end
  return nil
end

-- フィルタマッチング
function TemplateManager.match_filter(template, filter)
  if not filter then
    return true
  end
  
  -- 名前でのフィルタリング
  if filter.name and not template.name:match(filter.name) then
    return false
  end
  
  -- 作者でのフィルタリング
  if filter.author and not (template.author and template.author:match(filter.author)) then
    return false
  end
  
  -- お気に入りでのフィルタリング
  if filter.favorites_only and not template.is_favorite then
    return false
  end
  
  return true
end

-- 自動クリーンアップ
function TemplateManager.auto_cleanup()
  if not config.auto_cleanup then
    return
  end
  
  local now = os.time()
  if now - last_cleanup < config.cleanup_interval then
    return
  end
  
  -- キャッシュクリア
  TemplateManager.clear_cache()
  
  -- 統計情報のクリーンアップ
  local cleaned_stats = {}
  local current_templates = TemplateManager.list_templates()
  
  for _, template in ipairs(current_templates) do
    if template_stats.usage_count[template.name] then
      cleaned_stats[template.name] = template_stats.usage_count[template.name]
    end
  end
  
  template_stats.usage_count = cleaned_stats
  last_cleanup = now
  
  wezterm.log_info("Template manager cleanup completed")
end

-- テンプレート情報の詳細表示
function TemplateManager.get_template_details(template_name)
  local template_matches = TemplateManager.search_templates(template_name, {exact_match = true})
  if #template_matches == 0 then
    return nil
  end
  
  local template_info = template_matches[1]
  local template = TemplateBridge.load_template(template_info.file_path)
  
  if not template then
    return nil
  end
  
  local details = {
    basic_info = template_info,
    layout_info = LayoutEngine.get_layout_info(LayoutEngine.generate_from_template(template)),
    validation_result = TemplateManager.validate_template_quick(template),
    file_size = TemplateManager.get_file_size(template_info.file_path),
    usage_stats = {
      usage_count = template_stats.usage_count[template_name] or 0,
      last_used = TemplateManager.get_last_used(template_name),
      is_favorite = TemplateManager.is_favorite(template_name)
    }
  }
  
  return details
end

-- 設定の取得
function TemplateManager.get_config()
  return config
end

-- 設定の更新
function TemplateManager.update_config(new_config)
  if new_config then
    for k, v in pairs(new_config) do
      config[k] = v
    end
  end
end

-- ブリッジ統合機能
function TemplateManager.clear_cache()
  -- キャッシュクリア（ブリッジに委譲）
  TemplateBridge.clear_cache()
  template_cache = {}
end

function TemplateManager.get_bridge_statistics()
  return TemplateBridge.get_statistics()
end

function TemplateManager.check_backend_connection()
  return TemplateBridge.check_backend_connection()
end

return TemplateManager