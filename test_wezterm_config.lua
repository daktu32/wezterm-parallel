-- Test WezTerm configuration file
-- This loads the wezterm-parallel framework configuration

-- Add the project's lua directory to the package path
package.path = package.path .. ';/Users/aiq/work/wezterm-parallel/lua/?.lua'

-- Load the main configuration
return require('config.init')