-- WezTerm Multi-Process Development Framework - Enhanced Keybindings
-- Provides comprehensive keyboard shortcuts for all framework features

local wezterm = require 'wezterm'

local Keybindings = {}

-- Default keybinding configuration
local config = {
  leader_key = { key = 'Space', mods = 'CTRL|SHIFT' },
  workspace_prefix = 'CTRL|SHIFT',
  process_prefix = 'CTRL|ALT',
  pane_prefix = 'ALT',
  dashboard_prefix = 'CTRL|SHIFT',
}

-- Initialize keybindings
function Keybindings.init(framework_config)
  if framework_config.keybindings then
    for k, v in pairs(framework_config.keybindings) do
      config[k] = v
    end
  end
  
  wezterm.log_info("Keybindings initialized")
end

-- Build complete keybinding configuration
function Keybindings.build_keys(workspace_manager, pane_manager, dashboard)
  local keys = {}
  
  -- Add workspace management keys
  table.insert(keys, {
    key = 'n',
    mods = config.workspace_prefix,
    action = wezterm.action_callback(function(window, pane)
      workspace_manager.create_workspace_prompt(window, pane)
    end),
  })
  
  table.insert(keys, {
    key = 'w',
    mods = config.workspace_prefix,
    action = wezterm.action_callback(function(window, pane)
      workspace_manager.switch_workspace_prompt(window, pane)
    end),
  })
  
  -- Test backend connection
  table.insert(keys, {
    key = 't',
    mods = config.workspace_prefix,
    action = wezterm.action_callback(function(window, pane)
      local success = workspace_manager.test_connection()
      local message = success and "Backend connection: OK" or "Backend connection: FAILED"
      window:toast_notification("WezTerm Parallel", message, nil, 3000)
    end),
  })
  
  -- Get backend status
  table.insert(keys, {
    key = 's',
    mods = config.workspace_prefix,
    action = wezterm.action_callback(function(window, pane)
      local status = workspace_manager.get_backend_status()
      local message = status and "Backend status: " .. (status.status or "Unknown") or "Backend: No response"
      window:toast_notification("WezTerm Parallel", message, nil, 3000)
    end),
  })
  
  table.insert(keys, {
    key = 'x',
    mods = config.workspace_prefix,
    action = wezterm.action_callback(function(window, pane)
      workspace_manager.delete_workspace_prompt(window, pane)
    end),
  })
  
  table.insert(keys, {
    key = 'l',
    mods = config.workspace_prefix,
    action = wezterm.action_callback(function(window, pane)
      workspace_manager.list_workspaces_prompt(window, pane)
    end),
  })
  
  -- Add process management keys
  table.insert(keys, {
    key = 's',
    mods = config.process_prefix,
    action = wezterm.action_callback(function(window, pane)
      Keybindings.spawn_process_prompt(window, pane)
    end),
  })
  
  table.insert(keys, {
    key = 'k',
    mods = config.process_prefix,
    action = wezterm.action_callback(function(window, pane)
      workspace_manager.kill_process_prompt(window, pane)
    end),
  })
  
  table.insert(keys, {
    key = 'r',
    mods = config.process_prefix,
    action = wezterm.action_callback(function(window, pane)
      Keybindings.restart_process_prompt(window, pane)
    end),
  })
  
  table.insert(keys, {
    key = 'p',
    mods = config.process_prefix,
    action = wezterm.action_callback(function(window, pane)
      Keybindings.list_processes_prompt(window, pane)
    end),
  })
  
  -- Add pane management keys
  table.insert(keys, {
    key = 'h',
    mods = config.pane_prefix,
    action = wezterm.action.ActivatePaneDirection 'Left',
  })
  
  table.insert(keys, {
    key = 'l',
    mods = config.pane_prefix,
    action = wezterm.action.ActivatePaneDirection 'Right',
  })
  
  table.insert(keys, {
    key = 'k',
    mods = config.pane_prefix,
    action = wezterm.action.ActivatePaneDirection 'Up',
  })
  
  table.insert(keys, {
    key = 'j',
    mods = config.pane_prefix,
    action = wezterm.action.ActivatePaneDirection 'Down',
  })
  
  table.insert(keys, {
    key = 'v',
    mods = config.pane_prefix,
    action = wezterm.action.SplitHorizontal { domain = 'CurrentPaneDomain' },
  })
  
  table.insert(keys, {
    key = 'h',
    mods = config.pane_prefix,
    action = wezterm.action.SplitVertical { domain = 'CurrentPaneDomain' },
  })
  
  table.insert(keys, {
    key = 'z',
    mods = config.pane_prefix,
    action = wezterm.action.TogglePaneZoomState,
  })
  
  table.insert(keys, {
    key = 'x',
    mods = config.pane_prefix,
    action = wezterm.action.CloseCurrentPane { confirm = true },
  })
  
  -- Pane layouts
  table.insert(keys, {
    key = 'space',
    mods = config.pane_prefix,
    action = wezterm.action_callback(function(window, pane)
      pane_manager.show_layout_selector(window, pane)
    end),
  })
  
  -- Pane synchronization
  table.insert(keys, {
    key = 's',
    mods = config.pane_prefix,
    action = wezterm.action_callback(function(window, pane)
      pane_manager.toggle_sync_current_panes(window, pane)
    end),
  })
  
  -- Dashboard keys
  table.insert(keys, {
    key = 'd',
    mods = config.dashboard_prefix,
    action = wezterm.action_callback(function(window, pane)
      dashboard.toggle(window, pane)
    end),
  })
  
  table.insert(keys, {
    key = 'r',
    mods = config.dashboard_prefix,
    action = wezterm.action_callback(function(window, pane)
      if dashboard.is_visible() then
        dashboard.update_data()
        window:toast_notification("WezTerm Multi-Dev", "Dashboard refreshed", nil, 1000)
      end
    end),
  })
  
  -- Quick workspace switching (numbered)
  for i = 1, 9 do
    table.insert(keys, {
      key = tostring(i),
      mods = config.workspace_prefix,
      action = wezterm.action_callback(function(window, pane)
        Keybindings.switch_to_numbered_workspace(window, pane, i)
      end),
    })
  end
  
  -- Process focus keys (F1-F12)
  for i = 1, 12 do
    table.insert(keys, {
      key = 'F' .. tostring(i),
      mods = '',
      action = wezterm.action_callback(function(window, pane)
        Keybindings.focus_process_by_number(window, pane, i)
      end),
    })
  end
  
  -- Standard WezTerm keys
  table.insert(keys, {
    key = 'c',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.CopyTo 'Clipboard',
  })
  
  table.insert(keys, {
    key = 'v',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.PasteFrom 'Clipboard',
  })
  
  table.insert(keys, {
    key = 't',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.SpawnTab 'CurrentPaneDomain',
  })
  
  table.insert(keys, {
    key = 'w',
    mods = 'CTRL',
    action = wezterm.action.CloseCurrentTab { confirm = true },
  })
  
  table.insert(keys, {
    key = 'Enter',
    mods = 'ALT',
    action = wezterm.action.ToggleFullScreen,
  })
  
  -- Tab navigation
  table.insert(keys, {
    key = 'Tab',
    mods = 'CTRL',
    action = wezterm.action.ActivateTabRelative(1),
  })
  
  table.insert(keys, {
    key = 'Tab',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.ActivateTabRelative(-1),
  })
  
  -- Font size adjustment
  table.insert(keys, {
    key = '=',
    mods = 'CTRL',
    action = wezterm.action.IncreaseFontSize,
  })
  
  table.insert(keys, {
    key = '-',
    mods = 'CTRL',
    action = wezterm.action.DecreaseFontSize,
  })
  
  table.insert(keys, {
    key = '0',
    mods = 'CTRL',
    action = wezterm.action.ResetFontSize,
  })
  
  -- Search and debug
  table.insert(keys, {
    key = 'f',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.Search { CaseInSensitiveString = '' },
  })
  
  table.insert(keys, {
    key = 'l',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.ShowDebugOverlay,
  })
  
  -- Leader key sequences
  table.insert(keys, {
    key = config.leader_key.key,
    mods = config.leader_key.mods,
    action = wezterm.action.ActivateKeyTable {
      name = 'leader',
      timeout_milliseconds = 2000,
    },
  })
  
  return keys
