-- WezTerm Multi-Process Development Framework - UI Manager
-- Handles dashboard, notifications, and UI elements

local wezterm = require 'wezterm'

local UIManager = {}

-- Internal state
local config = {}
local notifications = {}
local dashboard_visible = false

-- Initialize the UI manager
function UIManager.init(framework_config)
  config = framework_config
  wezterm.log_info("UIManager initialized")
end

-- Show notification
function UIManager.show_notification(window, message, level)
  level = level or "info"
  
  local notification = {
    message = message,
    level = level,
    timestamp = os.time()
  }
  
  table.insert(notifications, notification)
  
  -- Keep only last 10 notifications
  if #notifications > 10 then
    table.remove(notifications, 1)
  end
  
  wezterm.log_info(string.format("[%s] %s", level:upper(), message))
  
  -- Show toast notification in WezTerm
  window:toast_notification("WezTerm Multi-Dev", message, nil, 3000)
end

-- Show dashboard
function UIManager.show_dashboard(window, pane)
  if dashboard_visible then
    UIManager.hide_dashboard(window, pane)
    return
  end
  
  dashboard_visible = true
  
  -- Create dashboard content
  local dashboard_content = UIManager.build_dashboard_content()
  
  -- Create new pane for dashboard
  local tab = pane:tab()
  local dashboard_pane = tab:split_pane({
    direction = "Right",
    size = 0.3,  -- 30% of the screen
  })
  
  -- Send dashboard content to pane
  dashboard_pane:send_text(dashboard_content)
  
  wezterm.log_info("Dashboard shown")
end

-- Hide dashboard
function UIManager.hide_dashboard(window, pane)
  dashboard_visible = false
  -- TODO: Implement dashboard hiding logic
  wezterm.log_info("Dashboard hidden")
end

-- Build dashboard content
function UIManager.build_dashboard_content()
  local content = {}
  
  -- Header
  table.insert(content, "╔══════════════════════════════════════════╗")
  table.insert(content, "║        WezTerm Multi-Dev Dashboard      ║")
  table.insert(content, "╠══════════════════════════════════════════╣")
  
  -- Workspace section
  table.insert(content, "║ Workspaces:                              ║")
  
  -- TODO: Get actual workspace data from workspace manager
  local workspaces = {
    { name = "default", status = "active", processes = 2 },
    { name = "frontend", status = "idle", processes = 1 },
    { name = "backend", status = "busy", processes = 3 }
  }
  
  for _, workspace in ipairs(workspaces) do
    local status_icon = workspace.status == "active" and "●" or 
                       workspace.status == "busy" and "◉" or "○"
    local line = string.format("║ %s %-15s %s (%d procs)      ║", 
                              status_icon, workspace.name, workspace.status, workspace.processes)
    table.insert(content, line)
  end
  
  table.insert(content, "╠══════════════════════════════════════════╣")
  
  -- Process section
  table.insert(content, "║ Processes:                               ║")
  
  -- TODO: Get actual process data from backend
  local processes = {
    { id = "claude-1", workspace = "default", status = "running", cpu = "12%" },
    { id = "claude-2", workspace = "frontend", status = "idle", cpu = "2%" },
    { id = "claude-3", workspace = "backend", status = "running", cpu = "18%" }
  }
  
  for _, process in ipairs(processes) do
    local status_icon = process.status == "running" and "▶" or 
                       process.status == "idle" and "⏸" or "⏹"
    local line = string.format("║ %s %-10s %-10s %5s        ║", 
                              status_icon, process.id, process.workspace, process.cpu)
    table.insert(content, line)
  end
  
  table.insert(content, "╠══════════════════════════════════════════╣")
  
  -- Recent notifications
  table.insert(content, "║ Recent Notifications:                    ║")
  
  local recent_notifications = {}
  for i = math.max(1, #notifications - 3), #notifications do
    if notifications[i] then
      table.insert(recent_notifications, notifications[i])
    end
  end
  
  if #recent_notifications == 0 then
    table.insert(content, "║ No recent notifications                  ║")
  else
    for _, notification in ipairs(recent_notifications) do
      local level_icon = notification.level == "error" and "✗" or 
                        notification.level == "warn" and "⚠" or "ℹ"
      local line = string.format("║ %s %-35s ║", 
                                level_icon, notification.message:sub(1, 35))
      table.insert(content, line)
    end
  end
  
  table.insert(content, "╠══════════════════════════════════════════╣")
  
  -- Keybindings help
  table.insert(content, "║ Keybindings:                             ║")
  table.insert(content, "║ Ctrl+Shift+N - New Workspace            ║")
  table.insert(content, "║ Ctrl+Shift+W - Switch Workspace         ║")
  table.insert(content, "║ Ctrl+Shift+D - Toggle Dashboard         ║")
  table.insert(content, "║ Ctrl+Shift+K - Kill Process             ║")
  
  table.insert(content, "╚══════════════════════════════════════════╝")
  
  return table.concat(content, "\n") .. "\n"
end

-- Update dashboard if visible
function UIManager.update_dashboard(window)
  if not dashboard_visible then
    return
  end
  
  -- TODO: Implement dashboard update logic
  wezterm.log_info("Dashboard updated")
end

-- Show process status in tab title
function UIManager.format_tab_title_with_status(tab, workspace_name, process_count)
  local status_indicator = ""
  
  if process_count > 0 then
    status_indicator = string.format(" (%d)", process_count)
  end
  
  return string.format("[%s%s] %s", workspace_name or "unknown", status_indicator, tab.active_pane.title)
end

-- Show error message
function UIManager.show_error(window, error_message)
  UIManager.show_notification(window, error_message, "error")
end

-- Show success message
function UIManager.show_success(window, success_message)
  UIManager.show_notification(window, success_message, "info")
end

-- Show warning message
function UIManager.show_warning(window, warning_message)
  UIManager.show_notification(window, warning_message, "warn")
end

-- Clear all notifications
function UIManager.clear_notifications()
  notifications = {}
  wezterm.log_info("Notifications cleared")
end

-- Get current notifications
function UIManager.get_notifications()
  return notifications
end

return UIManager