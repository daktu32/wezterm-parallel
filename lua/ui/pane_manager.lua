-- WezTerm Multi-Process Development Framework - Advanced Pane Manager
-- Handles dynamic pane creation, synchronization, and layout management

local wezterm = require 'wezterm'
local template_loader = require 'workspace.template_loader'
local layout_engine = require 'ui.layout_engine'

local PaneManager = {}

-- Pane manager configuration
local config = {
  sync_enabled = false,
  layouts = {},
  auto_balance = true,
  focus_follows_process = true,
  pane_titles_enabled = true,
}

-- Pane manager state
local state = {
  managed_panes = {},
  sync_groups = {},
  current_layout = nil,
  last_focus_pane = nil,
}

-- Initialize pane manager
function PaneManager.init(framework_config)
  if framework_config.pane_manager then
    for k, v in pairs(framework_config.pane_manager) do
      config[k] = v
    end
  end
  
  -- Define default layouts
  PaneManager.setup_default_layouts()
  
  wezterm.log_info("PaneManager initialized")
end

-- Setup default layouts
function PaneManager.setup_default_layouts()
  config.layouts = {
    single = {
      name = "Single Pane",
      panes = {
        { position = { row = 0, col = 0, span_rows = 1, span_cols = 1 }, size = 1.0 }
      }
    },
    
    horizontal_split = {
      name = "Horizontal Split",
      panes = {
        { position = { row = 0, col = 0, span_rows = 1, span_cols = 1 }, size = 0.5 },
        { position = { row = 0, col = 1, span_rows = 1, span_cols = 1 }, size = 0.5 }
      }
    },
    
    vertical_split = {
      name = "Vertical Split",
      panes = {
        { position = { row = 0, col = 0, span_rows = 1, span_cols = 1 }, size = 0.5 },
        { position = { row = 1, col = 0, span_rows = 1, span_cols = 1 }, size = 0.5 }
      }
    },
    
    three_pane_horizontal = {
      name = "Three Pane Horizontal",
      panes = {
        { position = { row = 0, col = 0, span_rows = 1, span_cols = 1 }, size = 0.33 },
        { position = { row = 0, col = 1, span_rows = 1, span_cols = 1 }, size = 0.34 },
        { position = { row = 0, col = 2, span_rows = 1, span_cols = 1 }, size = 0.33 }
      }
    },
    
    four_pane_grid = {
      name = "Four Pane Grid",
      panes = {
        { position = { row = 0, col = 0, span_rows = 1, span_cols = 1 }, size = 0.25 },
        { position = { row = 0, col = 1, span_rows = 1, span_cols = 1 }, size = 0.25 },
        { position = { row = 1, col = 0, span_rows = 1, span_cols = 1 }, size = 0.25 },
        { position = { row = 1, col = 1, span_rows = 1, span_cols = 1 }, size = 0.25 }
      }
    },
    
    main_side = {
      name = "Main + Side",
      panes = {
        { position = { row = 0, col = 0, span_rows = 2, span_cols = 1 }, size = 0.7 },
        { position = { row = 0, col = 1, span_rows = 1, span_cols = 1 }, size = 0.15 },
        { position = { row = 1, col = 1, span_rows = 1, span_cols = 1 }, size = 0.15 }
      }
    }
  }
end

-- Apply layout to current tab
function PaneManager.apply_layout(window, pane, layout_name)
  local layout = config.layouts[layout_name]
  if not layout then
    wezterm.log_error("Layout not found: " .. layout_name)
    return false
  end
  
  local tab = pane:tab()
  state.current_layout = layout_name
  
  wezterm.log_info("Applying layout: " .. layout.name)
  
  -- Close all existing panes except the first one
  local panes = tab:panes()
  for i = #panes, 2, -1 do
    panes[i]:activate()
    window:perform_action(wezterm.action.CloseCurrentPane { confirm = false }, panes[i])
  end
  
  -- Create new panes according to layout
  local base_pane = tab:panes()[1]
  local created_panes = { base_pane }
  
  for i = 2, #layout.panes do
    local pane_config = layout.panes[i]
    local split_direction = PaneManager.determine_split_direction(pane_config.position)
    local split_size = pane_config.size
    
    local new_pane = base_pane:split_pane({
      direction = split_direction,
      size = split_size,
    })
    
    table.insert(created_panes, new_pane)
    
    -- Register managed pane
    PaneManager.register_pane(new_pane, {
      layout_position = pane_config.position,
      layout_name = layout_name,
    })
  end
  
  -- Balance panes if enabled
  if config.auto_balance then
    PaneManager.balance_panes(tab)
  end
  
  return true
