local M = {}

-- initialize commander module
function M.setup(store_path)
  M.bridge = require('wezterm_parallel.commander.bridge').new(store_path)
end

return M
