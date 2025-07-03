-- WezTerm Multi-Process Development Framework - Room Manager
-- Handles room creation, switching, and management

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

local RoomManager = {}

-- Internal state
local config = {}
local current_room = "default"
local rooms = {}
local backend_process = nil

-- Helper function to count table entries
local function table_count(t)
  local count = 0
  for _ in pairs(t) do
    count = count + 1
  end
  return count
end

-- Initialize the room manager
function RoomManager.init(framework_config)
  config = framework_config
  socket_client.init(framework_config)
  wezterm.log_info("RoomManager initialized")
end

-- Start the backend Rust process
function RoomManager.start_backend()
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
function RoomManager.send_to_backend(message)
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

-- Create a new room
function RoomManager.create_room(name, template, window, pane)
  template = template or "basic"
  
  wezterm.log_info(string.format("Creating room: %s with template: %s", name, template))
  
  -- Use socket client to create room
  local response = socket_client.create_room(name, template)
  
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
    new_tab:set_title(string.format("Room: %s", name))
    
    -- Store room info with window reference
    rooms[name] = {
      tab = new_tab,
      template = template,
      created_at = os.time(),
      processes = {},
      window = window,
      pane = new_pane
    }
    
    current_room = name
    
    wezterm.log_info("Room created and tab activated: " .. name)
    
    -- プロジェクトディレクトリに移動
    new_pane:send_text("cd ~/projects/" .. name .. " 2>/dev/null || cd ~\n")
  else
    wezterm.log_error("Window not available for room creation")
  end
  
  return response
end

-- Switch to an existing room
function RoomManager.switch_room(name)
  if not rooms[name] then
    wezterm.log_error("Room not found: " .. name)
    return false
  end
  
  wezterm.log_info("Switching to room: " .. name)
  
  -- 現在のRoomの状態を保存
  RoomManager.save_current_room_state()
  
  local room = rooms[name]
  
  -- 保存されたタブが存在するか確認
  if room.tab then
    -- タブをアクティブ化
    room.tab:activate()
    current_room = name
    
    -- Room状態を復元
    RoomManager.restore_room_state(name)
    
    wezterm.log_info("Activated tab for room: " .. name)
    return true
  elseif room.window then
    -- タブが存在しない場合は新規作成
    local wezterm = require('wezterm')
    room.window:perform_action(
      wezterm.action.SpawnTab 'CurrentPaneDomain',
      room.pane
    )
    
    local new_tab = room.window:active_tab()
    new_tab:set_title(string.format("Room: %s", name))
    
    -- ルーム情報を更新
    room.tab = new_tab
    room.pane = room.window:active_pane()
    
    current_room = name
    wezterm.log_info("Created new tab for room: " .. name)
    
    -- プロジェクトディレクトリに移動
    room.pane:send_text("cd ~/projects/" .. name .. " 2>/dev/null || cd ~\n")
    
    return true
  else
    wezterm.log_error("Window not found for room: " .. name)
  end
  
  return false
end

-- Prompt user to create room
function RoomManager.create_room_prompt(window, pane)
  -- wezterm.actionをrequireで取得
  local wezterm = require('wezterm')
  local act = wezterm.action
  
  window:perform_action(
    act.PromptInputLine {
      description = 'Enter room name:',
      action = wezterm.action_callback(function(inner_window, inner_pane, line)
        if line and line ~= "" then
          -- 実際にルームを作成（タブも作成される）
          RoomManager.create_room(line, "basic", inner_window, inner_pane)
          
          wezterm.log_info("Room created: " .. line)
        end
      end),
    },
    pane
  )
end

-- Prompt user to switch room
function RoomManager.switch_room_prompt(window, pane)
  local wezterm = require('wezterm')
  local act = wezterm.action
  local choices = {}
  
  for name, ws in pairs(rooms) do
    local is_current = (name == current_room) and " [CURRENT]" or ""
    table.insert(choices, {
      id = name,
      label = string.format("%s%s", name, is_current),
    })
  end
  
  if #choices == 0 then
    wezterm.log_info("No rooms available. Create one with Ctrl+Shift+N")
    return
  end
  
  -- ルーム選択ダイアログを表示
  window:perform_action(
    act.InputSelector {
      action = wezterm.action_callback(function(inner_window, inner_pane, id, label)
        if id then
          -- ルームが存在しない場合は新規作成
          if not rooms[id] then
            RoomManager.create_room(id, "basic", inner_window, inner_pane)
          else
            -- 既存のルームに切り替え
            RoomManager.switch_room(id)
          end
          
          wezterm.log_info("Switched to room: " .. id)
        end
      end),
      title = 'Select Room',
      choices = choices,
      fuzzy = true,
    },
    pane
  )
end

-- Kill process prompt
function RoomManager.kill_process_prompt(window, pane)
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
          RoomManager.send_to_backend(message)
        end
      end),
    },
    pane
  )
end

-- Get current room name for tab title
function RoomManager.get_current_room_name(tab)
  if not tab then
    return nil
  end
  
  for name, room in pairs(rooms) do
    -- タブオブジェクトを直接比較するか、タイトルで比較
    if room.tab then
      local success, tab_id = pcall(function() return room.tab:tab_id() end)
      if success and tab_id and tab_id == tab:tab_id() then
        return name
      end
    end
  end
  
  -- タブタイトルからルーム名を取得（フォールバック）
  local title = tab:get_title()
  if title then
    local name = title:match("^Room: (.+)$")
    if name and rooms[name] then
      return name
    end
  end
  
  return nil
end

-- Ensure default room exists
function RoomManager.ensure_default_room()
  if not rooms["default"] then
    RoomManager.create_room("default", "basic")
  end
