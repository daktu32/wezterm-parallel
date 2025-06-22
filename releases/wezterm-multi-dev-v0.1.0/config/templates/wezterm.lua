-- WezTerm Multi-Process Development Framework Configuration
-- このファイルを ~/.config/wezterm/wezterm.lua にコピーしてください

local wezterm = require 'wezterm'
local config = wezterm.config_builder()

-- ====================================================================================
-- フレームワーク統合設定
-- ====================================================================================

-- フレームワークのLuaモジュールパスを追加
local framework_path = wezterm.home_dir .. '/.config/wezterm-multi-dev/lua'
package.path = package.path .. ';' .. framework_path .. '/?.lua'

-- フレームワークモジュールの読み込み
local dashboard = require 'ui.dashboard_enhanced'
local pane_manager = require 'ui.pane_enhanced'
local task_manager = require 'ui.task_manager'
local log_viewer = require 'ui.log_viewer'

-- フレームワーク設定
local framework_config = {
  -- ダッシュボード設定
  dashboard = {
    enabled = true,
    update_interval = 5000,  -- 5秒間隔で更新
    theme = "catppuccin",
    panels = {
      system_metrics = true,
      process_list = true,
      log_viewer = true,
      task_summary = true,
    },
  },
  
  -- ペイン管理設定
  pane_enhanced = {
    enabled = true,
    auto_optimize = true,
    smart_suggestions = true,
    keyboard_shortcuts = true,
    visual_indicators = true,
    performance_monitoring = true,
  },
  
  -- タスク管理設定
  task_manager = {
    enabled = true,
    auto_save = true,
    time_tracking = true,
    notifications = true,
    show_progress_bars = true,
  },
  
  -- ログ管理設定
  log_viewer = {
    enabled = true,
    auto_scroll = true,
    max_entries = 10000,
    search_enabled = true,
  },
}

-- フレームワーク初期化
dashboard.init(framework_config)
pane_manager.init(framework_config)
task_manager.init(framework_config)

-- ====================================================================================
-- WezTerm基本設定
-- ====================================================================================

-- カラースキーム
config.color_scheme = 'Catppuccin Mocha'

-- フォント設定
config.font = wezterm.font('JetBrains Mono', { weight = 'Medium' })
config.font_size = 13
config.line_height = 1.2

-- ウィンドウ設定
config.window_background_opacity = 0.95
config.window_decorations = "RESIZE"
config.window_close_confirmation = "NeverPrompt"

-- タブバー設定
config.use_fancy_tab_bar = true
config.tab_bar_at_bottom = false
config.tab_max_width = 32
config.show_tab_index_in_tab_bar = true

-- ペイン設定
config.inactive_pane_hsb = {
  saturation = 0.8,
  brightness = 0.6,
}

-- ====================================================================================
-- キーバインド設定
-- ====================================================================================

