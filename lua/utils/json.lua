-- Simple JSON encoder/decoder for WezTerm Lua
-- Provides basic JSON functionality for IPC communication

local json = {}

-- JSON encode function
function json.encode(value)
  local value_type = type(value)
  
  if value_type == "nil" then
    return "null"
  elseif value_type == "boolean" then
    return value and "true" or "false"
  elseif value_type == "number" then
    return tostring(value)
  elseif value_type == "string" then
    return json.encode_string(value)
  elseif value_type == "table" then
    return json.encode_table(value)
  else
    error("Cannot encode value of type: " .. value_type)
  end
end

-- JSON decode function
function json.decode(str)
  if not str or str == "" then
    return nil
  end
  
  -- Remove whitespace
  str = str:match("^%s*(.-)%s*$")
  
  if str == "null" then
    return nil
  elseif str == "true" then
    return true
  elseif str == "false" then
    return false
  elseif str:match("^%-?%d+%.?%d*$") then
    return tonumber(str)
  elseif str:match("^\".*\"$") then
    return json.decode_string(str)
  elseif str:match("^{.*}$") then
    return json.decode_object(str)
  elseif str:match("^%[.*%]$") then
    return json.decode_array(str)
  else
    error("Cannot decode JSON string: " .. str)
  end
end

-- Encode string with escaping
function json.encode_string(str)
  local escaped = str:gsub("\\", "\\\\")
                    :gsub("\"", "\\\"")
                    :gsub("\n", "\\n")
                    :gsub("\r", "\\r")
                    :gsub("\t", "\\t")
  return "\"" .. escaped .. "\""
end

-- Decode string with unescaping
function json.decode_string(str)
  -- Remove quotes
  str = str:sub(2, -2)
  
  -- Unescape
  local unescaped = str:gsub("\\\\", "\\")
                      :gsub("\\\"", "\"")
                      :gsub("\\n", "\n")
                      :gsub("\\r", "\r")
                      :gsub("\\t", "\t")
  return unescaped
end

-- Encode table as JSON object or array
function json.encode_table(tbl)
  -- Check if it's an array (consecutive integer keys starting from 1)
  local is_array = true
  local count = 0
  local max_key = 0
  
  for k, v in pairs(tbl) do
    count = count + 1
    if type(k) ~= "number" or k ~= math.floor(k) or k < 1 then
      is_array = false
      break
    end
    max_key = math.max(max_key, k)
  end
  
  if is_array and count == max_key then
    -- Encode as array
    local parts = {}
    for i = 1, max_key do
      table.insert(parts, json.encode(tbl[i]))
    end
    return "[" .. table.concat(parts, ",") .. "]"
  else
    -- Encode as object
    local parts = {}
    for k, v in pairs(tbl) do
      local key_str = json.encode_string(tostring(k))
      local value_str = json.encode(v)
      table.insert(parts, key_str .. ":" .. value_str)
    end
    return "{" .. table.concat(parts, ",") .. "}"
  end
end

-- Decode JSON object
function json.decode_object(str)
  -- Remove braces
  str = str:sub(2, -2)
  
  if str:match("^%s*$") then
    return {}
  end
  
  local result = {}
  local pos = 1
  
  while pos <= #str do
    -- Skip whitespace
    while pos <= #str and str:sub(pos, pos):match("%s") do
      pos = pos + 1
    end
    
    if pos > #str then
      break
    end
    
    -- Parse key
    local key_start = pos
    if str:sub(pos, pos) == "\"" then
      pos = pos + 1
      while pos <= #str and str:sub(pos, pos) ~= "\"" do
        if str:sub(pos, pos) == "\\" then
          pos = pos + 1 -- Skip escaped character
        end
        pos = pos + 1
      end
      pos = pos + 1 -- Skip closing quote
    end
    
    local key_str = str:sub(key_start, pos - 1)
    local key = json.decode(key_str)
    
    -- Skip whitespace and colon
    while pos <= #str and (str:sub(pos, pos):match("%s") or str:sub(pos, pos) == ":") do
      pos = pos + 1
    end
    
    -- Parse value
    local value_start = pos
    local depth = 0
    local in_string = false
    
    while pos <= #str do
      local char = str:sub(pos, pos)
      
      if char == "\\" and in_string then
        pos = pos + 1 -- Skip escaped character
      elseif char == "\"" then
        in_string = not in_string
      elseif not in_string then
        if char == "{" or char == "[" then
          depth = depth + 1
        elseif char == "}" or char == "]" then
          depth = depth - 1
        elseif char == "," and depth == 0 then
          break
        end
      end
      
      pos = pos + 1
    end
    
    local value_str = str:sub(value_start, pos - 1):match("^%s*(.-)%s*$")
    local value = json.decode(value_str)
    
    result[key] = value
    
    -- Skip comma
    if pos <= #str and str:sub(pos, pos) == "," then
      pos = pos + 1
    end
  end
  
  return result
