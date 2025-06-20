-- WezTerm Multi-Process Development Framework - Main Configuration
-- This file provides the main configuration for the WezTerm integration

local wezterm = require 'wezterm'
local workspace_manager = require 'lua.workspace.manager'
local ui_manager = require 'lua.ui.manager'

local config = wezterm.config_builder()

-- Framework configuration
local framework_config = {
  socket_path = "/tmp/wezterm-multi-dev.sock",
  auto_start_backend = true,
  max_workspaces = 8,
  max_processes_per_workspace = 16,
  debug_mode = false,
}

-- Initialize framework modules
workspace_manager.init(framework_config)
ui_manager.init(framework_config)

-- Basic WezTerm configuration
config.color_scheme = 'Dark+'
config.font = wezterm.font('JetBrains Mono')
config.font_size = 12.0

-- Tab bar configuration
config.tab_bar_at_bottom = true
config.use_fancy_tab_bar = false
config.tab_and_split_indices_are_zero_based = true

-- Workspace-specific keybindings
config.keys = {
  -- Framework specific keybindings
  {
    key = 'n',
    mods = 'CTRL|SHIFT',
    action = wezterm.action_callback(function(window, pane)
      workspace_manager.create_workspace_prompt(window, pane)
    end),
  },
  {
    key = 'w',
    mods = 'CTRL|SHIFT',
    action = wezterm.action_callback(function(window, pane)
      workspace_manager.switch_workspace_prompt(window, pane)
    end),
  },
  {
    key = 'd',
    mods = 'CTRL|SHIFT',
    action = wezterm.action_callback(function(window, pane)
      ui_manager.show_dashboard(window, pane)
    end),
  },
  {
    key = 'k',
    mods = 'CTRL|SHIFT',
    action = wezterm.action_callback(function(window, pane)
      workspace_manager.kill_process_prompt(window, pane)
    end),
  },
  
  -- Standard WezTerm keybindings
  {
    key = 'c',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.CopyTo 'Clipboard',
  },
  {
    key = 'v',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.PasteFrom 'Clipboard',
  },
  {
    key = 't',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.SpawnTab 'CurrentPaneDomain',
  },
  {
    key = 'Enter',
    mods = 'ALT',
    action = wezterm.action.ToggleFullScreen,
  },
}

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