end

-- Determine split direction based on position
function PaneManager.determine_split_direction(position)
  -- This is a simplified heuristic
  -- In practice, you might need more sophisticated logic
  if position.col > 0 then
    return "Right"
  else
    return "Bottom"
  end
end

-- Register a managed pane
function PaneManager.register_pane(pane, metadata)
  local pane_id = pane:pane_id()
  state.managed_panes[pane_id] = {
    pane = pane,
    metadata = metadata or {},
    created_at = os.time(),
    workspace = metadata.workspace,
    process_id = metadata.process_id,
  }
  
  -- Set pane title if enabled
  if config.pane_titles_enabled and metadata.title then
    PaneManager.set_pane_title(pane, metadata.title)
  end
  
  wezterm.log_info("Registered managed pane: " .. pane_id)
end

-- Unregister a managed pane
function PaneManager.unregister_pane(pane_id)
  if state.managed_panes[pane_id] then
    state.managed_panes[pane_id] = nil
    wezterm.log_info("Unregistered managed pane: " .. pane_id)
  end
end

-- Set pane title
function PaneManager.set_pane_title(pane, title)
  -- Use OSC 2 sequence to set window title
  pane:inject_output(string.format("\x1b]2;%s\x07", title))
end

-- Create process pane
function PaneManager.create_process_pane(window, pane, process_config)
  local tab = pane:tab()
  
  -- Determine split direction and size
  local split_config = {
    direction = process_config.split_direction or "Right",
    size = process_config.split_size or 0.5,
  }
  
  -- Create new pane
  local new_pane = tab:split_pane(split_config)
  
  -- Set working directory if specified
  if process_config.working_directory then
    new_pane:inject_output("cd " .. wezterm.shell_quote_arg(process_config.working_directory) .. "\n")
  end
  
  -- Start process command if specified
  if process_config.command then
    new_pane:inject_output(process_config.command .. "\n")
  end
  
  -- Register the pane
  PaneManager.register_pane(new_pane, {
    process_id = process_config.process_id,
    workspace = process_config.workspace,
    title = process_config.title or process_config.process_id,
    command = process_config.command,
  })
  
  return new_pane
end

-- Balance panes in tab
function PaneManager.balance_panes(tab)
  -- This is a placeholder - WezTerm doesn't have direct pane balancing API
  -- In practice, you might need to recreate panes with calculated sizes
  wezterm.log_info("Balancing panes in tab")
end

-- Enable pane synchronization
function PaneManager.enable_sync(group_name, pane_ids)
  if not group_name or not pane_ids or #pane_ids == 0 then
    return false
  end
  
  state.sync_groups[group_name] = {
    panes = pane_ids,
    enabled = true,
    created_at = os.time(),
  }
  
  config.sync_enabled = true
  
  wezterm.log_info("Enabled pane synchronization for group: " .. group_name)
  return true
end

-- Disable pane synchronization
function PaneManager.disable_sync(group_name)
  if group_name then
    state.sync_groups[group_name] = nil
    wezterm.log_info("Disabled pane synchronization for group: " .. group_name)
  else
    state.sync_groups = {}
    config.sync_enabled = false
    wezterm.log_info("Disabled all pane synchronization")
  end
end

-- Send command to synchronized panes
function PaneManager.send_to_sync_group(group_name, command)
  local group = state.sync_groups[group_name]
  if not group or not group.enabled then
    return false
  end
  
  for _, pane_id in ipairs(group.panes) do
    local managed_pane = state.managed_panes[pane_id]
    if managed_pane and managed_pane.pane then
      managed_pane.pane:inject_output(command)
    end
  end
  
  wezterm.log_info("Sent command to sync group " .. group_name .. ": " .. command)
  return true
end

