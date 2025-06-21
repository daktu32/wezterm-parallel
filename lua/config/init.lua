-- WezTerm Multi-Process Development Framework - Main Configuration
-- This file provides the main configuration for the WezTerm integration

local wezterm = require 'wezterm'
local workspace_manager = require 'lua.workspace.manager'
local ui_manager = require 'lua.ui.manager'
local dashboard = require 'lua.ui.dashboard'
local pane_manager = require 'lua.ui.pane_manager'
local keybindings = require 'lua.config.keybindings'

local config = wezterm.config_builder()

-- Framework configuration
local framework_config = {
  socket_path = "/tmp/wezterm-multi-dev.sock",
  auto_start_backend = true,
  max_workspaces = 8,
  max_processes_per_workspace = 16,
  debug_mode = false,
  
  -- Dashboard configuration
  dashboard = {
    update_interval = 2.0,
    width_percentage = 30,
    position = "right",
    theme = {
      background = "#1e1e2e",
      foreground = "#cdd6f4",
      border = "#45475a",
      header = "#89b4fa",
      success = "#a6e3a1",
      warning = "#f9e2af",
      error = "#f38ba8",
      info = "#89dceb",
    }
  },
  
  -- Pane manager configuration
  pane_manager = {
    auto_balance = true,
    focus_follows_process = true,
    pane_titles_enabled = true,
  },
  
  -- Keybindings configuration
  keybindings = {
    leader_key = { key = 'Space', mods = 'CTRL|SHIFT' },
    workspace_prefix = 'CTRL|SHIFT',
    process_prefix = 'CTRL|ALT',
    pane_prefix = 'ALT',
    dashboard_prefix = 'CTRL|SHIFT',
  }
}

-- Initialize framework modules
workspace_manager.init(framework_config)
ui_manager.init(framework_config)
dashboard.init(framework_config)
pane_manager.init(framework_config)
keybindings.init(framework_config)

-- Basic WezTerm configuration
config.color_scheme = 'Dark+'
config.font = wezterm.font('JetBrains Mono')
config.font_size = 12.0

-- Tab bar configuration
config.tab_bar_at_bottom = true
config.use_fancy_tab_bar = false
config.tab_and_split_indices_are_zero_based = true

-- Enhanced keybindings with comprehensive framework support
config.keys = keybindings.build_keys(workspace_manager, pane_manager, dashboard)
config.key_tables = keybindings.build_key_tables(workspace_manager, pane_manager, dashboard)

-- Window events
wezterm.on('gui-startup', function(cmd)
  if framework_config.auto_start_backend then
    workspace_manager.start_backend()
  end
  
  -- Create default workspace if none exists
  workspace_manager.ensure_default_workspace()
end)

wezterm.on('window-config-reloaded', function(window, pane)
  ui_manager.show_notification(window, "Configuration reloaded", "info")
  
  -- Reinitialize modules with new config
  workspace_manager.init(framework_config)
  ui_manager.init(framework_config)
  dashboard.init(framework_config)
  pane_manager.init(framework_config)
  keybindings.init(framework_config)
end)

-- Enhanced pane events
wezterm.on('pane-event-notification', function(pane, event_name, ...)
  pane_manager.handle_pane_event(event_name, pane, ...)
  
  -- Update dashboard if visible
  if dashboard.is_visible() then
    dashboard.handle_event({
      type = event_name,
      pane_id = pane:pane_id(),
      timestamp = os.time(),
    })
  end
end)

-- Process events from backend
wezterm.on('process-event', function(event)
  -- Handle process events and update UI accordingly
  if dashboard.is_visible() then
    dashboard.handle_event(event)
  end
  
  if event.type == "process_started" and framework_config.pane_manager.focus_follows_process then
    pane_manager.focus_process_pane(event.process_id)
  end
end)

-- Tab title customization for workspace indication
wezterm.on('format-tab-title', function(tab, tabs, panes, config, hover, max_width)
  local workspace_name = workspace_manager.get_current_workspace_name(tab)
  local title = tab.active_pane.title
  
  if workspace_name then
    title = string.format("[%s] %s", workspace_name, title)
  end
  
  return {
    { Text = title },
  }
end)

return config