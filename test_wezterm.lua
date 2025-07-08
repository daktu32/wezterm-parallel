-- WezTerm Multi-Process Development Framework - テスト用設定
-- 本番環境に影響しない独立したテスト設定

local wezterm = require 'wezterm'

-- プロジェクトのLuaモジュールへのパスを追加
package.path = package.path .. ';/Users/aiq/work/wezterm-parallel/lua/?.lua;/Users/aiq/work/wezterm-parallel/lua/?/init.lua'

-- フレームワークモジュールを読み込み
local room_manager = require 'room.manager'
local ui_manager = require 'ui.manager'
local dashboard = require 'ui.dashboard'
local pane_manager = require 'ui.pane_manager'
local keybindings = require 'config.keybindings'

local config = wezterm.config_builder()

-- テスト専用のフレームワーク設定
local framework_config = {
  socket_path = "/tmp/wezterm-parallel-test.sock", -- テスト用ソケット
  auto_start_backend = false, -- 手動起動でテスト
  max_workspaces = 4,
  max_processes_per_workspace = 8,
  debug_mode = true, -- デバッグモード有効
  
  -- ダッシュボード設定
  dashboard = {
    update_interval = 3.0, -- テスト用に少し長めに
    width_percentage = 25, -- 小さめに設定
    position = "right",
    theme = {
      background = "#2d3748",
      foreground = "#e2e8f0",
      border = "#4a5568",
      header = "#63b3ed",
      success = "#68d391",
      warning = "#fbd38d",
      error = "#fc8181",
      info = "#81e6d9",
    }
  },
  
  -- ペインマネージャー設定
  pane_manager = {
    auto_balance = true,
    focus_follows_process = true,
    pane_titles_enabled = true,
  },
  
  -- キーバインド設定（テスト用にシンプルに）
  keybindings = {
    leader_key = { key = 'a', mods = 'CTRL' }, -- Ctrl+A をリーダーキーに
    workspace_prefix = 'CTRL|SHIFT',
    process_prefix = 'CTRL|ALT',
    pane_prefix = 'ALT',
    dashboard_prefix = 'CTRL|SHIFT',
  }
}

-- モジュール初期化
pcall(function()
  room_manager.init(framework_config)
  ui_manager.init(framework_config)
  dashboard.init(framework_config)
  pane_manager.init(framework_config)
  pane_manager.init_template_features(framework_config)
  keybindings.init(framework_config)
end)

-- 基本的なWezTerm設定（テスト用）
config.color_scheme = 'Tomorrow Night Blue'
config.font = wezterm.font('Menlo', { weight = 'Regular' })
config.font_size = 11.0

-- ウィンドウタイトルにテストマークを追加
config.window_frame = {
  active_titlebar_bg = '#2d3748',
  inactive_titlebar_bg = '#2d3748',
}

-- タブバー設定
config.tab_bar_at_bottom = true
config.use_fancy_tab_bar = false
config.tab_and_split_indices_are_zero_based = true

-- テスト用のキーバインド設定
config.keys = {}

-- ダッシュボード切り替え（主要機能テスト）
table.insert(config.keys, {
  key = 'd',
  mods = 'CTRL|SHIFT',
  action = wezterm.action_callback(function(window, pane)
    if dashboard and dashboard.toggle then
      dashboard.toggle(window, pane)
    else
      window:toast_notification("Test WezTerm", "Dashboard module not loaded", nil, 2000)
    end
  end),
})

-- ダッシュボード更新
table.insert(config.keys, {
  key = 'r',
  mods = 'CTRL|SHIFT',
  action = wezterm.action_callback(function(window, pane)
    if dashboard and dashboard.update_data then
      dashboard.update_data()
      window:toast_notification("Test WezTerm", "Dashboard refreshed", nil, 1000)
    else
      window:toast_notification("Test WezTerm", "Dashboard not available", nil, 2000)
    end
  end),
})

-- バックエンド接続テスト
table.insert(config.keys, {
  key = 't',
  mods = 'CTRL|SHIFT',
  action = wezterm.action_callback(function(window, pane)
    if room_manager and room_manager.test_connection then
      local success = room_manager.test_connection()
      local message = success and "Backend connection: OK" or "Backend connection: FAILED"
      window:toast_notification("Test WezTerm", message, nil, 3000)
    else
      window:toast_notification("Test WezTerm", "Room manager not loaded", nil, 2000)
    end
  end),
})

-- ヘルプ表示
table.insert(config.keys, {
  key = 'h',
  mods = 'CTRL|SHIFT',
  action = wezterm.action_callback(function(window, pane)
    local help_text = [[
=== WezTerm Parallel テスト環境 ===

テスト用キーバインド:
Ctrl+Shift+D  - ダッシュボード切り替え
Ctrl+Shift+R  - ダッシュボード更新
Ctrl+Shift+T  - バックエンド接続テスト
Ctrl+Shift+H  - このヘルプ表示
Ctrl+Shift+Q  - テスト終了

注意: これはテスト環境です
本番のWezTerm設定には影響しません
    ]]
    
    local tab = window:spawn_tab {}
    local help_pane = tab:active_pane()
    help_pane:inject_output(help_text)
  end),
})

-- テスト終了
table.insert(config.keys, {
  key = 'q',
  mods = 'CTRL|SHIFT',
  action = wezterm.action_callback(function(window, pane)
    window:toast_notification("Test WezTerm", "テスト終了 - ウィンドウを閉じてください", nil, 3000)
  end),
})

-- 標準的なキーバインド（最小限）
table.insert(config.keys, {
  key = 'c',
  mods = 'CTRL|SHIFT',
  action = wezterm.action.CopyTo 'Clipboard',
})

table.insert(config.keys, {
  key = 'v',
  mods = 'CTRL|SHIFT',
  action = wezterm.action.PasteFrom 'Clipboard',
})

-- イベントハンドラー（テスト用）
wezterm.on('gui-startup', function(cmd)
  wezterm.log_info("Test WezTerm started with parallel framework")
  
  -- テスト用の初期化
  if framework_config.debug_mode then
    wezterm.log_info("Debug mode enabled for testing")
  end
end)

wezterm.on('window-config-reloaded', function(window, pane)
  window:toast_notification("Test WezTerm", "Configuration reloaded", nil, 1000)
end)

-- エラーハンドリング
wezterm.on('format-tab-title', function(tab, tabs, panes, config, hover, max_width)
  local title = tab.active_pane.title
  return {
    { Text = "[TEST] " .. title },
  }
end)

return config