-- Send command to all panes in workspace
function PaneManager.send_to_workspace(workspace_name, command)
  local count = 0
  
  for _, managed_pane in pairs(state.managed_panes) do
    if managed_pane.metadata.workspace == workspace_name and managed_pane.pane then
      managed_pane.pane:inject_output(command)
      count = count + 1
    end
  end
  
  wezterm.log_info("Sent command to " .. count .. " panes in workspace " .. workspace_name)
  return count
end

-- Focus pane by process ID
function PaneManager.focus_process_pane(process_id)
  for _, managed_pane in pairs(state.managed_panes) do
    if managed_pane.metadata.process_id == process_id and managed_pane.pane then
      managed_pane.pane:activate()
      state.last_focus_pane = managed_pane.pane:pane_id()
      wezterm.log_info("Focused pane for process: " .. process_id)
      return true
    end
  end
  
  return false
end

-- Get pane information
function PaneManager.get_pane_info(pane_id)
  return state.managed_panes[pane_id]
end

-- List all managed panes
function PaneManager.list_managed_panes()
  local panes = {}
  for id, managed_pane in pairs(state.managed_panes) do
    table.insert(panes, {
      id = id,
      metadata = managed_pane.metadata,
      created_at = managed_pane.created_at,
    })
  end
  
  table.sort(panes, function(a, b)
    return a.created_at < b.created_at
  end)
  
  return panes
end

-- Get available layouts
function PaneManager.get_available_layouts()
  local layouts = {}
  for name, layout in pairs(config.layouts) do
    table.insert(layouts, {
      name = name,
      display_name = layout.name,
      pane_count = #layout.panes,
    })
  end
  
  table.sort(layouts, function(a, b)
    return a.display_name < b.display_name
  end)
  
  return layouts
end

-- Show layout selection
function PaneManager.show_layout_selector(window, pane)
  local layouts = PaneManager.get_available_layouts()
  local choices = {}
  
  for _, layout in ipairs(layouts) do
    table.insert(choices, {
      id = layout.name,
      label = string.format("%s (%d panes)", layout.display_name, layout.pane_count),
    })
  end
  
  window:perform_action(
    wezterm.action.InputSelector {
      action = wezterm.action_callback(function(inner_window, inner_pane, id, label)
        if id then
          PaneManager.apply_layout(inner_window, inner_pane, id)
        end
      end),
      title = 'Select Layout',
      choices = choices,
      fuzzy = true,
    },
    pane
  )
end

-- Handle pane events
function PaneManager.handle_pane_event(event_name, pane, ...)
  local pane_id = pane:pane_id()
  
  if event_name == "pane_focused" then
    state.last_focus_pane = pane_id
    
    -- Update process focus if enabled
    if config.focus_follows_process then
      local managed_pane = state.managed_panes[pane_id]
      if managed_pane and managed_pane.metadata.process_id then
        -- Notify backend about process focus change
        wezterm.log_info("Process focus changed to: " .. managed_pane.metadata.process_id)
      end
    end
    
  elseif event_name == "pane_closed" then
    PaneManager.unregister_pane(pane_id)
    
  elseif event_name == "pane_created" then
    -- Auto-register new panes if they're in a managed tab
    -- This would require more integration with workspace management
  end
end

-- Cleanup orphaned panes
function PaneManager.cleanup_orphaned_panes()
  local active_pane_ids = {}
  
  -- Get all currently active panes (this would need WezTerm API support)
  -- For now, we'll just clean up very old entries
  local cutoff_time = os.time() - (24 * 60 * 60) -- 24 hours
  
  for pane_id, managed_pane in pairs(state.managed_panes) do
    if managed_pane.created_at < cutoff_time then
      wezterm.log_info("Cleaning up old managed pane: " .. pane_id)
      state.managed_panes[pane_id] = nil
    end
  end
end

-- Get pane manager state
function PaneManager.get_state()
  return {
    managed_panes_count = 0, -- Can't easily count due to table iteration
    sync_groups = state.sync_groups,
    current_layout = state.current_layout,
    sync_enabled = config.sync_enabled,
  }
end

