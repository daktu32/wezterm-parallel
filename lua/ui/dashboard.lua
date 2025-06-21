-- WezTerm Multi-Process Development Framework - Advanced Dashboard UI
-- Enhanced dashboard with real-time updates and interactive controls

local wezterm = require 'wezterm'

local Dashboard = {}

-- Dashboard configuration
local config = {
  update_interval = 1.0, -- seconds
  width_percentage = 35, -- percentage of screen width
  position = "right",    -- "left", "right", "top", "bottom"
  theme = {
    background = "#1e1e2e",
    foreground = "#cdd6f4",
    border = "#45475a",
    header = "#89b4fa",
    success = "#a6e3a1",
    warning = "#f9e2af",
    error = "#f38ba8",
    info = "#89dceb",
  }
}

-- Dashboard state
local state = {
  visible = false,
  pane_id = nil,
  last_update = 0,
  workspaces = {},
  processes = {},
  tasks = {},
  alerts = {},
  system_health = nil,
}

-- Initialize dashboard
function Dashboard.init(framework_config)
  if framework_config.dashboard then
    for k, v in pairs(framework_config.dashboard) do
      config[k] = v
    end
  end
  
  wezterm.log_info("Dashboard initialized with theme: " .. (config.theme.name or "default"))
end

-- Toggle dashboard visibility
function Dashboard.toggle(window, pane)
  if state.visible then
    Dashboard.hide(window, pane)
  else
    Dashboard.show(window, pane)
  end
end

-- Show dashboard
function Dashboard.show(window, pane)
  if state.visible then
    return
  end
  
  state.visible = true
  
  -- Create dashboard pane based on position
  local tab = pane:tab()
  local split_config = Dashboard.get_split_config()
  
  local dashboard_pane = tab:split_pane(split_config)
  state.pane_id = dashboard_pane:pane_id()
  
  -- Set custom title for dashboard pane
  dashboard_pane:inject_output("\x1b]2;WezTerm Multi-Dev Dashboard\x07")
  
  -- Initial render
  Dashboard.render(dashboard_pane)
  
  -- Start update loop
  Dashboard.start_update_loop(window, dashboard_pane)
  
  wezterm.log_info("Dashboard shown")
end

-- Hide dashboard
function Dashboard.hide(window, pane)
  if not state.visible or not state.pane_id then
    return
  end
  
  -- Find and close dashboard pane
  local tab = pane:tab()
  for _, p in ipairs(tab:panes()) do
    if p:pane_id() == state.pane_id then
      p:activate()
      window:perform_action(wezterm.action.CloseCurrentPane { confirm = false }, p)
      break
    end
  end
  
  state.visible = false
  state.pane_id = nil
  
  wezterm.log_info("Dashboard hidden")
end

-- Get split configuration based on position
function Dashboard.get_split_config()
  local size = config.width_percentage / 100
  
  if config.position == "right" then
    return {
      direction = "Right",
      size = size,
    }
  elseif config.position == "left" then
    return {
      direction = "Left",
      size = size,
    }
  elseif config.position == "top" then
    return {
      direction = "Top",
      size = size,
    }
  else -- bottom
    return {
      direction = "Bottom",
      size = size,
    }
  end
end

-- Render dashboard content
function Dashboard.render(pane)
  -- Clear screen
  pane:inject_output("\x1b[2J\x1b[H")
  
  -- Build dashboard content
  local content = Dashboard.build_content()
  
  -- Apply theme colors
  local styled_content = Dashboard.apply_theme(content)
  
  -- Inject content into pane
  pane:inject_output(styled_content)
end

