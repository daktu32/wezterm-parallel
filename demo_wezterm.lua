-- WezTerm Parallel Development Framework Demo
-- This demonstrates how to integrate the task management system with WezTerm

local wezterm = require('wezterm')

-- Task board UI component (simplified version)
local task_board = {
    -- Demo task data
    tasks = {
        {
            id = "0bf3a0c0-5d1f-450b-8f96-5226b7fb6a39",
            title = "Implement authentication system",
            status = "todo",
            priority = "urgent"
        },
        {
            id = "9c0d68de-1b69-47b3-86cf-22b0d059cf51", 
            title = "Fix memory leak in WebSocket handler",
            status = "in_progress",
            priority = "critical"
        }
    }
}

-- Function to render task board as text
function task_board.render()
    local output = {}
    table.insert(output, "=== WezTerm Parallel Task Board ===")
    table.insert(output, "")
    
    -- Column headers
    table.insert(output, string.format("%-25s %-25s %-25s %-25s", "To Do", "In Progress", "Review", "Done"))
    table.insert(output, string.rep("-", 100))
    
    -- Group tasks by status
    local columns = {
        todo = {},
        in_progress = {},
        review = {},
        done = {}
    }
    
    for _, task in ipairs(task_board.tasks) do
        table.insert(columns[task.status] or columns.todo, task)
    end
    
    -- Render rows
    local max_rows = 10
    for row = 1, max_rows do
        local row_content = {}
        
        for _, column_name in ipairs({"todo", "in_progress", "review", "done"}) do
            local task = columns[column_name][row]
            local cell_content = ""
            
            if task then
                local priority_icon = "📈"
                if task.priority == "urgent" then
                    priority_icon = "🔥"
                elseif task.priority == "critical" then
                    priority_icon = "⚠️"
                end
                
                cell_content = string.format("%s %s", priority_icon, task.title:sub(1, 20))
            end
            
            table.insert(row_content, string.format("%-25s", cell_content))
        end
        
        table.insert(output, table.concat(row_content, " "))
    end
    
    table.insert(output, "")
    table.insert(output, "Commands: [Ctrl+Shift+T] Toggle Board, [Ctrl+Shift+Alt+N] New Task")
    
    return table.concat(output, "\n")
end

-- WezTerm configuration
local config = {}

-- Key bindings for task management
config.keys = {
    -- Show task board overlay
    {
        key = 't',
        mods = 'CTRL|SHIFT',
        action = wezterm.action_callback(function(window, pane)
            local task_board_text = task_board.render()
            window:toast_notification("Task Board", task_board_text, nil, 10000)
        end),
    },
    
    -- Create new task (demo)
    {
        key = 'n',
        mods = 'CTRL|SHIFT|ALT',
        action = wezterm.action_callback(function(window, pane)
            window:perform_action(
                wezterm.action.PromptInputLine {
                    description = 'Enter task title:',
                    action = wezterm.action_callback(function(child_window, child_pane, line)
                        if line and line ~= "" then
                            -- In a real implementation, this would send to the IPC server
                            child_window:toast_notification(
                                "Task Created", 
                                "Task '" .. line .. "' would be created via IPC",
                                nil, 
                                3000
                            )
                        end
                    end),
                },
                pane
            )
        end),
    },
    
    -- Refresh task board
    {
        key = 'r',
        mods = 'CTRL|SHIFT|ALT',
        action = wezterm.action_callback(function(window, pane)
            -- In real implementation, this would fetch from WebSocket
            window:toast_notification("Task Board", "Refreshing task board...", nil, 2000)
        end),
    },
}

-- Color scheme optimized for task management
config.color_scheme = "Tomorrow Night"

-- Window configuration  
config.window_background_opacity = 0.95
config.window_decorations = "RESIZE"

-- Tab configuration
config.use_fancy_tab_bar = true
config.hide_tab_bar_if_only_one_tab = false

-- Font configuration
config.font_size = 12.0

-- Status line configuration
config.status_update_interval = 1000

-- Custom status line showing task info
wezterm.on('update-status', function(window, pane)
    local task_count = #task_board.tasks
    local status_text = string.format("📋 %d tasks | 🔥 WezTerm Parallel", task_count)
    
    window:set_right_status(wezterm.format {
        { Text = status_text },
    })
end)

-- Startup action
wezterm.on('gui-startup', function(cmd)
    local args = {}
    if cmd then
        args = cmd.args
    end
    
    local tab, pane, window = wezterm.mux.spawn_window(cmd or {})
    
    -- Show welcome message
    window:toast_notification(
        "WezTerm Parallel", 
        "Task management system loaded!\nPress Ctrl+Shift+T to view task board",
        nil, 
        5000
    )
end)

return config