local wezterm = require 'wezterm'
local ffi = require 'ffi'
local M = {}
M.__index = M

function M.new(store_path)
  local self = setmetatable({}, M)
  self.store_path = store_path or wezterm.home_dir .. '/.wezterm_tasks.json'
  return self
end

function M:send(cmd)
  local f = io.open(self.store_path, 'a+')
  if not f then
    wezterm.log_error('Unable to open task store: ' .. self.store_path)
    return
  end
  local task = { command = cmd, id = os.time() }
  local json = wezterm.json_encode(task)
  f:write(json .. '\n')
  f:close()
end

return M
