-- WezTerm Multi-Process Development Framework - Main Configuration
-- This file provides the main configuration for the WezTerm integration

local wezterm = require 'wezterm'
local room_manager = require 'room.manager'
local ui_manager = require 'ui.manager'
local dashboard = require 'ui.dashboard'
local pane_manager = require 'ui.pane_manager'
local keybindings = require 'config.keybindings'

local config = wezterm.config_builder()

-- Framework configuration
local framework_config = {
  socket_path = "/tmp/wezterm-parallel.sock",
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
    leader_key = { key = '', mods = 'SHIFT|CMD|CTRL|ALT' },
    workspace_prefix = 'CTRL|SHIFT',
    process_prefix = 'CTRL|ALT',
    pane_prefix = 'ALT',
    dashboard_prefix = 'CTRL|SHIFT',
  }
}

-- Initialize framework modules
room_manager.init(framework_config)
ui_manager.init(framework_config)
dashboard.init(framework_config)
pane_manager.init(framework_config)
pane_manager.init_template_features(framework_config)
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
config.keys = keybindings.build_keys(room_manager, pane_manager, dashboard)
config.key_tables = keybindings.build_key_tables(room_manager, pane_manager, dashboard)

-- Register keybinding event handlers
wezterm.on('room-create', function(window, pane)
  room_manager.create_room_prompt(window, pane)
end)

-- Window events
wezterm.on('gui-startup', function(cmd)
  if framework_config.auto_start_backend then
    room_manager.start_backend()
  end
  
  -- Create default room if none exists
  room_manager.ensure_default_room()
end)

wezterm.on('window-config-reloaded', function(window, pane)
  ui_manager.show_notification(window, "Configuration reloaded", "info")
  
  -- Reinitialize modules with new config
  room_manager.init(framework_config)
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

-- Tab title customization for room indication
wezterm.on('format-tab-title', function(tab, tabs, panes, config, hover, max_width)
  local room_name = room_manager.get_current_room_name(tab)
  local title = tab.active_pane.title
  
  if room_name then
    title = string.format("[%s] %s", room_name, title)
  end
  
  return {
    { Text = title },
  }
end)

return config