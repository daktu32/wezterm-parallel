-- WezTerm Multi-Process Development Framework - Enhanced Real-time Dashboard
-- Provides WebSocket client and real-time UI updates

local wezterm = require 'wezterm'
local json = require 'lua.utils.json'

local Dashboard = {}

-- Dashboard configuration
local config = {
    websocket_url = "ws://localhost:9999",
    update_interval = 1.0,
    reconnect_interval = 5.0,
    max_reconnect_attempts = 10,
    theme = {
        background = "#1e1e2e",
        foreground = "#cdd6f4",
        border = "#45475a",
        header = "#89b4fa",
        success = "#a6e3a1",
        warning = "#f9e2af",
        error = "#f38ba8",
        info = "#89dceb",
        metric_good = "#a6e3a1",
        metric_warn = "#f9e2af",
        metric_bad = "#f38ba8",
    }
}

-- Dashboard state
local state = {
    connected = false,
    websocket = nil,
    pane = nil,
    window = nil,
    metrics = {
        system = nil,
        workspaces = {},
        processes = {},
        alerts = {},
    },
    last_update = 0,
    reconnect_attempts = 0,
    update_timer = nil,
    visible = false,
}

-- Initialize enhanced dashboard
function Dashboard.init(framework_config)
    if framework_config.dashboard then
        -- Merge configuration
        for k, v in pairs(framework_config.dashboard) do
            config[k] = v
        end
        
        -- Update theme if provided
        if framework_config.dashboard.theme then
            for k, v in pairs(framework_config.dashboard.theme) do
                config.theme[k] = v
            end
        end
    end
    
    wezterm.log_info("Enhanced Dashboard initialized")
end

-- Connect to dashboard WebSocket server
function Dashboard.connect()
    if state.websocket then
        wezterm.log_info("WebSocket already connected")
        return
    end
    
    wezterm.log_info("Connecting to dashboard server: " .. config.websocket_url)
    
    -- Create WebSocket connection
    local success, ws, err = wezterm.websocket.connect(config.websocket_url, {
        on_message = Dashboard.handle_message,
        on_error = Dashboard.handle_error,
        on_close = Dashboard.handle_close,
    })
    
    if success then
        state.websocket = ws
        state.connected = true
        state.reconnect_attempts = 0
        
        -- Subscribe to all metrics
        Dashboard.send_command({
            command = "Subscribe",
            params = {
                subscriptions = {"All"}
            }
        })
        
        -- Request initial full update
        Dashboard.send_command({
            command = "RequestFullUpdate"
        })
        
        wezterm.log_info("Dashboard WebSocket connected")
    else
        wezterm.log_error("Failed to connect to dashboard: " .. (err or "unknown error"))
        Dashboard.schedule_reconnect()
    end
end

-- Handle incoming WebSocket messages
function Dashboard.handle_message(message)
    local ok, data = pcall(json.decode, message)
    if not ok then
        wezterm.log_error("Failed to parse dashboard message: " .. message)
        return
    end
    
    if data.payload then
        local msg_type = data.payload.type
        local msg_data = data.payload.data
        
        if msg_type == "MetricsUpdate" then
            Dashboard.handle_metrics_update(msg_data)
        elseif msg_type == "Alert" then
            Dashboard.handle_alert(msg_data)
        elseif msg_type == "StatusChange" then
            Dashboard.handle_status_change(msg_data)
        elseif msg_type == "Error" then
            Dashboard.handle_error_message(msg_data)
        end
    end
    
    -- Update UI if visible
    if state.visible and state.pane then
        Dashboard.render()
    end
end

-- Handle metrics update
function Dashboard.handle_metrics_update(update)
    state.last_update = update.timestamp or os.time()
    
    -- Update system metrics
    if update.system then
        state.metrics.system = update.system
    end
    
    -- Update workspace metrics
    if update.workspaces then
        for _, workspace in ipairs(update.workspaces) do
            state.metrics.workspaces[workspace.workspace_name] = workspace
        end
    end
    
    -- Update process metrics
    if update.processes then
        for _, process in ipairs(update.processes) do
            state.metrics.processes[process.process_id] = process
        end
    end
    
    -- Update framework metrics if full update
    if update.framework then
        state.metrics.framework = update.framework
    end
