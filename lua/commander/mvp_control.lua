local wezterm = require 'wezterm'
local M = {}

-- Send a command using the bridge
function M.send_command(cmd)
  if not M.bridge then
    wezterm.log_error('Commander bridge not initialized')
    return
  end
  M.bridge:send(cmd)
end

return M