end

-- Spawn Claude Code process in current room
function RoomManager.spawn_claude_code(room_name)
  room_name = room_name or current_room
  
  local process_id = "claude-code-" .. room_name
  local command_args = {"--room=" .. room_name}
  
  return socket_client.spawn_process(room_name, process_id, command_args)
end

-- Test connection to backend
function RoomManager.test_connection()
  return socket_client.test_connection()
end

-- Get backend status
function RoomManager.get_backend_status()
  return socket_client.get_backend_status()
end

-- List all rooms
function RoomManager.list_rooms()
  return rooms
end


-- Delete room prompt
function RoomManager.delete_room_prompt(window, pane)
  local wezterm = require('wezterm')
  local act = wezterm.action
  local choices = {}
  
  for name, ws in pairs(rooms) do
    if name ~= current_room then  -- 現在のルームは削除不可
      table.insert(choices, {
        id = name,
        label = name,
      })
    end
  end
  
  if #choices == 0 then
    wezterm.log_info("No other rooms to delete")
    return
  end
  
  window:perform_action(
    act.InputSelector {
      action = wezterm.action_callback(function(inner_window, inner_pane, id, label)
        if id and rooms[id] then
          -- 確認ダイアログを表示
          inner_window:perform_action(
            act.PromptInputLine {
              description = string.format('本当にRoom "%s" を削除しますか？ "yes" と入力してください:', id),
              action = wezterm.action_callback(function(confirm_window, confirm_pane, line)
                if line and line:lower() == "yes" then
                  -- Save deletion record to history
                  local room = rooms[id]
                  if room then
                    room.access_history = room.access_history or {}
                    table.insert(room.access_history, {
                      timestamp = os.time(),
                      duration = 0,
                      action = "delete"
                    })
                  end
                  
                  -- タブがある場合は閉じる
                  if room and room.tab then
                    -- タブを閉じるアクションを実行
                    confirm_window:perform_action(
                      wezterm.action.CloseCurrentTab { confirm = false },
                      room.pane
                    )
                  end
                  
                  -- ルームを削除
                  rooms[id] = nil
                  
                  wezterm.log_info("Room deleted: " .. id)
                else
                  wezterm.log_info("Room deletion cancelled: " .. id)
                end
              end),
            },
            inner_pane
          )
        end
      end),
      title = 'Delete Room (確認が必要)',
      choices = choices,
      fuzzy = true,
    },
    pane
  )
end


-- Save current room state before switching
function RoomManager.save_current_room_state()
  if not current_room or current_room == "" then
    return
  end
  
  local room = rooms[current_room]
  if not room or not room.pane then
    return
  end
  
  -- Get current working directory
  local success, stdout, stderr = pcall(function()
    return room.pane:get_current_working_dir()
  end)
  
  if success and stdout then
    room.saved_cwd = stdout.file_path
  end
  
  -- Save pane title
  if room.tab then
    room.saved_title = room.tab:get_title()
  end
  
  -- Record access for history
  local session_duration = os.time() - (room.session_start or os.time())
  room.access_history = room.access_history or {}
  table.insert(room.access_history, {
    timestamp = os.time(),
    duration = session_duration,
    action = "switch_out"
  })
  
  wezterm.log_info("Saved state for room: " .. current_room)
end

-- Restore room state when switching to it
function RoomManager.restore_room_state(name)
  local room = rooms[name]
  if not room then
    return
  end
  
  -- Record session start time
  room.session_start = os.time()
  
  -- Restore working directory if saved
  if room.saved_cwd and room.pane then
    room.pane:send_text("cd " .. room.saved_cwd .. "\n")
    wezterm.log_info("Restored working directory: " .. room.saved_cwd)
  end
  
  -- Record access for history
  room.access_history = room.access_history or {}
  table.insert(room.access_history, {
    timestamp = os.time(),
    duration = 0,
    action = "switch_in"
  })
  
  -- Increment session count
  room.session_count = (room.session_count or 0) + 1
  
  wezterm.log_info("Restored state for room: " .. name)
end

-- Get room usage statistics
function RoomManager.get_room_stats(name)
  local room = rooms[name]
  if not room then
    return nil
  end
  
  local total_duration = 0
  local access_count = 0
  
  if room.access_history then
    for _, access in ipairs(room.access_history) do
      total_duration = total_duration + (access.duration or 0)
      access_count = access_count + 1
    end
  end
  
  return {
    name = name,
    created_at = room.created_at,
    session_count = room.session_count or 1,
    total_duration = total_duration,
    access_count = access_count,
    last_accessed = room.access_history and room.access_history[#room.access_history] and room.access_history[#room.access_history].timestamp or room.created_at,
    average_session_duration = access_count > 0 and total_duration / access_count or 0
  }
end

-- Get sorted room list
function RoomManager.get_sorted_rooms(sort_by)
  local room_list = {}
  
  for name, room in pairs(rooms) do
    local stats = RoomManager.get_room_stats(name)
    if stats then
      table.insert(room_list, stats)
    end
  end
  
  -- Sort based on criteria
  if sort_by == "last_accessed" then
    table.sort(room_list, function(a, b)
      return a.last_accessed > b.last_accessed
    end)
  elseif sort_by == "name" then
    table.sort(room_list, function(a, b)
      return a.name < b.name
    end)
  elseif sort_by == "created_at" then
    table.sort(room_list, function(a, b)
      return a.created_at > b.created_at
    end)
  elseif sort_by == "session_count" then
    table.sort(room_list, function(a, b)
      return a.session_count > b.session_count
    end)
  end
  
  return room_list
end

return RoomManager