#!/usr/bin/env lua

-- Simple test script for WezTerm Lua integration components
-- This tests the individual modules without WezTerm dependencies

print("Testing WezTerm Parallel Lua Integration...")

-- Mock WezTerm for testing
_G.wezterm = {
  log_info = function(msg) print("[INFO] " .. msg) end,
  log_error = function(msg) print("[ERROR] " .. msg) end,
  log_warn = function(msg) print("[WARN] " .. msg) end,
  run_child_process = function(cmd, opts) 
    return {
      exit_code = 0,
      stdout = '{"type":"pong","timestamp":' .. os.time() .. '}',
      stderr = nil
    }
  end,
  sleep_ms = function(ms) 
    -- Simple sleep implementation for testing
    local start = os.clock()
    while os.clock() - start < ms / 1000 do end
  end
}

-- Add lua directory to path
package.path = package.path .. ";./lua/?.lua"

-- Test JSON utility
print("\n1. Testing JSON utility...")
local json = require 'lua.utils.json'
local test_data = {name = "test", value = 42}
local encoded = json.encode(test_data)
print("Encoded: " .. encoded)
local decoded = json.decode(encoded)
print("Decoded name: " .. decoded.name)
print("✓ JSON utility works")

-- Test socket client
print("\n2. Testing socket client...")
local socket_client = require 'lua.utils.socket_client'
socket_client.init({
  socket_path = "/tmp/wezterm-parallel-test.sock",
  timeout = 1000
})

-- Test configuration
local config = socket_client.get_config()
print("Socket path: " .. config.socket_path)
print("Timeout: " .. config.timeout)
print("✓ Socket client initialized")

-- Test workspace manager (basic initialization)
print("\n3. Testing workspace manager...")
local workspace_manager = require 'lua.workspace.manager'
workspace_manager.init({
  socket_path = "/tmp/wezterm-parallel-test.sock",
  debug_mode = true
})
print("✓ Workspace manager initialized")

print("\n✓ All basic Lua components loaded successfully!")
print("\nNext steps:")
print("1. Start the Rust backend: cargo run")
print("2. Test with actual WezTerm configuration")
print("3. Use keybindings: Ctrl+Shift+t (test connection), Ctrl+Shift+s (status)")