end

-- Build leader key table
function Keybindings.build_key_tables(workspace_manager, pane_manager, dashboard)
  return {
    leader = {
      -- Workspace management
      { key = 'w', action = wezterm.action_callback(function(window, pane)
          workspace_manager.switch_workspace_prompt(window, pane)
        end) },
      { key = 'n', action = wezterm.action_callback(function(window, pane)
          workspace_manager.create_workspace_prompt(window, pane)
        end) },
      { key = 'x', action = wezterm.action_callback(function(window, pane)
          workspace_manager.delete_workspace_prompt(window, pane)
        end) },
      { key = 'l', action = wezterm.action_callback(function(window, pane)
          workspace_manager.list_workspaces_prompt(window, pane)
        end) },
      
      -- Process management
      { key = 's', action = wezterm.action_callback(function(window, pane)
          Keybindings.spawn_process_prompt(window, pane)
        end) },
      { key = 'k', action = wezterm.action_callback(function(window, pane)
          workspace_manager.kill_process_prompt(window, pane)
        end) },
      { key = 'r', action = wezterm.action_callback(function(window, pane)
          Keybindings.restart_process_prompt(window, pane)
        end) },
      { key = 'p', action = wezterm.action_callback(function(window, pane)
          Keybindings.list_processes_prompt(window, pane)
        end) },
      
      -- Dashboard
      { key = 'd', action = wezterm.action_callback(function(window, pane)
          dashboard.toggle(window, pane)
        end) },
      
      -- Pane layouts
      { key = 'space', action = wezterm.action_callback(function(window, pane)
          pane_manager.show_layout_selector(window, pane)
        end) },
      
      -- Help
      { key = '?', action = wezterm.action_callback(function(window, pane)
          Keybindings.show_help(window, pane)
        end) },
      
      -- Config reload
      { key = 'R', action = wezterm.action.ReloadConfiguration },
    }
  }
