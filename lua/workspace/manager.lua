-- WezTerm Multi-Process Development Framework - Workspace Manager
-- Handles workspace creation, switching, and management

local wezterm = require 'wezterm'
local json = require 'json'  -- Note: may need to install lua-json

local WorkspaceManager = {}

-- Internal state
local config = {}
local current_workspace = "default"
local workspaces = {}
local backend_process = nil

-- Initialize the workspace manager
function WorkspaceManager.init(framework_config)
  config = framework_config
  wezterm.log_info("WorkspaceManager initialized")
end

-- Start the backend Rust process
function WorkspaceManager.start_backend()
  if backend_process then
    wezterm.log_info("Backend already running")
    return
  end
  
  local success, process, err = wezterm.run_child_process({
    'cargo', 'run', '--manifest-path', wezterm.home_dir .. '/.config/wezterm-multi-dev/Cargo.toml'
  })
  
  if success then
    backend_process = process
    wezterm.log_info("Backend process started successfully")
  else
    wezterm.log_error("Failed to start backend: " .. (err or "unknown error"))
  end
end

-- Send message to backend via Unix socket
function WorkspaceManager.send_to_backend(message)
  local message_json = json.encode(message)
  
  -- TODO: Implement actual Unix socket communication
  -- For now, just log the message
  wezterm.log_info("Sending to backend: " .. message_json)
  
  return {
    StatusUpdate = {
      process_id = "lua_client",
      status = "Message sent successfully"
    }
  }
end

-- Create a new workspace
function WorkspaceManager.create_workspace(name, template)
  template = template or "default"
  
  wezterm.log_info(string.format("Creating workspace: %s with template: %s", name, template))
  
  -- Send message to backend
  local message = {
    WorkspaceCreate = {
      name = name,
      template = template
    }
  }
  
  local response = WorkspaceManager.send_to_backend(message)
  
  -- Create new tab for workspace
  local window = wezterm.mux.get_active_window()
  if window then
    local tab = window:spawn_tab({
      cwd = wezterm.home_dir .. "/projects/" .. name,
    })
    
    -- Set tab title
    tab:set_title(string.format("Workspace: %s", name))
    
    -- Store workspace info
    workspaces[name] = {
      tab_id = tab:tab_id(),
      template = template,
      created_at = os.time(),
      processes = {}
    }
    
    current_workspace = name
  end
  
  return response
end

-- Switch to an existing workspace
function WorkspaceManager.switch_workspace(name)
  if not workspaces[name] then
    wezterm.log_error("Workspace not found: " .. name)
    return false
  end
  
  wezterm.log_info("Switching to workspace: " .. name)
  
  local workspace = workspaces[name]
  local window = wezterm.mux.get_active_window()
  
  if window then
    local tab = window:get_tab(workspace.tab_id)
    if tab then
      tab:activate()
      current_workspace = name
      return true
    else
      wezterm.log_error("Tab not found for workspace: " .. name)
    end
  end
  
  return false
end

-- Prompt user to create workspace
function WorkspaceManager.create_workspace_prompt(window, pane)
  window:perform_action(
    wezterm.action.PromptInputLine {
      description = 'Enter workspace name:',
      action = wezterm.action_callback(function(inner_window, inner_pane, line)
        if line and line ~= "" then
          WorkspaceManager.create_workspace(line)
        end
      end),
    },
    pane
  )
end

-- Prompt user to switch workspace
function WorkspaceManager.switch_workspace_prompt(window, pane)
  local workspace_list = {}
  for name, _ in pairs(workspaces) do
    table.insert(workspace_list, name)
  end
  
  if #workspace_list == 0 then
    wezterm.log_info("No workspaces available")
    return
  end
  
  -- Create choices for workspace selection
  local choices = {}
  for _, name in ipairs(workspace_list) do
    table.insert(choices, {
      id = name,
      label = name,
    })
  end
  
  window:perform_action(
    wezterm.action.InputSelector {
      action = wezterm.action_callback(function(inner_window, inner_pane, id, label)
        if id then
          WorkspaceManager.switch_workspace(id)
        end
      end),
      title = 'Select Workspace',
      choices = choices,
    },
    pane
  )
end

-- Kill process prompt
function WorkspaceManager.kill_process_prompt(window, pane)
  window:perform_action(
    wezterm.action.PromptInputLine {
      description = 'Enter process ID to kill:',
      action = wezterm.action_callback(function(inner_window, inner_pane, line)
        if line and line ~= "" then
          -- Send kill message to backend
          local message = {
            ProcessKill = {
              process_id = line
            }
          }
          WorkspaceManager.send_to_backend(message)
        end
      end),
    },
    pane
  )
end

-- Get current workspace name for tab title
function WorkspaceManager.get_current_workspace_name(tab)
  for name, workspace in pairs(workspaces) do
    if workspace.tab_id == tab:tab_id() then
      return name
    end
  end
  return nil
end

-- Ensure default workspace exists
function WorkspaceManager.ensure_default_workspace()
  if not workspaces["default"] then
    WorkspaceManager.create_workspace("default", "basic")
  end
end

-- Spawn Claude Code process in current workspace
function WorkspaceManager.spawn_claude_code(workspace_name)
  workspace_name = workspace_name or current_workspace
  
  local message = {
    ProcessSpawn = {
      workspace = workspace_name,
      command = "claude-code --workspace=" .. workspace_name
    }
  }
  
  return WorkspaceManager.send_to_backend(message)
end

-- List all workspaces
function WorkspaceManager.list_workspaces()
  return workspaces
end

return WorkspaceManager