end

-- Handle alert notification
function Dashboard.handle_alert(alert)
    table.insert(state.metrics.alerts, 1, alert)
    
    -- Keep only recent alerts
    if #state.metrics.alerts > 50 then
        state.metrics.alerts = {unpack(state.metrics.alerts, 1, 50)}
    end
    
    -- Show toast notification for critical alerts
    if alert.severity == "Critical" and state.window then
        state.window:toast_notification(
            "Critical Alert",
            alert.message,
            nil,
            5000
        )
    end
end

-- Handle status change
function Dashboard.handle_status_change(change)
    wezterm.log_info(string.format(
        "Status change: %s %s -> %s",
        change.component,
        change.previous_status,
        change.new_status
    ))
end

-- Handle error message
function Dashboard.handle_error_message(error)
    wezterm.log_error("Dashboard error: " .. error.message)
end

-- Handle WebSocket error
function Dashboard.handle_error(error)
    wezterm.log_error("Dashboard WebSocket error: " .. error)
    state.connected = false
end

-- Handle WebSocket close
function Dashboard.handle_close()
    wezterm.log_info("Dashboard WebSocket closed")
    state.connected = false
    state.websocket = nil
    Dashboard.schedule_reconnect()
end

-- Schedule reconnection attempt
function Dashboard.schedule_reconnect()
    if state.reconnect_attempts >= config.max_reconnect_attempts then
        wezterm.log_error("Max reconnection attempts reached")
        return
    end
    
    state.reconnect_attempts = state.reconnect_attempts + 1
    
    wezterm.time.call_after(config.reconnect_interval, function()
        wezterm.log_info("Attempting to reconnect to dashboard...")
        Dashboard.connect()
    end)
end

-- Send command to dashboard server
function Dashboard.send_command(command)
    if not state.websocket or not state.connected then
        wezterm.log_warn("Cannot send command: not connected")
        return
    end
    
    local message = {
        id = tostring(os.time()),
        payload = {
            type = "Command",
            data = command
        }
    }
    
    local ok, encoded = pcall(json.encode, message)
    if ok then
        state.websocket:send(encoded)
    else
        wezterm.log_error("Failed to encode command")
    end
end

-- Toggle dashboard visibility
function Dashboard.toggle(window, pane)
    if state.visible then
        Dashboard.hide()
    else
        Dashboard.show(window, pane)
    end
end

-- Show dashboard
function Dashboard.show(window, pane)
    state.window = window
    state.visible = true
    
    -- Connect if not connected
    if not state.connected then
        Dashboard.connect()
    end
    
    -- Create dashboard pane
    local tab = window:active_tab()
    local dashboard_pane = tab:split_pane({
        direction = config.position == "right" and "Right" or "Left",
        size = config.width_percentage / 100,
    })
    
    state.pane = dashboard_pane
    
    -- Initial render
    Dashboard.render()
    
    -- Start update timer
    if not state.update_timer then
        state.update_timer = wezterm.time.call_after(
            config.update_interval,
            Dashboard.update_loop
        )
    end
    
    wezterm.log_info("Dashboard shown")
end

-- Hide dashboard
function Dashboard.hide()
    if state.pane then
        state.pane:close()
        state.pane = nil
    end
    
    state.visible = false
    
    -- Cancel update timer
    if state.update_timer then
        wezterm.time.cancel(state.update_timer)
        state.update_timer = nil
    end
    
    wezterm.log_info("Dashboard hidden")
end

-- Update loop
function Dashboard.update_loop()
    if state.visible then
        Dashboard.render()
        
        -- Schedule next update
        state.update_timer = wezterm.time.call_after(
            config.update_interval,
            Dashboard.update_loop
        )
    end
end

