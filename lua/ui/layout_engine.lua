-- WezTerm Multi-Process Development Framework - Layout Engine
-- 動的ペイン分割・配置システム

local wezterm = require 'wezterm'

local LayoutEngine = {}

-- レイアウトエンジン設定
local config = {
  minimum_pane_size = 0.1,  -- 最小ペインサイズ（10%）
  balance_threshold = 0.3,  -- バランス調整の閾値
  grid_snap_enabled = true, -- グリッドスナップ有効
  animation_enabled = true, -- アニメーション有効
  default_split_ratio = 0.5, -- デフォルト分割比率
  
  -- エラーハンドリング設定
  error_recovery_enabled = true, -- エラー復旧機能有効
  fallback_layout_enabled = true, -- フォールバックレイアウト有効
  user_friendly_errors = true, -- ユーザー向けエラーメッセージ
  max_panes = 20, -- 最大ペイン数
  timeout_ms = 5000, -- タイムアウト（ミリ秒）
  
  -- 復旧オプション
  auto_fix_conflicts = true, -- 競合の自動修正
  auto_normalize_sizes = true, -- サイズの自動正規化
  invalid_pane_handling = "skip" -- skip, fix, error
}

-- エラー情報とユーザーメッセージ
local error_messages = {
  invalid_template = "テンプレートが無効です。構造を確認してください。",
  pane_limit_exceeded = "ペイン数が上限（" .. config.max_panes .. "）を超えています。",
  invalid_pane_size = "ペインサイズが無効です。0.1から1.0の間で設定してください。",
  position_conflict = "ペインの位置が重複しています。自動修正を試行します。",
  missing_required_field = "必須フィールドが不足しています。",
  calculation_error = "レイアウト計算でエラーが発生しました。",
  viewport_error = "ビューポートサイズが無効です。"
}

-- エラー統計
local error_stats = {
  total_errors = 0,
  recovered_errors = 0,
  unrecoverable_errors = 0
}

-- 初期化
function LayoutEngine.init(framework_config)
  if framework_config and framework_config.layout_engine then
    for k, v in pairs(framework_config.layout_engine) do
      config[k] = v
    end
  end
  
  wezterm.log_info("LayoutEngine initialized")
end

-- 基本レイアウトの計算（エラーハンドリング強化）
function LayoutEngine.calculate_layout(panes, viewport_size)
  -- 入力検証
  local validation_result = LayoutEngine.validate_inputs(panes, viewport_size)
  if not validation_result.valid then
    if config.error_recovery_enabled then
      return LayoutEngine.attempt_error_recovery(panes, viewport_size, validation_result)
    else
      wezterm.log_error("Layout calculation failed: " .. validation_result.error)
      return {}
    end
  end
  
  if not panes or #panes == 0 then
    return {}
  end
  
  local layout = {}
  local total_size = 0
  
  -- サイズの正規化
  for _, pane in ipairs(panes) do
    total_size = total_size + (pane.size or 1)
  end
  
  local normalized_panes = {}
  for _, pane in ipairs(panes) do
    local normalized_pane = {}
    for k, v in pairs(pane) do
      normalized_pane[k] = v
    end
    normalized_pane.size = (pane.size or 1) / total_size
    table.insert(normalized_panes, normalized_pane)
  end
  
  -- 各ペインの実際のサイズを計算
  for i, pane in ipairs(normalized_panes) do
    local layout_pane = {
      id = pane.id,
      size = pane.size,
      position = pane.position or {row = 0, col = i - 1, span_rows = 1, span_cols = 1},
      calculated_width = math.floor(viewport_size.width * pane.size),
      calculated_height = math.floor(viewport_size.height * pane.size),
      command = pane.command,
      working_directory = pane.working_directory
    }
    
    table.insert(layout, layout_pane)
  end
  
  return layout
end