-- Build dashboard content
function Dashboard.build_content()
  local lines = {}
  
  -- Header
  table.insert(lines, Dashboard.build_header())
  table.insert(lines, "")
  
  -- System health
  table.insert(lines, Dashboard.build_system_health())
  table.insert(lines, "")
  
  -- Workspaces section
  table.insert(lines, Dashboard.build_workspaces_section())
  table.insert(lines, "")
  
  -- Processes section
  table.insert(lines, Dashboard.build_processes_section())
  table.insert(lines, "")
  
  -- Tasks section
  table.insert(lines, Dashboard.build_tasks_section())
  table.insert(lines, "")
  
  -- Alerts section
  if #state.alerts > 0 then
    table.insert(lines, Dashboard.build_alerts_section())
    table.insert(lines, "")
  end
  
  -- Footer with keybindings
  table.insert(lines, Dashboard.build_footer())
  
  return table.concat(lines, "\n")
end

-- Build header
function Dashboard.build_header()
  local header = [[
╔═══════════════════════════════════════╗
║     WezTerm Multi-Dev Dashboard       ║
╚═══════════════════════════════════════╝]]
  
  return header
end

-- Build system health section
function Dashboard.build_system_health()
  local health = state.system_health or {
    total_processes = 0,
    responsive_processes = 0,
    avg_cpu_usage = 0,
    total_memory_usage = 0,
    overall_status = "Unknown"
  }
  
  local status_icon = "◉"
  local status_color = "info"
  
  if health.overall_status == "Healthy" then
    status_icon = "●"
    status_color = "success"
  elseif health.overall_status == "Warning" then
    status_icon = "◐"
    status_color = "warning"
  elseif health.overall_status == "Critical" then
    status_icon = "◯"
    status_color = "error"
  end
  
  local lines = {
    "╭─ System Health ────────────────────╮",
    string.format("│ Status: %s %s", Dashboard.colorize(status_icon, status_color), health.overall_status),
    string.format("│ Processes: %d/%d responsive", health.responsive_processes, health.total_processes),
    string.format("│ CPU Usage: %.1f%%", health.avg_cpu_usage),
    string.format("│ Memory: %d MB", health.total_memory_usage),
    "╰────────────────────────────────────╯"
  }
  
  return table.concat(lines, "\n")
end

-- Build workspaces section
function Dashboard.build_workspaces_section()
  local lines = {
    "╭─ Workspaces ───────────────────────╮"
  }
  
  if next(state.workspaces) == nil then
    table.insert(lines, "│ No workspaces                      │")
  else
    for name, workspace in pairs(state.workspaces) do
      local status_icon = workspace.is_active and "▶" or "○"
      local status_color = workspace.is_active and "success" or "foreground"
      local process_count = workspace.process_count or 0
      
      local line = string.format("│ %s %-15s (%d procs)", 
        Dashboard.colorize(status_icon, status_color),
        name:sub(1, 15),
        process_count
      )
      
      -- Pad to fixed width
      line = Dashboard.pad_line(line, 38)
      table.insert(lines, line .. "│")
    end
  end
  
  table.insert(lines, "╰────────────────────────────────────╯")
  
  return table.concat(lines, "\n")
end

