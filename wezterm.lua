-- WezTerm Multi-Process Development Framework - Main Configuration Entry Point

-- Add the lua directory to the package path
package.path = package.path .. ";./lua/?.lua"

return require 'lua.config.init'