-- 分割方向の決定
function LayoutEngine.determine_split_direction(position)
  if not position then
    return "Right"
  end
  
  -- 列が異なる場合は水平分割（右に分割）
  if position.col and position.col > 0 then
    return "Right"
  end
  
  -- 行が異なる場合は垂直分割（下に分割）
  if position.row and position.row > 0 then
    return "Bottom"
  end
  
  -- デフォルトは右分割
  return "Right"
end

-- ペインサイズの正規化
function LayoutEngine.normalize_pane_sizes(panes)
  if not panes or #panes == 0 then
    return {}
  end
  
  local total_size = 0
  local normalized = {}
  
  -- 合計サイズを計算
  for _, pane in ipairs(panes) do
    total_size = total_size + (pane.size or 1)
  end
  
  -- 正規化
  for _, pane in ipairs(panes) do
    local normalized_pane = {}
    for k, v in pairs(pane) do
      normalized_pane[k] = v
    end
    normalized_pane.size = (pane.size or 1) / total_size
    table.insert(normalized, normalized_pane)
  end
  
  return normalized
end

-- グリッドレイアウトの計算
function LayoutEngine.calculate_grid_layout(grid_config, viewport_size)
  if not grid_config or not grid_config.panes then
    return {}
  end
  
  local rows = grid_config.rows or math.ceil(math.sqrt(#grid_config.panes))
  local cols = grid_config.cols or math.ceil(#grid_config.panes / rows)
  
  local layout = {}
  local pane_width = viewport_size.width / cols
  local pane_height = viewport_size.height / rows
  
  for i, pane in ipairs(grid_config.panes) do
    local row = math.floor((i - 1) / cols)
    local col = (i - 1) % cols
    
    local layout_pane = {
      id = pane.id,
      size = 1 / #grid_config.panes,
      position = {
        row = row,
        col = col,
        span_rows = 1,
        span_cols = 1
      },
      calculated_width = math.floor(pane_width),
      calculated_height = math.floor(pane_height),
      command = pane.command,
      working_directory = pane.working_directory
    }
    
    table.insert(layout, layout_pane)
  end
  
  return layout
end

-- レイアウトのサイズ調整
function LayoutEngine.adjust_layout_for_size(layout, new_size)
  if not layout or not new_size then
    return layout
  end
  
  local adjusted = {}
  
  for _, pane in ipairs(layout) do
    local adjusted_pane = {}
    for k, v in pairs(pane) do
      adjusted_pane[k] = v
    end
    
    -- 新しいサイズでの計算
    adjusted_pane.calculated_width = math.floor(new_size.width * pane.size)
    adjusted_pane.calculated_height = math.floor(new_size.height * pane.size)
    
    table.insert(adjusted, adjusted_pane)
  end
  
  return adjusted
end

-- レイアウトの競合検出
function LayoutEngine.detect_conflicts(layout)
  if not layout or #layout <= 1 then
    return {}
  end
  
  local conflicts = {}
  local occupied_positions = {}
  
  for i, pane in ipairs(layout) do
    if pane.position then
      -- スパン領域を考慮した競合検出
      local span_rows = pane.position.span_rows or 1
      local span_cols = pane.position.span_cols or 1
      local start_row = pane.position.row or 0
      local start_col = pane.position.col or 0
      
      for r = start_row, start_row + span_rows - 1 do
        for c = start_col, start_col + span_cols - 1 do
          local pos_key = string.format("%d_%d", r, c)
          
          if occupied_positions[pos_key] then
            table.insert(conflicts, {
              pane1 = occupied_positions[pos_key],
              pane2 = pane,
              position = {row = r, col = c},
              conflict_type = "position_overlap"
            })
          else
            occupied_positions[pos_key] = pane
          end
        end
      end
    end
  end
  
  return conflicts
end

-- 最小ペインサイズの強制
function LayoutEngine.enforce_minimum_size(layout, min_size)
  if not layout or not min_size then
    return layout
  end
  
  local enforced = {}
  local adjustment_needed = false
  local total_adjustments = 0
  
  for _, pane in ipairs(layout) do
    local enforced_pane = {}
    for k, v in pairs(pane) do
      enforced_pane[k] = v
    end
    
    if pane.size and pane.size < min_size then
      local adjustment = min_size - pane.size
      enforced_pane.size = min_size
      total_adjustments = total_adjustments + adjustment
      adjustment_needed = true
    end
    
    table.insert(enforced, enforced_pane)
  end
  
  -- 調整が必要な場合、他のペインから調整分を減らす
  if adjustment_needed and total_adjustments > 0 then
    local adjustable_panes = {}
    for _, pane in ipairs(enforced) do
      if pane.size and pane.size > min_size then
        table.insert(adjustable_panes, pane)
      end
    end
    
    if #adjustable_panes > 0 then
      local adjustment_per_pane = total_adjustments / #adjustable_panes
      for _, pane in ipairs(adjustable_panes) do
        pane.size = math.max(min_size, pane.size - adjustment_per_pane)
      end
    end
    
    -- 正規化して合計が1.0になるように調整
    enforced = LayoutEngine.normalize_pane_sizes(enforced)
  end
  
  return enforced
end

-- レイアウトのバランス調整
function LayoutEngine.balance_layout(layout)
  if not layout or #layout <= 1 then
    return layout
  end
  
  local balanced = {}
  local equal_size = 1.0 / #layout
  
  for _, pane in ipairs(layout) do
    local balanced_pane = {}
    for k, v in pairs(pane) do
      balanced_pane[k] = v
    end
    balanced_pane.size = equal_size
    table.insert(balanced, balanced_pane)
  end
  
  return balanced
end

-- テンプレートからレイアウト生成
function LayoutEngine.generate_from_template(template)
  if not template or not template.layout or not template.layout.panes then
    return {}
  end
  
  local layout = {}
  
  for _, pane in ipairs(template.layout.panes) do
    local layout_pane = {
      id = pane.id,
      size = pane.size or config.default_split_ratio,
      position = pane.position,
      command = pane.command,
      working_directory = pane.working_directory,
      title = pane.title
    }
    
    table.insert(layout, layout_pane)
  end
  
  return layout
end

-- レイアウトの妥当性検証
function LayoutEngine.validate_layout(layout)
  if not layout or type(layout) ~= "table" then
    return false
  end
  
  local total_size = 0
  local ids = {}
  
  for _, pane in ipairs(layout) do
    -- IDの重複チェック
    if pane.id and ids[pane.id] then
      wezterm.log_error("Duplicate pane ID: " .. pane.id)
      return false
    end
    if pane.id then
      ids[pane.id] = true
    end
    
    -- サイズの妥当性チェック
    if pane.size then
      if type(pane.size) ~= "number" or pane.size <= 0 or pane.size > 1 then
        wezterm.log_error("Invalid pane size: " .. tostring(pane.size))
        return false
      end
      total_size = total_size + pane.size
    end
    
    -- ポジションの妥当性チェック
    if pane.position then
      if type(pane.position) ~= "table" then
        wezterm.log_error("Invalid pane position format")
        return false
      end
      
      if pane.position.row and (type(pane.position.row) ~= "number" or pane.position.row < 0) then
        wezterm.log_error("Invalid pane row: " .. tostring(pane.position.row))
        return false
      end
      
      if pane.position.col and (type(pane.position.col) ~= "number" or pane.position.col < 0) then
        wezterm.log_error("Invalid pane col: " .. tostring(pane.position.col))
        return false
      end
    end
  end
  
  -- 合計サイズの確認（許容範囲内）
  if total_size > 0 and math.abs(total_size - 1.0) > 0.1 then
    wezterm.log_warn("Total pane size deviation: " .. tostring(total_size))
  end
  
  return true
end

-- 動的ペイン追加
function LayoutEngine.add_pane_to_layout(layout, new_pane, position_hint)
  if not layout or not new_pane then
    return layout
  end
  
  local updated_layout = {}
  
  -- 既存のペインをコピー
  for _, pane in ipairs(layout) do
    local copied_pane = {}
    for k, v in pairs(pane) do
      copied_pane[k] = v
    end
    table.insert(updated_layout, copied_pane)
  end
  
  -- 新しいペインを追加
  local layout_pane = {
    id = new_pane.id or ("pane_" .. (#layout + 1)),
    size = new_pane.size or (1.0 / (#layout + 1)),
    position = new_pane.position or position_hint,
    command = new_pane.command,
    working_directory = new_pane.working_directory,
    title = new_pane.title
  }
  
  table.insert(updated_layout, layout_pane)
  
  -- サイズの再正規化
  updated_layout = LayoutEngine.normalize_pane_sizes(updated_layout)
  
  return updated_layout
end

-- ペインの削除
function LayoutEngine.remove_pane_from_layout(layout, pane_id)
  if not layout or not pane_id then
    return layout
  end
  
  local updated_layout = {}
  
  for _, pane in ipairs(layout) do
    if pane.id ~= pane_id then
      local copied_pane = {}
      for k, v in pairs(pane) do
        copied_pane[k] = v
      end
      table.insert(updated_layout, copied_pane)
    end
  end
  
  -- サイズの再正規化
  if #updated_layout > 0 then
    updated_layout = LayoutEngine.normalize_pane_sizes(updated_layout)
  end
  
  return updated_layout
end

-- レイアウトの最適化
function LayoutEngine.optimize_layout(layout, viewport_size, optimization_options)
  if not layout then
    return layout
  end
  
  local optimized = {}
  
  -- 既存のレイアウトをコピー
  for _, pane in ipairs(layout) do
    local optimized_pane = {}
    for k, v in pairs(pane) do
      optimized_pane[k] = v
    end
    table.insert(optimized, optimized_pane)
  end
  
  -- 最小サイズの強制
  if config.minimum_pane_size > 0 then
    optimized = LayoutEngine.enforce_minimum_size(optimized, config.minimum_pane_size)
  end
  
  -- 競合の解決
  local conflicts = LayoutEngine.detect_conflicts(optimized)
  if #conflicts > 0 then
    wezterm.log_warn("Layout conflicts detected, attempting to resolve...")
    -- 簡単な競合解決: 重複ペインを再配置
    for i, conflict in ipairs(conflicts) do
      if conflict.pane2 and conflict.pane2.position then
        conflict.pane2.position.col = (conflict.pane2.position.col or 0) + 1
      end
    end
  end
  
  -- ビューポートサイズに合わせて調整
  if viewport_size then
    optimized = LayoutEngine.adjust_layout_for_size(optimized, viewport_size)
  end
  
  return optimized
end

-- レイアウト情報の取得
function LayoutEngine.get_layout_info(layout)
  if not layout then
    return {}
  end
  
  local info = {
    pane_count = #layout,
    total_size = 0,
    min_size = 1.0,
    max_size = 0.0,
    has_conflicts = false,
    panes = {}
  }
  
  for _, pane in ipairs(layout) do
    info.total_size = info.total_size + (pane.size or 0)
    
    if pane.size then
      info.min_size = math.min(info.min_size, pane.size)
      info.max_size = math.max(info.max_size, pane.size)
    end
    
    table.insert(info.panes, {
      id = pane.id,
      size = pane.size,
      position = pane.position
    })
  end
  
  -- 競合チェック
  local conflicts = LayoutEngine.detect_conflicts(layout)
  info.has_conflicts = #conflicts > 0
  
  return info
end

-- 設定の取得
function LayoutEngine.get_config()
  return config
end

-- 設定の更新
function LayoutEngine.update_config(new_config)
  if new_config then
    for k, v in pairs(new_config) do
      config[k] = v
    end
  end
end

-- エラーハンドリング機能の追加

-- 入力値の検証
function LayoutEngine.validate_inputs(panes, viewport_size)
  local result = {valid = true, error = "", warnings = {}}
  
  -- ペイン数の確認
  if panes and #panes > config.max_panes then
    result.valid = false
    result.error = error_messages.pane_limit_exceeded
    error_stats.total_errors = error_stats.total_errors + 1
    return result
  end
  
  -- ビューポートサイズの確認
  if viewport_size then
    if not viewport_size.width or not viewport_size.height or 
       viewport_size.width <= 0 or viewport_size.height <= 0 then
      result.valid = false
      result.error = error_messages.viewport_error
      error_stats.total_errors = error_stats.total_errors + 1
      return result
    end
  end
  
  -- 各ペインの基本検証
  if panes then
    for i, pane in ipairs(panes) do
      if pane.size and (pane.size <= 0 or pane.size > 1) then
        if config.invalid_pane_handling == "error" then
          result.valid = false
          result.error = error_messages.invalid_pane_size
          error_stats.total_errors = error_stats.total_errors + 1
          return result
        else
          table.insert(result.warnings, "Pane " .. (pane.id or i) .. " has invalid size, will be auto-corrected")
        end
      end
    end
  end
  
  return result
end

-- エラー復旧の試行
function LayoutEngine.attempt_error_recovery(panes, viewport_size, validation_result)
  wezterm.log_warn("Attempting error recovery: " .. validation_result.error)
  
  -- フォールバックレイアウトの使用
  if config.fallback_layout_enabled then
    local fallback_layout = LayoutEngine.create_fallback_layout(panes, viewport_size)
    if fallback_layout and #fallback_layout > 0 then
      error_stats.recovered_errors = error_stats.recovered_errors + 1
      wezterm.log_info("Error recovery successful using fallback layout")
      return fallback_layout
    end
  end
  
  error_stats.unrecoverable_errors = error_stats.unrecoverable_errors + 1
  wezterm.log_error("Error recovery failed")
  return {}
end

-- フォールバックレイアウトの作成
function LayoutEngine.create_fallback_layout(panes, viewport_size)
  if not panes or #panes == 0 then
    return {}
  end
  
  -- 簡単な等分割レイアウトを作成
  local fallback = {}
  local equal_size = 1.0 / math.min(#panes, config.max_panes)
  
  for i, pane in ipairs(panes) do
    if i > config.max_panes then
      break
    end
    
    local fallback_pane = {
      id = pane.id or ("fallback_pane_" .. i),
      size = equal_size,
      position = {
        row = 0,
        col = i - 1,
        span_rows = 1,
        span_cols = 1
      },
      command = pane.command or "echo 'Fallback pane'",
      working_directory = pane.working_directory or ".",
      is_fallback = true
    }
    
    if viewport_size then
      fallback_pane.calculated_width = math.floor(viewport_size.width * equal_size)
      fallback_pane.calculated_height = math.floor(viewport_size.height)
    end
    
    table.insert(fallback, fallback_pane)
  end
  
  return fallback
end

-- テンプレートからレイアウト生成（改良版）
function LayoutEngine.generate_from_template_safe(template)
  if not template then
    wezterm.log_error("Template is nil")
    return nil
  end
  
  -- テンプレートの基本検証
  local validation = LayoutEngine.validate_template_structure(template)
  if not validation.valid then
    if config.user_friendly_errors then
      wezterm.log_error(error_messages.invalid_template .. " " .. validation.error)
    else
      wezterm.log_error("Template validation failed: " .. validation.error)
    end
    
    if config.fallback_layout_enabled then
      return LayoutEngine.create_fallback_from_template(template)
    end
    return nil
  end
  
  -- 元の生成関数を呼び出し
  local success, layout = pcall(LayoutEngine.generate_from_template, template)
  if not success then
    wezterm.log_error("Template generation failed: " .. tostring(layout))
    if config.fallback_layout_enabled then
      return LayoutEngine.create_fallback_from_template(template)
    end
    return nil
  end
  
  return layout
end

-- テンプレート構造の検証
function LayoutEngine.validate_template_structure(template)
  local result = {valid = true, error = ""}
  
  if not template.layout then
    result.valid = false
    result.error = "Missing layout section"
    return result
  end
  
  if not template.layout.panes or type(template.layout.panes) ~= "table" then
    result.valid = false
    result.error = "Missing or invalid panes array"
    return result
  end
  
  if #template.layout.panes == 0 then
    result.valid = false
    result.error = "No panes defined in template"
    return result
  end
  
  if #template.layout.panes > config.max_panes then
    result.valid = false
    result.error = "Too many panes defined (max: " .. config.max_panes .. ")"
    return result
  end
  
  return result
end

-- テンプレートからフォールバック作成
function LayoutEngine.create_fallback_from_template(template)
  if not template or not template.layout or not template.layout.panes then
    return {}
  end
  
  local panes = template.layout.panes
  return LayoutEngine.create_fallback_layout(panes, {width = 1920, height = 1080})
end

-- 競合の自動解決
function LayoutEngine.auto_resolve_conflicts(layout)
  if not config.auto_fix_conflicts then
    return layout
  end
  
  local conflicts = LayoutEngine.detect_conflicts(layout)
  if #conflicts == 0 then
    return layout
  end
  
  wezterm.log_warn("Auto-resolving " .. #conflicts .. " layout conflicts")
  
  local resolved_layout = {}
  for _, pane in ipairs(layout) do
    local resolved_pane = {}
    for k, v in pairs(pane) do
      resolved_pane[k] = v
    end
    table.insert(resolved_layout, resolved_pane)
  end
  
  -- 簡単な競合解決: 重複ペインを右にシフト
  local position_map = {}
  for _, pane in ipairs(resolved_layout) do
    if pane.position then
      local key = pane.position.row .. "_" .. pane.position.col
      if position_map[key] then
        -- 競合発見、位置をシフト
        local new_col = pane.position.col
        repeat
          new_col = new_col + 1
          key = pane.position.row .. "_" .. new_col
        until not position_map[key]
        pane.position.col = new_col
      end
      position_map[key] = pane
    end
  end
  
  return resolved_layout
end

-- 不正なテンプレートの検知と修正
function LayoutEngine.sanitize_template(template)
  if not template then
    return nil
  end
  
  local sanitized = {}
  for k, v in pairs(template) do
    sanitized[k] = v
  end
  
  -- 基本フィールドの確保
  if not sanitized.layout then
    sanitized.layout = {}
  end
  
  if not sanitized.layout.panes then
    sanitized.layout.panes = {}
  end
  
  -- ペインの検証と修正
  local valid_panes = {}
  for i, pane in ipairs(sanitized.layout.panes) do
    if type(pane) == "table" then
      local valid_pane = {
        id = pane.id or ("pane_" .. i),
        size = pane.size or config.default_split_ratio,
        position = pane.position or {row = 0, col = i-1, span_rows = 1, span_cols = 1},
        command = pane.command or "zsh",
        working_directory = pane.working_directory or "."
      }
      
      -- サイズの正規化
      if valid_pane.size <= 0 or valid_pane.size > 1 then
        valid_pane.size = config.default_split_ratio
      end
      
      table.insert(valid_panes, valid_pane)
    end
  end
  
  sanitized.layout.panes = valid_panes
  return sanitized
end

-- エラー統計の取得
function LayoutEngine.get_error_stats()
  return error_stats
end

-- エラー統計のリセット
function LayoutEngine.reset_error_stats()
  error_stats = {
    total_errors = 0,
    recovered_errors = 0,
    unrecoverable_errors = 0
  }
end

-- ユーザー向けエラーメッセージの取得
function LayoutEngine.get_user_friendly_error(error_type)
  return error_messages[error_type] or "予期しないエラーが発生しました。"
end

return LayoutEngine