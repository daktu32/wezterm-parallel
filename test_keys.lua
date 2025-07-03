-- キーバインドテスト用スクリプト
local wezterm = require 'wezterm'

-- 現在の設定を読み込む
package.path = package.path .. ';/Users/aiq/work/wezterm-parallel/lua/?.lua'
local config = require('config.init')

print("=== WezTerm Parallel Framework Key Bindings ===")
print("Total keys: " .. #config.keys)
print("")

-- Ctrl+Shift系のキーバインドを探す
print("Ctrl+Shift keybindings:")
for i, key in ipairs(config.keys) do
  if key.mods == "CTRL|SHIFT" then
    print(string.format("  %s + %s", key.mods, key.key))
  end
end

print("")
print("First 10 keybindings:")
for i = 1, math.min(10, #config.keys) do
  local key = config.keys[i]
  print(string.format("%d. %s + %s", i, key.mods or "NONE", key.key))
end