-- WezTerm Multi-Process Development Framework - Task Board UI
-- Provides Kanban-style task board interface in WezTerm

local wezterm = require('wezterm')
local socket_client = require('utils.socket_client')
local json = require('utils.json')

local TaskBoard = {}

-- Task board configuration
local config = {
    -- WebSocket connection for real-time updates
    websocket_url = "ws://localhost:9999",
    
    -- Board appearance
    column_width = 25,
    max_visible_tasks = 10,
    
    -- Colors for different task priorities
    colors = {
        urgent = "#ff5722",
        critical = "#ff9800", 
        high = "#ffc107",
        medium = "#2196f3",
        low = "#4caf50"
    },
    
    -- Column colors
    column_colors = {
        todo = "#e3f2fd",
        in_progress = "#fff3e0", 
        review = "#fce4ec",
        done = "#e8f5e8"
    }
}

-- Current board state
local board_state = {
    board_id = "default",
    title = "Task Board",
    columns = {},
    tasks = {},
    last_updated = 0,
    websocket_connected = false
}

-- WebSocket connection for real-time updates
local websocket = nil

-- Initialize task board
function TaskBoard.init()
    -- Connect to WebSocket for real-time updates
    TaskBoard.connect_websocket()
    
    -- Request initial board state
    TaskBoard.request_board_update()
    
    wezterm.log_info("Task board initialized")
end

-- Connect to WebSocket server
function TaskBoard.connect_websocket()
    if board_state.websocket_connected then
        return
    end
    
    -- Try to connect via socket client
    local success = socket_client.connect()
    if success then
        board_state.websocket_connected = true
        wezterm.log_info("Connected to task board WebSocket")
        
        -- Subscribe to task board updates
        socket_client.send_message({
            type = "subscribe",
            data = {
                subscriptions = {"task_board", "task_updates"}
            }
        })
    else
        wezterm.log_error("Failed to connect to task board WebSocket")
    end
end

-- Request board state update
function TaskBoard.request_board_update()
    if not board_state.websocket_connected then
        TaskBoard.connect_websocket()
        return
    end
    
    socket_client.send_message({
        type = "get_board_state",
        data = {
            board_id = board_state.board_id
        }
    })
end

-- Handle WebSocket message
function TaskBoard.handle_websocket_message(message)
    if not message or not message.type then
        return
    end
    
    local msg_type = message.type
    local data = message.data or {}
    
    if msg_type == "TaskBoardUpdate" then
        TaskBoard.update_board_state(data)
    elseif msg_type == "TaskUpdate" then
        TaskBoard.handle_task_update(data)
    elseif msg_type == "TaskMoved" then
        TaskBoard.handle_task_moved(data)
    elseif msg_type == "TaskProgress" then
        TaskBoard.handle_task_progress(data)
    end
end

-- Update board state from WebSocket
function TaskBoard.update_board_state(data)
    if data.board_id == board_state.board_id then
        board_state.columns = data.columns or {}
        board_state.last_updated = data.timestamp or 0
        
        wezterm.log_info("Updated task board state")
        
        -- Refresh display if visible
        TaskBoard.refresh_display()
    end
end

-- Handle task update
function TaskBoard.handle_task_update(data)
    local task = data.task
    local action = data.action
    
    if not task or not task.id then
        return
    end
    
    -- Update local task cache
    if action == "Created" or action == "Updated" then
        board_state.tasks[task.id] = task
    elseif action == "Deleted" then
        board_state.tasks[task.id] = nil
    end
    
    -- Refresh display
    TaskBoard.refresh_display()
end

-- Handle task moved between columns
function TaskBoard.handle_task_moved(data)
    local task_id = data.task_id
    local from_column = data.from_column
    local to_column = data.to_column
    
    wezterm.log_info(string.format("Task %s moved from %s to %s", task_id, from_column, to_column))
    
    -- Request fresh board state
    TaskBoard.request_board_update()
end

-- Handle task progress update
function TaskBoard.handle_task_progress(data)
    local task_id = data.task_id
    local progress = data.progress
    
    -- Update local task cache if exists
    if board_state.tasks[task_id] then
        board_state.tasks[task_id].progress = progress
        TaskBoard.refresh_display()
    end
end

-- Create task board pane
function TaskBoard.create_pane(window, workspace_name)
    local tab = window:mux_window():active_tab()
    
    -- Create new pane for task board
    local pane = tab:split_pane({
        direction = "Right",
        size = { Percent = 50 },
        command = {
            args = { "lua", "-e", TaskBoard.get_display_script() }
        }
    })
    
    return pane
end

-- Get display script for task board
function TaskBoard.get_display_script()
    return [[
local function display_task_board()
    -- This would be a Lua script that renders the task board
    -- For now, we'll use a simple text display
    print("=== WezTerm Task Board ===")
    print("")
    
    -- Column headers
    print(string.format("%-25s %-25s %-25s %-25s", "To Do", "In Progress", "Review", "Done"))
    print(string.rep("-", 100))
    
    -- Task rows (placeholder)
    for i = 1, 10 do
        print(string.format("%-25s %-25s %-25s %-25s", "", "", "", ""))
    end
    
    print("")
    print("Commands: [a]dd task, [m]ove task, [p]rogress, [r]efresh, [q]uit")
end

display_task_board()
]]
end