config.keys = {
  -- ====================================================================================
  -- フレームワーク機能
  -- ====================================================================================
  
  -- ダッシュボード表示
  { key = 'd', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    dashboard.show_dashboard(window, pane)
  end)},
  
  -- ペイン管理メニュー
  { key = 'p', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    pane_manager.show_management_menu(window, pane)
  end)},
  
  -- タスク管理
  { key = 't', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    task_manager.show_task_list(window, pane)
  end)},
  
  -- クイックタスク追加
  { key = 'a', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    task_manager.show_quick_add_dialog(window, pane)
  end)},
  
  -- ペイン同期トグル
  { key = 's', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    local tab = window:active_tab()
    local pane_sync = require 'ui.pane_sync'
    local enabled = pane_sync.toggle_tab_sync(tab)
    
    local status = enabled and "有効" or "無効"
    window:toast_notification("ペイン同期", "ペイン同期を" .. status .. "にしました", nil, 2000)
  end)},
  
  -- レイアウト選択
  { key = 'l', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    local layout_manager = require 'ui.layout_manager'
    layout_manager.show_layout_selector(window, pane)
  end)},
  
  -- ログビューア
  { key = 'g', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    log_viewer.show_log_viewer(window, pane)
  end)},
  
  -- メトリクス表示
  { key = 'm', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    dashboard.show_metrics_only(window, pane)
  end)},
  
  -- カンバンボード表示
  { key = 'k', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    task_manager.show_kanban_board(window, pane)
  end)},
  
  -- 時間追跡開始/停止
  { key = 'r', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    task_manager.toggle_time_tracking(window, pane)
  end)},
  
  -- ヘルプ表示
  { key = 'h', mods = 'CTRL|SHIFT', action = wezterm.action_callback(function(window, pane)
    pane_manager.show_help(window, pane)
  end)},
  
  -- ====================================================================================
  -- ペイン操作
  -- ====================================================================================
  
  -- ペイン分割
  { key = 'Enter', mods = 'ALT', action = wezterm.action.SplitHorizontal { domain = 'CurrentPaneDomain' } },
  { key = 'Enter', mods = 'ALT|SHIFT', action = wezterm.action.SplitVertical { domain = 'CurrentPaneDomain' } },
  
  -- ペイン移動
  { key = 'h', mods = 'ALT', action = wezterm.action.ActivatePaneDirection 'Left' },
  { key = 'j', mods = 'ALT', action = wezterm.action.ActivatePaneDirection 'Down' },
  { key = 'k', mods = 'ALT', action = wezterm.action.ActivatePaneDirection 'Up' },
  { key = 'l', mods = 'ALT', action = wezterm.action.ActivatePaneDirection 'Right' },
  
  -- ペインサイズ調整
  { key = 'h', mods = 'ALT|SHIFT', action = wezterm.action.AdjustPaneSize { 'Left', 5 } },
  { key = 'j', mods = 'ALT|SHIFT', action = wezterm.action.AdjustPaneSize { 'Down', 5 } },
  { key = 'k', mods = 'ALT|SHIFT', action = wezterm.action.AdjustPaneSize { 'Up', 5 } },
  { key = 'l', mods = 'ALT|SHIFT', action = wezterm.action.AdjustPaneSize { 'Right', 5 } },
  
  -- ペイン削除
  { key = 'x', mods = 'ALT', action = wezterm.action.CloseCurrentPane { confirm = true } },
  
  -- ペインズーム
  { key = 'z', mods = 'ALT', action = wezterm.action.TogglePaneZoomState },
  
  -- ====================================================================================
  -- タブ操作
  -- ====================================================================================
  
  -- 新しいタブ
  { key = 't', mods = 'CMD', action = wezterm.action.SpawnTab 'CurrentPaneDomain' },
  { key = 't', mods = 'CTRL', action = wezterm.action.SpawnTab 'CurrentPaneDomain' },
  
  -- タブ移動
  { key = 'Tab', mods = 'CTRL', action = wezterm.action.ActivateTabRelative(1) },
  { key = 'Tab', mods = 'CTRL|SHIFT', action = wezterm.action.ActivateTabRelative(-1) },
  
  -- タブ直接移動 (Alt+数字)
  { key = '1', mods = 'ALT', action = wezterm.action.ActivateTab(0) },
  { key = '2', mods = 'ALT', action = wezterm.action.ActivateTab(1) },
  { key = '3', mods = 'ALT', action = wezterm.action.ActivateTab(2) },
  { key = '4', mods = 'ALT', action = wezterm.action.ActivateTab(3) },
  { key = '5', mods = 'ALT', action = wezterm.action.ActivateTab(4) },
  { key = '6', mods = 'ALT', action = wezterm.action.ActivateTab(5) },
  { key = '7', mods = 'ALT', action = wezterm.action.ActivateTab(6) },
  { key = '8', mods = 'ALT', action = wezterm.action.ActivateTab(7) },
  { key = '9', mods = 'ALT', action = wezterm.action.ActivateTab(8) },
  
  -- タブ削除
  { key = 'w', mods = 'CMD', action = wezterm.action.CloseCurrentTab { confirm = true } },
  { key = 'w', mods = 'CTRL', action = wezterm.action.CloseCurrentTab { confirm = true } },
  
  -- ====================================================================================
  -- その他の操作
  -- ====================================================================================
  
  -- 設定リロード
  { key = 'r', mods = 'CMD|SHIFT', action = wezterm.action.ReloadConfiguration },
  { key = 'r', mods = 'CTRL|SHIFT', action = wezterm.action.ReloadConfiguration },
  
  -- デバッグオーバーレイ
  { key = 'l', mods = 'CMD|SHIFT', action = wezterm.action.ShowDebugOverlay },
  
  -- コマンドパレット
  { key = 'p', mods = 'CMD|SHIFT', action = wezterm.action.ActivateCommandPalette },
  
  -- フォントサイズ調整
  { key = '=', mods = 'CMD', action = wezterm.action.IncreaseFontSize },
  { key = '-', mods = 'CMD', action = wezterm.action.DecreaseFontSize },
  { key = '0', mods = 'CMD', action = wezterm.action.ResetFontSize },
  
  -- フルスクリーン
  { key = 'Enter', mods = 'CMD', action = wezterm.action.ToggleFullScreen },
}

-- ====================================================================================
-- マウス設定
-- ====================================================================================

config.mouse_bindings = {
  -- 右クリックでペースト
  {
    event = { Down = { streak = 1, button = 'Right' } },
    mods = 'NONE',
    action = wezterm.action.PasteFrom 'Clipboard',
  },
  
  -- Ctrl+左クリックでリンクを開く
  {
    event = { Up = { streak = 1, button = 'Left' } },
    mods = 'CTRL',
    action = wezterm.action.OpenLinkAtMouseCursor,
  },
}

-- ====================================================================================
-- ワークスペース設定
-- ====================================================================================

-- ワークスペース自動命名
wezterm.on('format-window-title', function(tab, pane, tabs, panes, config)
  local zoomed = ''
  if tab.active_pane.is_zoomed then
    zoomed = ' [Z]'
  end

  local index = ''
  if #tabs > 1 then
    index = string.format('[%d/%d] ', tab.tab_index + 1, #tabs)
  end

  return index .. tab.active_pane.title .. zoomed
end)

-- タブタイトルのフォーマット
wezterm.on('format-tab-title', function(tab, tabs, panes, config, hover, max_width)
  local title = tab.active_pane.title
  if title and #title > 0 then
    return {
      { Text = ' ' .. title .. ' ' },
    }
  end
  return {
    { Text = ' Terminal ' },
  }
end)

-- ====================================================================================
-- 起動時フック
-- ====================================================================================

wezterm.on('gui-startup', function(cmd)
  -- フレームワークサービスの起動確認
  local success = os.execute('pgrep -f wezterm-multi-dev > /dev/null')
  if not success then
    -- フレームワークサービスが起動していない場合の警告
    wezterm.log_warn('WezTerm Multi-Dev framework service is not running. Please start it manually.')
  end
end)

-- ====================================================================================
-- カスタム設定 (オプション)
-- ====================================================================================

-- 個人設定ファイルが存在する場合は読み込み
local personal_config_path = wezterm.home_dir .. '/.config/wezterm/personal.lua'
local f = io.open(personal_config_path, 'r')
if f then
  f:close()
  local personal_config = dofile(personal_config_path)
  if personal_config then
    for k, v in pairs(personal_config) do
      config[k] = v
    end
  end
end

return config