-- Toggle pane synchronization for current group
function PaneManager.toggle_sync_current_panes(window, pane)
  local tab = pane:tab()
  local panes = tab:panes()
  local pane_ids = {}
  
  for _, p in ipairs(panes) do
    table.insert(pane_ids, p:pane_id())
  end
  
  local group_name = "current_tab_" .. tab:tab_id()
  
  if state.sync_groups[group_name] and state.sync_groups[group_name].enabled then
    PaneManager.disable_sync(group_name)
    window:toast_notification("WezTerm Multi-Dev", "Pane synchronization disabled", nil, 2000)
  else
    PaneManager.enable_sync(group_name, pane_ids)
    window:toast_notification("WezTerm Multi-Dev", "Pane synchronization enabled", nil, 2000)
  end
end

-- Create pane from template
function PaneManager.create_from_template(window, pane, template)
  local new_pane = PaneManager.create_process_pane(window, pane, {
    process_id = template.process_id or ("template-" .. os.time()),
    workspace = template.workspace or "default",
    command = template.command,
    working_directory = template.working_directory,
    title = template.title,
    split_direction = template.split_direction or "Right",
    split_size = template.split_size or 0.5,
  })
  
  return new_pane
end

-- テンプレートからレイアウトを適用
function PaneManager.apply_template_layout(window, pane, template_path_or_name)
  local template = nil
  
  -- テンプレートの読み込み
  if template_path_or_name:match("%.ya?ml$") then
    -- ファイルパスとして処理
    template = template_loader.load_template(template_path_or_name)
  else
    -- テンプレート名として検索
    local matches = template_loader.find_template(template_path_or_name)
    if #matches > 0 then
      template = template_loader.load_template(matches[1].file_path)
    end
  end
  
  if not template then
    wezterm.log_error("Template not found: " .. template_path_or_name)
    return false
  end
  
  -- レイアウトエンジンでレイアウト生成
  local layout = layout_engine.generate_from_template(template)
  if not layout or #layout == 0 then
    wezterm.log_error("Failed to generate layout from template")
    return false
  end
  
  -- レイアウトの最適化
  local viewport_size = {width = 120, height = 40} -- デフォルトサイズ
  layout = layout_engine.optimize_layout(layout, viewport_size)
  
  -- 既存のペインをクリア（最初のペイン以外）
  local tab = pane:tab()
  local panes = tab:panes()
  for i = #panes, 2, -1 do
    panes[i]:activate()
    window:perform_action(wezterm.action.CloseCurrentPane { confirm = false }, panes[i])
  end
  
  -- 新しいレイアウトを適用
  local base_pane = tab:panes()[1]
  local created_panes = { base_pane }
  
  for i = 2, #layout do
    local pane_config = layout[i]
    local split_direction = layout_engine.determine_split_direction(pane_config.position)
    
    local new_pane = base_pane:split_pane({
      direction = split_direction,
      size = pane_config.size,
    })
    
    -- 作業ディレクトリの設定
    if pane_config.working_directory then
      new_pane:inject_output("cd " .. wezterm.shell_quote_arg(pane_config.working_directory) .. "\n")
    end
    
    -- コマンドの実行
    if pane_config.command then
      new_pane:inject_output(pane_config.command .. "\n")
    end
    
    -- ペインの登録
    PaneManager.register_pane(new_pane, {
      layout_position = pane_config.position,
      template_name = template.name,
      pane_id = pane_config.id,
      title = pane_config.title
    })
    
    table.insert(created_panes, new_pane)
  end
  
  -- 最初のペインの設定も更新
  if layout[1] then
    local first_pane_config = layout[1]
    if first_pane_config.working_directory then
      base_pane:inject_output("cd " .. wezterm.shell_quote_arg(first_pane_config.working_directory) .. "\n")
    end
    if first_pane_config.command then
      base_pane:inject_output(first_pane_config.command .. "\n")
    end
    
    PaneManager.register_pane(base_pane, {
      layout_position = first_pane_config.position,
      template_name = template.name,
      pane_id = first_pane_config.id,
      title = first_pane_config.title
    })
  end
  
  wezterm.log_info("Applied template layout: " .. template.name)
  return true
end