-- Render dashboard content
function Dashboard.render()
    if not state.pane then
        return
    end
    
    local output = {}
    
    -- Clear screen and move to top
    table.insert(output, "\x1b[2J\x1b[H")
    
    -- Header
    table.insert(output, Dashboard.render_header())
    
    -- Connection status
    table.insert(output, Dashboard.render_connection_status())
    
    -- System metrics
    if state.metrics.system then
        table.insert(output, Dashboard.render_system_metrics())
    end
    
    -- Workspace overview
    table.insert(output, Dashboard.render_workspace_overview())
    
    -- Process list
    table.insert(output, Dashboard.render_process_list())
    
    -- Alerts
    table.insert(output, Dashboard.render_alerts())
    
    -- Footer
    table.insert(output, Dashboard.render_footer())
    
    -- Send to pane
    state.pane:inject_output(table.concat(output))
end

-- Render header
function Dashboard.render_header()
    local header = {}
    
    -- Title with color
    table.insert(header, string.format(
        "\x1b[38;2;%s;%s;%sm%s\x1b[0m\n",
        Dashboard.hex_to_rgb(config.theme.header),
        "═══ WezTerm Multi-Process Dashboard ═══"
    ))
    
    -- Timestamp
    table.insert(header, string.format(
        "\x1b[38;2;%s;%s;%sm%s\x1b[0m\n\n",
        Dashboard.hex_to_rgb(config.theme.info),
        os.date("%Y-%m-%d %H:%M:%S")
    ))
    
    return table.concat(header)
end

-- Render connection status
function Dashboard.render_connection_status()
    local status = {}
    
    local color = state.connected and config.theme.success or config.theme.error
    local status_text = state.connected and "● Connected" or "○ Disconnected"
    
    table.insert(status, string.format(
        "\x1b[38;2;%s;%s;%sm%s\x1b[0m",
        Dashboard.hex_to_rgb(color),
        status_text
    ))
    
    if not state.connected and state.reconnect_attempts > 0 then
        table.insert(status, string.format(
            " (Reconnect attempt %d/%d)",
            state.reconnect_attempts,
            config.max_reconnect_attempts
        ))
    end
    
    table.insert(status, "\n\n")
    
    return table.concat(status)
end

-- Render system metrics
function Dashboard.render_system_metrics()
    local metrics = state.metrics.system
    local output = {}
    
    -- Section header
    table.insert(output, string.format(
        "\x1b[38;2;%s;%s;%sm▼ System Metrics\x1b[0m\n",
        Dashboard.hex_to_rgb(config.theme.header)
    ))
    
    -- CPU usage with color coding
    local cpu_color = Dashboard.get_metric_color(metrics.cpu_usage, 60, 80)
    table.insert(output, string.format(
        "  CPU:     \x1b[38;2;%s;%s;%sm%5.1f%%\x1b[0m %s\n",
        Dashboard.hex_to_rgb(cpu_color),
        metrics.cpu_usage,
        Dashboard.render_bar(metrics.cpu_usage, 100, 20)
    ))
    
    -- Memory usage
    local mem_color = Dashboard.get_metric_color(metrics.memory_percentage, 70, 85)
    table.insert(output, string.format(
        "  Memory:  \x1b[38;2;%s;%s;%sm%5.1f%%\x1b[0m %s (%.1f GB / %.1f GB)\n",
        Dashboard.hex_to_rgb(mem_color),
        metrics.memory_percentage,
        Dashboard.render_bar(metrics.memory_percentage, 100, 20),
        metrics.memory_usage / (1024 * 1024 * 1024),
        metrics.total_memory / (1024 * 1024 * 1024)
    ))
    
    -- Disk usage
    local disk_color = Dashboard.get_metric_color(metrics.disk_percentage, 80, 90)
    table.insert(output, string.format(
        "  Disk:    \x1b[38;2;%s;%s;%sm%5.1f%%\x1b[0m %s\n",
        Dashboard.hex_to_rgb(disk_color),
        metrics.disk_percentage,
        Dashboard.render_bar(metrics.disk_percentage, 100, 20)
    ))
    
    -- Load average
    table.insert(output, string.format(
        "  Load:    %.2f, %.2f, %.2f\n",
        metrics.load_average[1],
        metrics.load_average[2],
        metrics.load_average[3]
    ))
    
    -- Network I/O
    table.insert(output, string.format(
        "  Network: ↓ %.1f KB/s  ↑ %.1f KB/s\n",
        metrics.network_io.rx_rate / 1024,
        metrics.network_io.tx_rate / 1024
    ))
    
    table.insert(output, "\n")
    
    return table.concat(output)
