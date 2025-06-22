-- WezTerm Multi-Process Development Framework Integration Test
-- テスト用の最小限な設定ファイル

local wezterm = require 'wezterm'
local config = wezterm.config_builder()

-- ====================================================================================
-- 基本設定
-- ====================================================================================

config.font = wezterm.font("Monaco")
config.font_size = 12.0
config.automatically_reload_config = true

-- ====================================================================================
-- フレームワーク統合テスト
-- ====================================================================================

-- フレームワークパスを設定（テスト用）
local framework_base = wezterm.home_dir .. '/work/wezterm-parallel'

-- フレームワークが利用可能かテスト
local function test_framework_availability()
    wezterm.log_info("Testing WezTerm Multi-Process Framework availability...")
    
    -- Luaモジュールパスの確認
    local lua_path = framework_base .. '/lua'
    local test_files = {
        lua_path .. '/config/init.lua',
        lua_path .. '/ui/dashboard.lua',
        lua_path .. '/workspace/manager.lua'
    }
    
    for _, file in ipairs(test_files) do
        local f = io.open(file, 'r')
        if f then
            f:close()
            wezterm.log_info("✓ Found: " .. file)
        else
            wezterm.log_warn("✗ Missing: " .. file)
        end
    end
    
    -- 設定ファイルの確認
    local config_file = framework_base .. '/config/templates/framework.yaml'
    local f = io.open(config_file, 'r')
    if f then
        f:close()
        wezterm.log_info("✓ Framework configuration found: " .. config_file)
    else
        wezterm.log_warn("✗ Framework configuration missing: " .. config_file)
    end
    
    wezterm.log_info("Framework availability test completed")
end

-- ====================================================================================
-- テスト用キーバインド
-- ====================================================================================

config.keys = {
    -- フレームワークテスト実行
    {
        key = 'F1',
        action = wezterm.action_callback(function()
            test_framework_availability()
            return false
        end),
    },
    
    -- システム情報表示
    {
        key = 'F2',
        action = wezterm.action_callback(function()
            wezterm.log_info("System Info Test:")
            wezterm.log_info("OS: " .. wezterm.target_triple)
            wezterm.log_info("WezTerm Version: " .. wezterm.version)
            wezterm.log_info("Home Directory: " .. wezterm.home_dir)
            return false
        end),
    },
    
    -- ログ表示切り替え
    {
        key = 'F3',
        action = wezterm.action.ShowDebugOverlay,
    },
}

-- ====================================================================================
-- イベントハンドラ
-- ====================================================================================

-- 起動時にフレームワークテストを実行
wezterm.on('gui-startup', function()
    wezterm.log_info("WezTerm Multi-Process Framework Integration Test Started")
    test_framework_availability()
    
    -- テスト用のタブ/ペインを作成
    local tab, pane, window = wezterm.mux.spawn_window({
        workspace = 'wezterm-multi-dev-test',
    })
    
    -- 追加ペインを作成してテスト
    local pane2 = pane:split({
        direction = 'Right',
        size = 0.5,
    })
    
    wezterm.log_info("Test workspace 'wezterm-multi-dev-test' created with 2 panes")
end)

-- エラーハンドリング
wezterm.on('window-config-reloaded', function()
    wezterm.log_info("Configuration reloaded - framework integration test ready")
end)

return config