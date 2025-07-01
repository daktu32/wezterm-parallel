-- WezTerm Multi-Process Development Framework - Workspace Manager
-- Handles workspace creation, switching, and management

-- Mock wezterm for compatibility
local wezterm = _G.wezterm or {
  log_info = function(msg) print("[INFO] " .. msg) end,
  log_error = function(msg) print("[ERROR] " .. msg) end,
  run_child_process = function() return {exit_code=1, stderr="Mock implementation"} end,
  home_dir = "~",
  mux = {
    get_active_window = function() return nil end
  }
}

local json = require 'utils.json'
local socket_client = require 'utils.socket_client'

local WorkspaceManager = {}

-- Internal state
local config = {}
local current_workspace = "default"
local workspaces = {}
local backend_process = nil

-- Helper function to count table entries
local function table_count(t)
  local count = 0
  for _ in pairs(t) do
    count = count + 1
  end
  return count
end

-- Initialize the workspace manager
function WorkspaceManager.init(framework_config)
  config = framework_config
  socket_client.init(framework_config)
  wezterm.log_info("WorkspaceManager initialized")
end

-- Start the backend Rust process
function WorkspaceManager.start_backend()
  if backend_process then
    wezterm.log_info("Backend already running")
    return
  end
  
  local success, process, err = wezterm.run_child_process({
    'cargo', 'run', '--manifest-path', wezterm.home_dir .. '/.config/wezterm-parallel/Cargo.toml'
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
  
  -- Use WezTerm's run_child_process to communicate with backend
  local success, stdout, stderr = wezterm.run_child_process({
    'echo', message_json, '|', 'nc', '-U', config.socket_path
  })
  
  if success then
    wezterm.log_info("Message sent to backend: " .. message_json)
    -- Try to parse response
    local ok, response = pcall(json.decode, stdout)
    if ok and response then
      return response
    end
  else
    wezterm.log_error("Failed to send message: " .. (stderr or "unknown error"))
  end
  
  return {
    StatusUpdate = {
      process_id = "lua_client",
      status = "Message sent: " .. (success and "success" or "failed")
    }
  }
end

-- Create a new workspace
function WorkspaceManager.create_workspace(name, template, window, pane)
  template = template or "basic"
  
  wezterm.log_info(string.format("Creating workspace: %s with template: %s", name, template))
  
  -- Use socket client to create workspace
  local response = socket_client.create_workspace(name, template)
  
  -- wezterm をローカルで require
  local wezterm = require('wezterm')
  
  -- 新しいタブをSpawnTabアクションで作成
  if window then
    window:perform_action(
      wezterm.action.SpawnTab 'CurrentPaneDomain',
      pane
    )
    
    -- 新しいタブのペインを取得
    local new_pane = window:active_pane()
    local new_tab = window:active_tab()
    
    -- タブタイトルを設定
    new_tab:set_title(string.format("Workspace: %s", name))
    
    -- Store workspace info with window reference
    workspaces[name] = {
      tab = new_tab,
      template = template,
      created_at = os.time(),
      processes = {},
      window = window,
      pane = new_pane
    }
    
    current_workspace = name
    
    wezterm.log_info("Workspace created and tab activated: " .. name)
    
    -- プロジェクトディレクトリに移動
    new_pane:send_text("cd ~/projects/" .. name .. " 2>/dev/null || cd ~\n")
  else
    wezterm.log_error("Window not available for workspace creation")
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
  
  -- 保存されたタブが存在するか確認
  if workspace.tab then
    -- タブをアクティブ化
    workspace.tab:activate()
    current_workspace = name
    wezterm.log_info("Activated tab for workspace: " .. name)
    return true
  elseif workspace.window then
    -- タブが存在しない場合は新規作成
    local wezterm = require('wezterm')
    workspace.window:perform_action(
      wezterm.action.SpawnTab 'CurrentPaneDomain',
      workspace.pane
    )
    
    local new_tab = workspace.window:active_tab()
    new_tab:set_title(string.format("Workspace: %s", name))
    
    -- ワークスペース情報を更新
    workspace.tab = new_tab
    workspace.pane = workspace.window:active_pane()
    
    current_workspace = name
    wezterm.log_info("Created new tab for workspace: " .. name)
    
    -- プロジェクトディレクトリに移動
    workspace.pane:send_text("cd ~/projects/" .. name .. " 2>/dev/null || cd ~\n")
    
    return true
  else
    wezterm.log_error("Window not found for workspace: " .. name)
  end
  
  return false
end

-- Prompt user to create workspace
function WorkspaceManager.create_workspace_prompt(window, pane)
  -- wezterm.actionをrequireで取得
  local wezterm = require('wezterm')
  local act = wezterm.action
  
  window:perform_action(
    act.PromptInputLine {
      description = 'Enter workspace name:',
      action = wezterm.action_callback(function(inner_window, inner_pane, line)
        if line and line ~= "" then
          -- 実際にワークスペースを作成（タブも作成される）
          WorkspaceManager.create_workspace(line, "basic", inner_window, inner_pane)
          
          wezterm.log_info("Workspace created: " .. line)
        end
      end),
    },
    pane
  )
end

-- Prompt user to switch workspace
function WorkspaceManager.switch_workspace_prompt(window, pane)
  local wezterm = require('wezterm')
  local act = wezterm.action
  local choices = {}
  
  for name, ws in pairs(workspaces) do
    local is_current = (name == current_workspace) and " [CURRENT]" or ""
    table.insert(choices, {
      id = name,
      label = string.format("%s%s", name, is_current),
    })
  end
  
  if #choices == 0 then
    wezterm.log_info("No workspaces available. Create one with Ctrl+Shift+N")
    return
  end
  
  -- ワークスペース選択ダイアログを表示
  window:perform_action(
    act.InputSelector {
      action = wezterm.action_callback(function(inner_window, inner_pane, id, label)
        if id then
          -- ワークスペースが存在しない場合は新規作成
          if not workspaces[id] then
            WorkspaceManager.create_workspace(id, "basic", inner_window, inner_pane)
          else
            -- 既存のワークスペースに切り替え
            WorkspaceManager.switch_workspace(id)
          end
          
          wezterm.log_info("Switched to workspace: " .. id)
        end
      end),
      title = 'Select Workspace',
      choices = choices,
      fuzzy = true,
    },
    pane
  )
end

-- Kill process prompt
function WorkspaceManager.kill_process_prompt(window, pane)
  local wezterm = require('wezterm')
  local act = wezterm.action
  
  window:perform_action(
    act.PromptInputLine {
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
  if not tab then
    return nil
  end
  
  for name, workspace in pairs(workspaces) do
    -- タブオブジェクトを直接比較するか、タイトルで比較
    if workspace.tab then
      local success, tab_id = pcall(function() return workspace.tab:tab_id() end)
      if success and tab_id and tab_id == tab:tab_id() then
        return name
      end
    end
  end
  
  -- タブタイトルからワークスペース名を取得（フォールバック）
  local title = tab:get_title()
  if title then
    local name = title:match("^Workspace: (.+)$")
    if name and workspaces[name] then
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
  
  local process_id = "claude-code-" .. workspace_name
  local command_args = {"--workspace=" .. workspace_name}
  
  return socket_client.spawn_process(workspace_name, process_id, command_args)
end

-- Test connection to backend
function WorkspaceManager.test_connection()
  return socket_client.test_connection()
end

-- Get backend status
function WorkspaceManager.get_backend_status()
  return socket_client.get_backend_status()
end

-- List all workspaces
function WorkspaceManager.list_workspaces()
  return workspaces
end


-- Delete workspace prompt
function WorkspaceManager.delete_workspace_prompt(window, pane)
  local wezterm = require('wezterm')
  local act = wezterm.action
  local choices = {}
  
  for name, ws in pairs(workspaces) do
    if name ~= current_workspace then  -- 現在のワークスペースは削除不可
      table.insert(choices, {
        id = name,
        label = name,
      })
    end
  end
  
  if #choices == 0 then
    wezterm.log_info("No other workspaces to delete")
    return
  end
  
  window:perform_action(
    act.InputSelector {
      action = wezterm.action_callback(function(inner_window, inner_pane, id, label)
        if id and workspaces[id] then
          -- タブがある場合は閉じる
          if workspaces[id].tab then
            -- タブを閉じるアクションを実行
            inner_window:perform_action(
              wezterm.action.CloseCurrentTab { confirm = false },
              workspaces[id].pane
            )
          end
          
          -- ワークスペースを削除
          workspaces[id] = nil
          
          wezterm.log_info("Workspace deleted: " .. id)
        end
      end),
      title = 'Delete Workspace',
      choices = choices,
      fuzzy = true,
    },
    pane
  )
end


return WorkspaceManager