end

-- Decode JSON array
function json.decode_array(str)
  -- Remove brackets
  str = str:sub(2, -2)
  
  if str:match("^%s*$") then
    return {}
  end
  
  local result = {}
  local pos = 1
  
  while pos <= #str do
    -- Skip whitespace
    while pos <= #str and str:sub(pos, pos):match("%s") do
      pos = pos + 1
    end
    
    if pos > #str then
      break
    end
    
    -- Parse value
    local value_start = pos
    local depth = 0
    local in_string = false
    
    while pos <= #str do
      local char = str:sub(pos, pos)
      
      if char == "\\" and in_string then
        pos = pos + 1 -- Skip escaped character
      elseif char == "\"" then
        in_string = not in_string
      elseif not in_string then
        if char == "{" or char == "[" then
          depth = depth + 1
        elseif char == "}" or char == "]" then
          depth = depth - 1
        elseif char == "," and depth == 0 then
          break
        end
      end
      
      pos = pos + 1
    end
    
    local value_str = str:sub(value_start, pos - 1):match("^%s*(.-)%s*$")
    local value = json.decode(value_str)
    
    table.insert(result, value)
    
    -- Skip comma
    if pos <= #str and str:sub(pos, pos) == "," then
      pos = pos + 1
    end
  end
  
  return result
end

-- Pretty print JSON with indentation
function json.pretty(value, indent)
  indent = indent or 0
  local indent_str = string.rep("  ", indent)
  local value_type = type(value)
  
  if value_type == "table" then
    -- Check if it's an array
    local is_array = true
    local count = 0
    local max_key = 0
    
    for k, v in pairs(value) do
      count = count + 1
      if type(k) ~= "number" or k ~= math.floor(k) or k < 1 then
        is_array = false
        break
      end
      max_key = math.max(max_key, k)
    end
    
    if is_array and count == max_key then
      -- Pretty print array
      local parts = {}
      table.insert(parts, "[")
      for i = 1, max_key do
        local prefix = i == 1 and "\n" .. string.rep("  ", indent + 1) or ",\n" .. string.rep("  ", indent + 1)
        table.insert(parts, prefix .. json.pretty(value[i], indent + 1))
      end
      table.insert(parts, "\n" .. indent_str .. "]")
      return table.concat(parts)
    else
      -- Pretty print object
      local parts = {}
      table.insert(parts, "{")
      
      local keys = {}
      for k in pairs(value) do
        table.insert(keys, k)
      end
      table.sort(keys)
      
      for i, k in ipairs(keys) do
        local prefix = i == 1 and "\n" .. string.rep("  ", indent + 1) or ",\n" .. string.rep("  ", indent + 1)
        local key_str = json.encode_string(tostring(k))
        local value_str = json.pretty(value[k], indent + 1)
        table.insert(parts, prefix .. key_str .. ": " .. value_str)
      end
      
      table.insert(parts, "\n" .. indent_str .. "}")
      return table.concat(parts)
    end
  else
    return json.encode(value)
  end
end

return json