end

-- Render workspace overview
function Dashboard.render_workspace_overview()
    local output = {}
    
    -- Section header
    table.insert(output, string.format(
        "\x1b[38;2;%s;%s;%sm▼ Workspaces\x1b[0m\n",
        Dashboard.hex_to_rgb(config.theme.header)
    ))
    
    -- Sort workspaces by name
    local workspace_names = {}
    for name, _ in pairs(state.metrics.workspaces) do
        table.insert(workspace_names, name)
    end
    table.sort(workspace_names)
    
    -- Display each workspace
    for _, name in ipairs(workspace_names) do
        local workspace = state.metrics.workspaces[name]
        local health_color = Dashboard.get_health_color(workspace.health_score)
        
        table.insert(output, string.format(
            "  %s \x1b[38;2;%s;%s;%sm[%.0f%%]\x1b[0m - %d/%d processes",
            Dashboard.pad_string(name, 15),
            Dashboard.hex_to_rgb(health_color),
            workspace.health_score,
            workspace.running_processes,
            workspace.total_processes
        ))
        
        if workspace.failed_processes > 0 then
            table.insert(output, string.format(
                " \x1b[38;2;%s;%s;%sm(%d failed)\x1b[0m",
                Dashboard.hex_to_rgb(config.theme.error),
                workspace.failed_processes
            ))
        end
        
        table.insert(output, "\n")
    end
    
    table.insert(output, "\n")
    
    return table.concat(output)
end

-- Render process list
function Dashboard.render_process_list()
    local output = {}
    
    -- Section header
    table.insert(output, string.format(
        "\x1b[38;2;%s;%s;%sm▼ Processes\x1b[0m\n",
        Dashboard.hex_to_rgb(config.theme.header)
    ))
    
    -- Get process list sorted by workspace and ID
    local processes = {}
    for _, process in pairs(state.metrics.processes) do
        table.insert(processes, process)
    end
    
    table.sort(processes, function(a, b)
        if a.workspace == b.workspace then
            return a.process_id < b.process_id
        end
        return a.workspace < b.workspace
    end)
    
    -- Display processes (limit to recent ones)
    local display_count = 0
    for _, process in ipairs(processes) do
        if display_count >= 10 then
            table.insert(output, string.format(
                "  ... and %d more processes\n",
                #processes - display_count
            ))
            break
        end
        
        local status_color = Dashboard.get_status_color(process.status)
        
        table.insert(output, string.format(
            "  %s \x1b[38;2;%s;%s;%sm%s\x1b[0m CPU:%5.1f%% Mem:%5.1f%%",
            Dashboard.pad_string(process.process_id, 20),
            Dashboard.hex_to_rgb(status_color),
            Dashboard.pad_string(process.status, 10),
            process.cpu_usage,
            process.memory_percentage
        ))
        
        -- Add response time if available
        if process.response_time then
            table.insert(output, string.format(" RT:%dms", process.response_time))
        end
        
        table.insert(output, "\n")
        display_count = display_count + 1
    end
    
    table.insert(output, "\n")
    
    return table.concat(output)
end