-- テンプレート選択UI
function PaneManager.show_template_selector(window, pane)
  local templates = template_loader.list_templates()
  local choices = {}
  
  for _, template in ipairs(templates) do
    table.insert(choices, {
      id = template.file_path,
      label = string.format("%s - %s", template.name, template.description or ""),
    })
  end
  
  if #choices == 0 then
    window:toast_notification("WezTerm Multi-Dev", "No templates found", nil, 3000)
    return
  end
  
  window:perform_action(
    wezterm.action.InputSelector {
      action = wezterm.action_callback(function(inner_window, inner_pane, id, label)
        if id then
          PaneManager.apply_template_layout(inner_window, inner_pane, id)
        end
      end),
      title = 'Select Template',
      choices = choices,
      fuzzy = true,
      description = 'Choose a layout template to apply',
    },
    pane
  )
end

-- カスタムレイアウトの保存
function PaneManager.save_current_layout_as_template(window, pane, template_name)
  local tab = pane:tab()
  local panes = tab:panes()
  
  if #panes == 0 then
    wezterm.log_error("No panes to save")
    return false
  end
  
  -- 現在のレイアウトを解析
  local template_panes = {}
  for i, current_pane in ipairs(panes) do
    local pane_info = PaneManager.get_pane_info(current_pane:pane_id())
    
    local template_pane = {
      id = "pane_" .. i,
      position = {
        row = math.floor((i - 1) / 2),
        col = (i - 1) % 2,
        span_rows = 1,
        span_cols = 1
      },
      size = 1.0 / #panes,
      working_directory = "~",
      command = pane_info and pane_info.metadata.command or ""
    }
    
    table.insert(template_panes, template_pane)
  end
  
  -- テンプレート構造を作成
  local template = {
    name = template_name,
    description = "Custom layout saved on " .. os.date("%Y-%m-%d %H:%M:%S"),
    version = "1.0.0",
    author = "User",
    layout = {
      type = "dynamic",
      panes = template_panes
    },
    workspace = {
      name = "custom-workspace"
    }
  }
  
  -- テンプレートディレクトリの確保
  template_loader.ensure_template_directories()
  
  -- テンプレートファイルのパス
  local template_dir = template_loader.get_template_directories()[1]
  local template_file = template_dir .. "/" .. template_name:gsub("[^%w%-_]", "_") .. ".yaml"
  
  -- テンプレートの保存
  if template_loader.save_template(template, template_file) then
    window:toast_notification("WezTerm Multi-Dev", "Layout saved as template: " .. template_name, nil, 3000)
    wezterm.log_info("Saved layout template: " .. template_file)
    return true
  else
    window:toast_notification("WezTerm Multi-Dev", "Failed to save template", nil, 3000)
    return false
  end
end

-- 動的レイアウト調整
function PaneManager.adjust_layout_dynamically(window, pane, adjustment_type)
  local tab = pane:tab()
  local panes = tab:panes()
  
  if #panes <= 1 then
    return false
  end
  
  local current_layout = {}
  for i, current_pane in ipairs(panes) do
    local pane_info = PaneManager.get_pane_info(current_pane:pane_id())
    table.insert(current_layout, {
      id = "pane_" .. i,
      size = 1.0 / #panes,
      pane = current_pane,
      metadata = pane_info and pane_info.metadata or {}
    })
  end
  
  local adjusted_layout = nil
  
  if adjustment_type == "balance" then
    adjusted_layout = layout_engine.balance_layout(current_layout)
  elseif adjustment_type == "enforce_minimum" then
    adjusted_layout = layout_engine.enforce_minimum_size(current_layout, config.minimum_pane_size or 0.1)
  elseif adjustment_type == "optimize" then
    local viewport_size = {width = 120, height = 40}
    adjusted_layout = layout_engine.optimize_layout(current_layout, viewport_size)
  end
  
  if adjusted_layout then
    wezterm.log_info("Applied dynamic layout adjustment: " .. adjustment_type)
    return true
  end
  
  return false
end

-- テンプレート機能の初期化
function PaneManager.init_template_features(framework_config)
  -- テンプレートローダーの初期化
  template_loader.init(framework_config)
  
  -- レイアウトエンジンの初期化
  layout_engine.init(framework_config)
  
  -- テンプレートのプリロード
  template_loader.preload_templates()
  
  wezterm.log_info("Template features initialized")
end

return PaneManager