-- Render task board as text
function TaskBoard.render_text()
    local output = {}
    
    -- Header
    table.insert(output, "=== " .. board_state.title .. " ===")
    table.insert(output, "")
    
    -- Column headers
    local headers = {}
    for _, column in ipairs(board_state.columns) do
        table.insert(headers, string.format("%-" .. config.column_width .. "s", column.title))
    end
    table.insert(output, table.concat(headers, " "))
    table.insert(output, string.rep("-", #headers * (config.column_width + 1)))
    
    -- Task rows
    local max_tasks = 0
    for _, column in ipairs(board_state.columns) do
        max_tasks = math.max(max_tasks, #column.tasks)
    end
    
    for row = 1, math.min(max_tasks, config.max_visible_tasks) do
        local row_data = {}
        
        for _, column in ipairs(board_state.columns) do
            local task_id = column.tasks[row]
            local task_text = ""
            
            if task_id then
                local task = board_state.tasks[task_id]
                if task then
                    -- Format task: priority + title + progress
                    local priority_icon = TaskBoard.get_priority_icon(task.priority)
                    local progress_text = task.progress and task.progress > 0 
                        and string.format("(%d%%)", task.progress) or ""
                    
                    task_text = string.format("%s %s %s", 
                        priority_icon, 
                        task.title:sub(1, config.column_width - 8), 
                        progress_text)
                end
            end
            
            table.insert(row_data, string.format("%-" .. config.column_width .. "s", task_text))
        end
        
        table.insert(output, table.concat(row_data, " "))
    end
    
    -- Footer
    table.insert(output, "")
    table.insert(output, string.format("Last updated: %s", os.date("%H:%M:%S", board_state.last_updated)))
    table.insert(output, "Commands: [a]dd [m]ove [p]rogress [r]efresh [q]uit")
    
    return table.concat(output, "\n")
end

-- Get priority icon
function TaskBoard.get_priority_icon(priority)
    local icons = {
        urgent = "🔥",
        critical = "⚠️",
        high = "📈",
        medium = "➖",
        low = "📉"
    }
    return icons[priority] or "➖"
end

-- Create new task via dialog
function TaskBoard.create_task_dialog(window)
    local workspace = window:active_workspace()
    
    -- Show input dialog
    window:perform_action(wezterm.action.PromptInputLine {
        description = "Enter task title:",
        action = wezterm.action_callback(function(child_window, child_pane, line)
            if line and line ~= "" then
                TaskBoard.create_task(line, workspace)
            end
        end)
    }, window:active_pane())
end

-- Create new task
function TaskBoard.create_task(title, workspace)
    if not board_state.websocket_connected then
        wezterm.log_error("Not connected to task board WebSocket")
        return
    end
    
    local task_data = {
        title = title,
        category = "Development",
        priority = "medium",
        workspace = workspace,
        status = "todo"
    }
    
    socket_client.send_message({
        type = "create_task",
        data = { task_data = task_data }
    })
    
    wezterm.log_info("Created task: " .. title)
end

-- Move task between columns
function TaskBoard.move_task(task_id, to_column)
    if not board_state.websocket_connected then
        wezterm.log_error("Not connected to task board WebSocket")
        return
    end
    
    socket_client.send_message({
        type = "move_task",
        data = {
            task_id = task_id,
            to_column = to_column
        }
    })
    
    wezterm.log_info(string.format("Moved task %s to %s", task_id, to_column))
end

-- Update task progress
function TaskBoard.update_task_progress(task_id, progress)
    if not board_state.websocket_connected then
        wezterm.log_error("Not connected to task board WebSocket")
        return
    end
    
    socket_client.send_message({
        type = "update_task_progress",
        data = {
            task_id = task_id,
            progress = progress
        }
    })
    
    wezterm.log_info(string.format("Updated task %s progress to %d%%", task_id, progress))
end

-- Refresh display
function TaskBoard.refresh_display()
    -- This would update any visible task board panes
    -- For now, we'll just log the update
    wezterm.log_info("Task board display refreshed")
end

-- Show task board in overlay
function TaskBoard.show_overlay(window)
    local text = TaskBoard.render_text()
    
    window:toast_notification("Task Board", text, nil, 10000)
end

-- Toggle task board overlay
function TaskBoard.toggle_overlay(window, pane)
    TaskBoard.show_overlay(window)
end

-- Key bindings for task board
function TaskBoard.get_key_bindings()
    return {
        -- Show task board overlay
        {
            key = "t",
            mods = "CTRL|SHIFT",
            action = wezterm.action_callback(function(window, pane)
                TaskBoard.toggle_overlay(window, pane)
            end)
        },
        
        -- Create new task
        {
            key = "n",
            mods = "CTRL|SHIFT|ALT",
            action = wezterm.action_callback(function(window, pane)
                TaskBoard.create_task_dialog(window)
            end)
        },
        
        -- Refresh task board
        {
            key = "r",
            mods = "CTRL|SHIFT|ALT",
            action = wezterm.action_callback(function(window, pane)
                TaskBoard.request_board_update()
                TaskBoard.show_overlay(window)
            end)
        }
    }
end

-- Export module
return TaskBoard