-- Render alerts
function Dashboard.render_alerts()
    local output = {}
    
    if #state.metrics.alerts == 0 then
        return ""
    end
    
    -- Section header
    table.insert(output, string.format(
        "\x1b[38;2;%s;%s;%sm▼ Recent Alerts\x1b[0m\n",
        Dashboard.hex_to_rgb(config.theme.header)
    ))
    
    -- Display recent alerts (limit to 5)
    local display_count = math.min(5, #state.metrics.alerts)
    for i = 1, display_count do
        local alert = state.metrics.alerts[i]
        local severity_color = Dashboard.get_severity_color(alert.severity)
        
        table.insert(output, string.format(
            "  \x1b[38;2;%s;%s;%sm[%s]\x1b[0m %s\n",
            Dashboard.hex_to_rgb(severity_color),
            alert.severity,
            alert.message
        ))
    end
    
    if #state.metrics.alerts > display_count then
        table.insert(output, string.format(
            "  ... and %d more alerts\n",
            #state.metrics.alerts - display_count
        ))
    end
    
    table.insert(output, "\n")
    
    return table.concat(output)
end

-- Render footer
function Dashboard.render_footer()
    local footer = {}
    
    table.insert(footer, string.format(
        "\x1b[38;2;%s;%s;%sm%s\x1b[0m\n",
        Dashboard.hex_to_rgb(config.theme.border),
        string.rep("─", 40)
    ))
    
    -- Keyboard shortcuts
    table.insert(footer, string.format(
        "\x1b[38;2;%s;%s;%smCtrl+Shift+D: Toggle | R: Refresh | Q: Quit\x1b[0m\n",
        Dashboard.hex_to_rgb(config.theme.info)
    ))
    
    return table.concat(footer)
end

-- Utility: Render progress bar
function Dashboard.render_bar(value, max, width)
    local filled = math.floor((value / max) * width)
    local empty = width - filled
    
    local bar = string.rep("█", filled) .. string.rep("░", empty)
    
    return "[" .. bar .. "]"
end

-- Utility: Get color based on metric value
function Dashboard.get_metric_color(value, warn_threshold, error_threshold)
    if value >= error_threshold then
        return config.theme.metric_bad
    elseif value >= warn_threshold then
        return config.theme.metric_warn
    else
        return config.theme.metric_good
    end
end

-- Utility: Get color based on health score
function Dashboard.get_health_color(score)
    if score >= 90 then
        return config.theme.success
    elseif score >= 70 then
        return config.theme.warning
    else
        return config.theme.error
    end
end

-- Utility: Get color based on process status
function Dashboard.get_status_color(status)
    local status_colors = {
        Running = config.theme.success,
        Idle = config.theme.info,
        Busy = config.theme.warning,
        Unresponsive = config.theme.error,
        Failed = config.theme.error,
        Starting = config.theme.info,
        Stopping = config.theme.warning,
        Stopped = config.theme.foreground,
    }
    
    return status_colors[status] or config.theme.foreground
end

-- Utility: Get color based on alert severity
function Dashboard.get_severity_color(severity)
    local severity_colors = {
        Info = config.theme.info,
        Warning = config.theme.warning,
        Critical = config.theme.error,
        Resolved = config.theme.success,
    }
    
    return severity_colors[severity] or config.theme.foreground
end

-- Utility: Convert hex color to RGB values
function Dashboard.hex_to_rgb(hex)
    hex = hex:gsub("#", "")
    local r = tonumber(hex:sub(1, 2), 16)
    local g = tonumber(hex:sub(3, 4), 16)
    local b = tonumber(hex:sub(5, 6), 16)
    return r, g, b
end

-- Utility: Pad string to fixed width
function Dashboard.pad_string(str, width)
    if #str >= width then
        return str:sub(1, width)
    else
        return str .. string.rep(" ", width - #str)
    end
end

-- Check if dashboard is visible
function Dashboard.is_visible()
    return state.visible
end

-- Handle dashboard events
function Dashboard.handle_event(event)
    -- Process event and update state
    if event.type == "process_started" then
        -- Request process update
        Dashboard.send_command({
            command = "RequestFullUpdate"
        })
    end
end

-- Get dashboard state
function Dashboard.get_state()
    return {
        connected = state.connected,
        visible = state.visible,
        last_update = state.last_update,
        metrics_count = {
            workspaces = 0, -- Can't easily count table keys
            processes = 0,
            alerts = #state.metrics.alerts,
        }
    }
end

-- Update dashboard data manually
function Dashboard.update_data()
    if state.connected then
        Dashboard.send_command({
            command = "RequestFullUpdate"
        })
    end
end

return Dashboard