-- Build processes section
function Dashboard.build_processes_section()
  local lines = {
    "╭─ Processes ────────────────────────╮"
  }
  
  if next(state.processes) == nil then
    table.insert(lines, "│ No running processes               │")
  else
    -- Sort processes by workspace and ID
    local sorted_processes = {}
    for id, proc in pairs(state.processes) do
      table.insert(sorted_processes, proc)
    end
    table.sort(sorted_processes, function(a, b)
      if a.workspace == b.workspace then
        return a.id < b.id
      end
      return a.workspace < b.workspace
    end)
    
    -- Display up to 10 processes
    local count = 0
    for _, proc in ipairs(sorted_processes) do
      if count >= 10 then
        table.insert(lines, "│ ... and " .. (#sorted_processes - 10) .. " more")
        break
      end
      
      local status_icon = "●"
      local status_color = "success"
      
      if proc.status == "Running" then
        status_icon = "●"
        status_color = "success"
      elseif proc.status == "Busy" then
        status_icon = "◉"
        status_color = "warning"
      elseif proc.status == "Idle" then
        status_icon = "○"
        status_color = "info"
      elseif proc.status == "Failed" then
        status_icon = "✗"
        status_color = "error"
      end
      
      local line = string.format("│ %s %-10s %5s%% %4dMB", 
        Dashboard.colorize(status_icon, status_color),
        proc.id:sub(1, 10),
        tostring(proc.cpu_usage or 0),
        proc.memory_usage or 0
      )
      
      line = Dashboard.pad_line(line, 38)
      table.insert(lines, line .. "│")
      
      count = count + 1
    end
  end
  
  table.insert(lines, "╰────────────────────────────────────╯")
  
  return table.concat(lines, "\n")
end

-- Build tasks section
function Dashboard.build_tasks_section()
  local lines = {
    "╭─ Task Queue ───────────────────────╮"
  }
  
  local queued_count = 0
  local running_count = 0
  
  for _, task in pairs(state.tasks) do
    if task.status == "Queued" then
      queued_count = queued_count + 1
    elseif task.status == "Running" then
      running_count = running_count + 1
    end
  end
  
  if queued_count == 0 and running_count == 0 then
    table.insert(lines, "│ No active tasks                    │")
  else
    table.insert(lines, string.format("│ Running: %d, Queued: %d", running_count, queued_count))
    
    -- Show up to 5 recent tasks
    local recent_tasks = {}
    for _, task in pairs(state.tasks) do
      table.insert(recent_tasks, task)
    end
    table.sort(recent_tasks, function(a, b)
      return (a.created_at or 0) > (b.created_at or 0)
    end)
    
    for i = 1, math.min(5, #recent_tasks) do
      local task = recent_tasks[i]
      local status_icon = "◐"
      local status_color = "info"
      
      if task.status == "Running" then
        status_icon = "▶"
        status_color = "success"
      elseif task.status == "Completed" then
        status_icon = "✓"
        status_color = "success"
      elseif task.status == "Failed" then
        status_icon = "✗"
        status_color = "error"
      end
      
      local line = string.format("│ %s %s", 
        Dashboard.colorize(status_icon, status_color),
        task.command:sub(1, 30)
      )
      
      line = Dashboard.pad_line(line, 38)
      table.insert(lines, line .. "│")
    end
  end
  
  table.insert(lines, "╰────────────────────────────────────╯")
  
  return table.concat(lines, "\n")
end

-- Build alerts section
function Dashboard.build_alerts_section()
  local lines = {
    "╭─ Alerts ───────────────────────────╮"
  }
  
  -- Show up to 5 recent alerts
  local recent_alerts = {}
  for _, alert in ipairs(state.alerts) do
    if not alert.acknowledged then
      table.insert(recent_alerts, alert)
    end
  end
  
  for i = 1, math.min(5, #recent_alerts) do
    local alert = recent_alerts[i]
    local icon = "ℹ"
    local color = "info"
    
    if alert.severity == "Critical" then
      icon = "✗"
      color = "error"
    elseif alert.severity == "Warning" then
      icon = "⚠"
      color = "warning"
    end
    
    local line = string.format("│ %s %s", 
      Dashboard.colorize(icon, color),
      alert.message:sub(1, 32)
    )
    
    line = Dashboard.pad_line(line, 38)
    table.insert(lines, line .. "│")
  end
  
  if #recent_alerts > 5 then
    table.insert(lines, "│ ... and " .. (#recent_alerts - 5) .. " more alerts")
  end
  
  table.insert(lines, "╰────────────────────────────────────╯")
  
  return table.concat(lines, "\n")
end

-- Build footer
function Dashboard.build_footer()
  local footer = [[
╭─ Keybindings ──────────────────────╮
│ Ctrl+Shift+N - New Workspace       │
│ Ctrl+Shift+W - Switch Workspace    │
│ Ctrl+Shift+K - Kill Process        │
│ Ctrl+Shift+D - Toggle Dashboard    │
│ Ctrl+Shift+R - Refresh Dashboard   │
╰────────────────────────────────────╯]]
  
  return footer
end

-- Apply theme colors
function Dashboard.apply_theme(content)
  -- Apply background color
  local styled = string.format("\x1b[48;2;%d;%d;%dm", 
    Dashboard.hex_to_rgb(config.theme.background))
  
  -- Apply foreground color
  styled = styled .. string.format("\x1b[38;2;%d;%d;%dm", 
    Dashboard.hex_to_rgb(config.theme.foreground))
  
  -- Add content
  styled = styled .. content
  
  -- Reset colors
  styled = styled .. "\x1b[0m"
  
  return styled
end

-- Colorize text based on theme
function Dashboard.colorize(text, color_name)
  local color = config.theme[color_name] or config.theme.foreground
  local r, g, b = Dashboard.hex_to_rgb(color)
  return string.format("\x1b[38;2;%d;%d;%dm%s\x1b[39m", r, g, b, text)
end

-- Convert hex color to RGB
function Dashboard.hex_to_rgb(hex)
  hex = hex:gsub("#", "")
  return tonumber("0x" .. hex:sub(1, 2)),
         tonumber("0x" .. hex:sub(3, 4)),
         tonumber("0x" .. hex:sub(5, 6))
end

-- Pad line to fixed width
function Dashboard.pad_line(line, width)
  -- Remove ANSI escape sequences for length calculation
  local clean_line = line:gsub("\x1b%[[%d;]*m", "")
  local current_length = #clean_line
  
  if current_length < width then
    return line .. string.rep(" ", width - current_length)
  else
    return line
  end
end

-- Start update loop
function Dashboard.start_update_loop(window, pane)
  if not state.visible then
    return
  end
  
  -- Update dashboard content
  Dashboard.update_data()
  Dashboard.render(pane)
  
  -- Schedule next update
  wezterm.time.call_after(config.update_interval, function()
    Dashboard.start_update_loop(window, pane)
  end)
end

-- Update dashboard data
function Dashboard.update_data()
  -- This would normally fetch data from the backend
  -- For now, we'll use mock data
  
  -- Update timestamp
  state.last_update = os.time()
  
  -- Mock workspace data
  state.workspaces = {
    default = { is_active = true, process_count = 2 },
    frontend = { is_active = false, process_count = 1 },
    backend = { is_active = false, process_count = 3 }
  }
  
  -- Mock process data
  state.processes = {
    ["claude-1"] = { 
      id = "claude-1", 
      workspace = "default", 
      status = "Running",
      cpu_usage = 25,
      memory_usage = 128
    },
    ["claude-2"] = { 
      id = "claude-2", 
      workspace = "frontend", 
      status = "Idle",
      cpu_usage = 5,
      memory_usage = 64
    },
    ["claude-3"] = { 
      id = "claude-3", 
      workspace = "backend", 
      status = "Busy",
      cpu_usage = 60,
      memory_usage = 256
    }
  }
  
  -- Mock task data
  state.tasks = {
    ["task-1"] = {
      id = "task-1",
      command = "Build project",
      status = "Running",
      created_at = os.time() - 60
    },
    ["task-2"] = {
      id = "task-2",
      command = "Run tests",
      status = "Queued",
      created_at = os.time() - 30
    }
  }
  
  -- Mock system health
  state.system_health = {
    total_processes = 3,
    responsive_processes = 3,
    avg_cpu_usage = 30.0,
    total_memory_usage = 448,
    overall_status = "Healthy"
  }
  
  -- Clear old alerts
  state.alerts = {}
end

-- Handle dashboard events
function Dashboard.handle_event(event)
  if event.type == "workspace_created" then
    state.workspaces[event.workspace_name] = {
      is_active = false,
      process_count = 0
    }
  elseif event.type == "process_started" then
    state.processes[event.process_id] = event.process_info
  elseif event.type == "alert" then
    table.insert(state.alerts, event.alert)
  end
end

-- Get dashboard state
function Dashboard.get_state()
  return state
end

-- Check if dashboard is visible
function Dashboard.is_visible()
  return state.visible
end

return Dashboard