-- WezTerm Parallel - クイックスタート設定
-- 最小構成でのWezTerm統合

local wezterm = require 'wezterm'
local config = {}

-- 基本設定
config.font = wezterm.font('JetBrains Mono')
config.font_size = 12.0
config.color_scheme = 'Tomorrow Night'

-- WezTerm Parallel 基本統合
local function call_wtp_api(endpoint, data)
  local url = "http://localhost:8080" .. endpoint
  local curl_cmd = string.format(
    'curl -s -X POST "%s" -H "Content-Type: application/json" -d \'%s\'',
    url, data or '{}'
  )
  return os.execute(curl_cmd)
end

-- アクション定義
local wtp_actions = {
  -- 新しいワークスペース作成
  new_workspace = wezterm.action_callback(function()
    wezterm.log_info("Creating new workspace...")
    call_wtp_api("/api/workspaces", '{"name":"workspace-' .. os.time() .. '","template":"basic"}')
  end),
  
  -- ダッシュボードを開く
  open_dashboard = wezterm.action_callback(function()
    wezterm.log_info("Opening dashboard...")
    os.execute("open http://localhost:8081 2>/dev/null || xdg-open http://localhost:8081 2>/dev/null || start http://localhost:8081")
  end),
  
  -- フレームワーク状態確認
  check_status = wezterm.action_callback(function()
    wezterm.log_info("Checking WTP status...")
    os.execute("curl -s http://localhost:8080/api/status || echo 'WezTerm Parallel not running'")
  end),
}

-- キーバインド設定
config.keys = {
  -- WezTerm Parallel 操作
  { key = 'n', mods = 'CTRL|SHIFT', action = wtp_actions.new_workspace },
  { key = 'd', mods = 'CTRL|SHIFT', action = wtp_actions.open_dashboard },
  { key = 's', mods = 'CTRL|ALT',   action = wtp_actions.check_status },
  
  -- 基本的なWezTerm操作
  { key = 't', mods = 'CTRL|SHIFT', action = wezterm.action.SpawnTab 'CurrentPaneDomain' },
  { key = 'w', mods = 'CTRL|SHIFT', action = wezterm.action.CloseCurrentTab{confirm=false} },
  { key = 'h', mods = 'CTRL|SHIFT', action = wezterm.action.SplitHorizontal{domain='CurrentPaneDomain'} },
  { key = 'v', mods = 'CTRL|SHIFT', action = wezterm.action.SplitVertical{domain='CurrentPaneDomain'} },
}

-- スタートアップメッセージ
wezterm.on('gui-startup', function()
  wezterm.log_info("WezTerm Parallel integration loaded")
  wezterm.log_info("Available keybinds:")
  wezterm.log_info("  Ctrl+Shift+N: New workspace")
  wezterm.log_info("  Ctrl+Shift+D: Open dashboard") 
  wezterm.log_info("  Ctrl+Alt+S:   Check status")
end)

return config