end

-- Helper functions for prompts
function Keybindings.spawn_process_prompt(window, pane)
  window:perform_action(
    wezterm.action.PromptInputLine {
      description = 'Enter process command:',
      action = wezterm.action_callback(function(inner_window, inner_pane, line)
        if line and line ~= "" then
          -- Send spawn command to backend
          local message = {
            ProcessSpawn = {
              workspace = "current", -- TODO: Get current workspace
              command = line
            }
          }
          -- TODO: Send to backend
          inner_window:toast_notification("WezTerm Multi-Dev", "Process spawn requested: " .. line, nil, 2000)
        end
      end),
    },
    pane
  )
end

function Keybindings.restart_process_prompt(window, pane)
  window:perform_action(
    wezterm.action.PromptInputLine {
      description = 'Enter process ID to restart:',
      action = wezterm.action_callback(function(inner_window, inner_pane, line)
        if line and line ~= "" then
          -- Send restart command to backend
          local message = {
            ProcessRestart = {
              process_id = line
            }
          }
          -- TODO: Send to backend
          inner_window:toast_notification("WezTerm Multi-Dev", "Process restart requested: " .. line, nil, 2000)
        end
      end),
    },
    pane
  )
end

function Keybindings.list_processes_prompt(window, pane)
  -- TODO: Get actual process list from backend
  local processes = {
    { id = "claude-1", workspace = "default", status = "Running" },
    { id = "claude-2", workspace = "frontend", status = "Idle" },
    { id = "claude-3", workspace = "backend", status = "Busy" },
  }
  
  local choices = {}
  for _, proc in ipairs(processes) do
    table.insert(choices, {
      id = proc.id,
      label = string.format("%s [%s] - %s", proc.id, proc.workspace, proc.status),
    })
  end
  
  if #choices > 0 then
    window:perform_action(
      wezterm.action.InputSelector {
        action = wezterm.action_callback(function(inner_window, inner_pane, id, label)
          if id then
            inner_window:toast_notification("WezTerm Multi-Dev", "Selected process: " .. id, nil, 2000)
            -- TODO: Perform action on selected process
          end
        end),
        title = 'Select Process',
        choices = choices,
        fuzzy = true,
      },
      pane
    )
  else
    window:toast_notification("WezTerm Multi-Dev", "No processes found", nil, 2000)
  end
