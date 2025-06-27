-- WezTerm Multi-Process Development Framework - Unix Domain Socket Client
-- Provides communication with the Rust backend through Unix Domain Socket

-- Mock wezterm for compatibility
local wezterm = _G.wezterm or {
  log_info = function(msg) print("[INFO] " .. msg) end,
  log_error = function(msg) print("[ERROR] " .. msg) end,
  run_child_process = function() return {exit_code=1, stderr="Mock implementation"} end,
  sleep_ms = function(ms) end
}

local json = require 'lua.utils.json'

local socket_client = {}

-- Configuration
local config = {
  socket_path = "/tmp/wezterm-parallel.sock",
  timeout = 5000, -- milliseconds
  retry_attempts = 3,
  retry_delay = 1000, -- milliseconds
}

-- Client state
local client_state = {
  connected = false,
  last_error = nil,
  message_id_counter = 0,
  pending_responses = {},
}

-- Initialize the socket client with configuration
function socket_client.init(framework_config)
  if framework_config and framework_config.socket_path then
    config.socket_path = framework_config.socket_path
  end
  
  if framework_config and framework_config.timeout then
    config.timeout = framework_config.timeout
  end
  
  wezterm.log_info("Socket client initialized with path: " .. config.socket_path)
end

-- Get next message ID for request tracking
local function get_next_message_id()
  client_state.message_id_counter = client_state.message_id_counter + 1
  return tostring(client_state.message_id_counter)
end

-- Send a message to the backend and optionally wait for response
function socket_client.send_message(message_type, payload, wait_for_response)
  wait_for_response = wait_for_response or false
  
  local message = {
    id = get_next_message_id(),
    type = message_type,
    payload = payload or {},
    timestamp = os.time(),
  }
  
  local message_json = json.encode(message)
  
  wezterm.log_info("Sending message: " .. message_type .. " (ID: " .. message.id .. ")")
  
  -- Attempt to send message with retries
  for attempt = 1, config.retry_attempts do
    local success, result = pcall(function()
      -- Use WezTerm's spawn API to communicate with Unix socket
      -- Note: This is a simplified implementation. In a real scenario,
      -- you might need to use a more sophisticated approach or external tool
      local cmd = {
        "nc", "-U", config.socket_path
      }
      
      local handle = wezterm.run_child_process(cmd, {
        stdin = message_json,
        timeout = config.timeout / 1000,
      })
      
      if handle.exit_code == 0 then
        client_state.connected = true
        client_state.last_error = nil
        
        if wait_for_response and handle.stdout then
          local response = json.decode(handle.stdout)
          return response
        end
        
        return true
      else
        error("Failed to send message: " .. (handle.stderr or "Unknown error"))
      end
    end)
    
    if success then
      return result
    else
      client_state.last_error = result
      wezterm.log_error("Socket send attempt " .. attempt .. " failed: " .. result)
      
      if attempt < config.retry_attempts then
        wezterm.sleep_ms(config.retry_delay)
      end
    end
  end
  
  client_state.connected = false
  return nil, client_state.last_error
end

-- Workspace management functions
function socket_client.create_workspace(name, template)
  return socket_client.send_message("workspace_create", {
    name = name,
    template = template or "basic"
  }, true)
end

function socket_client.delete_workspace(name)
  return socket_client.send_message("workspace_delete", {
    name = name
  }, true)
end

function socket_client.list_workspaces()
  return socket_client.send_message("workspace_list", {}, true)
end

function socket_client.switch_workspace(name)
  return socket_client.send_message("workspace_switch", {
    name = name
  }, true)
end

function socket_client.get_workspace_status(name)
  return socket_client.send_message("workspace_status", {
    name = name
  }, true)
end

-- Process management functions
function socket_client.spawn_process(workspace_name, process_id, command_args)
  return socket_client.send_message("process_spawn", {
    workspace = workspace_name,
    process_id = process_id,
    command_args = command_args or {}
  }, true)
end

function socket_client.kill_process(process_id)
  return socket_client.send_message("process_kill", {
    process_id = process_id
  }, true)
end

function socket_client.list_processes()
  return socket_client.send_message("process_list", {}, true)
end

function socket_client.get_process_info(process_id)
  return socket_client.send_message("process_info", {
    process_id = process_id
  }, true)
end

-- System monitoring functions
function socket_client.get_system_metrics()
  return socket_client.send_message("metrics_system", {}, true)
end

function socket_client.get_workspace_metrics(workspace_name)
  return socket_client.send_message("metrics_workspace", {
    workspace = workspace_name
  }, true)
end

-- Health check and status functions
function socket_client.ping()
  return socket_client.send_message("ping", {}, true)
end

function socket_client.get_backend_status()
  return socket_client.send_message("status", {}, true)
end

-- Utility functions
function socket_client.is_connected()
  return client_state.connected
end

function socket_client.get_last_error()
  return client_state.last_error
end

function socket_client.test_connection()
  local result = socket_client.ping()
  
  if result and result.type == "pong" then
    wezterm.log_info("Socket connection test successful")
    return true
  else
    wezterm.log_error("Socket connection test failed: " .. (client_state.last_error or "No response"))
    return false
  end
end

-- Configuration getters
function socket_client.get_config()
  return config
end

function socket_client.get_client_state()
  return client_state
end

return socket_client