end

function Keybindings.switch_to_numbered_workspace(window, pane, number)
  -- TODO: Get workspace list and switch to nth workspace
  window:toast_notification("WezTerm Multi-Dev", "Switch to workspace " .. number, nil, 1000)
end

function Keybindings.focus_process_by_number(window, pane, number)
  -- TODO: Focus nth process pane
  window:toast_notification("WezTerm Multi-Dev", "Focus process " .. number, nil, 1000)
end

function Keybindings.show_help(window, pane)
  local help_text = [[
WezTerm Multi-Dev Framework - Keybindings Help

=== Workspace Management ===
Ctrl+Shift+N     - Create new workspace
Ctrl+Shift+W     - Switch workspace
Ctrl+Shift+X     - Delete workspace
Ctrl+Shift+L     - List workspaces
Ctrl+Shift+1-9   - Switch to numbered workspace

=== Process Management ===
Ctrl+Alt+S       - Spawn new process
Ctrl+Alt+K       - Kill process
Ctrl+Alt+R       - Restart process
Ctrl+Alt+P       - List processes
F1-F12           - Focus process by number

=== Pane Management ===
Alt+H/J/K/L      - Navigate panes
Alt+V            - Split vertically
Alt+H            - Split horizontally
Alt+Z            - Toggle pane zoom
Alt+X            - Close pane
Alt+S            - Toggle pane sync
Alt+Space        - Select layout

=== Dashboard ===
Ctrl+Shift+D     - Toggle dashboard
Ctrl+Shift+R     - Refresh dashboard

=== Leader Key (Ctrl+Shift+Space) ===
w                - Switch workspace
n                - New workspace
s                - Spawn process
d                - Toggle dashboard
?                - Show this help

=== Standard WezTerm ===
Ctrl+Shift+C     - Copy
Ctrl+Shift+V     - Paste
Ctrl+Shift+T     - New tab
Ctrl+W           - Close tab
Alt+Enter        - Toggle fullscreen
Ctrl+F           - Search
]]
  
  -- Create new tab with help content
  local tab = window:spawn_tab {}
  local help_pane = tab:active_pane()
  help_pane:inject_output(help_text)
  help_pane:inject_output("\n\nPress any key to close this help...")
end

-- Get key description for UI display
function Keybindings.get_key_descriptions()
  return {
    {
      category = "Workspace",
      keys = {
        { key = "Ctrl+Shift+N", description = "Create workspace" },
        { key = "Ctrl+Shift+W", description = "Switch workspace" },
        { key = "Ctrl+Shift+X", description = "Delete workspace" },
        { key = "Ctrl+Shift+L", description = "List workspaces" },
      }
    },
    {
      category = "Process",
      keys = {
        { key = "Ctrl+Alt+S", description = "Spawn process" },
        { key = "Ctrl+Alt+K", description = "Kill process" },
        { key = "Ctrl+Alt+R", description = "Restart process" },
        { key = "Ctrl+Alt+P", description = "List processes" },
      }
    },
    {
      category = "Pane",
      keys = {
        { key = "Alt+H/J/K/L", description = "Navigate panes" },
        { key = "Alt+Space", description = "Select layout" },
        { key = "Alt+S", description = "Toggle sync" },
      }
    },
    {
      category = "Dashboard",
      keys = {
        { key = "Ctrl+Shift+D", description = "Toggle dashboard" },
        { key = "Ctrl+Shift+R", description = "Refresh" },
      }
    }
  }
end

return Keybindings