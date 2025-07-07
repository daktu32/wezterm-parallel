# 🎨 カスタマイズガイド

**WezTerm Parallelを自分好みにカスタマイズする方法**

このガイドでは、テーマ変更・プラグイン開発・拡張機能の実装方法を詳しく説明します。

## 🎭 テーマ・外観のカスタマイズ

### 1. ダッシュボードテーマ

#### 内蔵テーマの変更

```yaml
# ~/.config/wezterm-parallel/config.yaml
dashboard:
  theme: "catppuccin"  # catppuccin, nord, dark, light, custom

themes:
  current: "catppuccin"
  
  # Catppuccin テーマ
  catppuccin:
    colors:
      primary: "#89b4fa"      # ブルー
      secondary: "#a6e3a1"    # グリーン
      warning: "#f9e2af"      # イエロー
      error: "#f38ba8"        # レッド
      success: "#94e2d5"      # シアン
      background: "#1e1e2e"   # ダークグレー
      surface: "#313244"      # ライトグレー
      text: "#cdd6f4"         # ホワイト
      
  # Nord テーマ  
  nord:
    colors:
      primary: "#5e81ac"
      secondary: "#a3be8c"
      warning: "#ebcb8b"
      error: "#bf616a"
      success: "#8fbcbb"
      background: "#2e3440"
      surface: "#3b4252"
      text: "#d8dee9"
```

#### カスタムテーマの作成

```yaml
# ~/.config/wezterm-parallel/themes/my-theme.yaml
name: "My Custom Theme"
version: "1.0.0"
author: "Your Name"

colors:
  # メインカラー
  primary: "#007acc"        # VS Code Blue
  secondary: "#00d4aa"      # ターコイズ
  accent: "#ff6b35"         # オレンジ
  
  # ステータスカラー
  success: "#28a745"        # グリーン
  warning: "#ffc107"        # イエロー
  error: "#dc3545"          # レッド
  info: "#17a2b8"           # シアン
  
  # 背景・テキスト
  background: "#1a1a1a"     # ダークグレー
  surface: "#2d2d2d"        # 薄いグレー
  text_primary: "#ffffff"   # ホワイト
  text_secondary: "#cccccc" # 薄いグレー
  text_muted: "#999999"     # グレー
  
  # ボーダー・分割線
  border: "#404040"
  divider: "#333333"

fonts:
  ui: "SF Pro Display"      # UI用フォント
  mono: "SF Mono"           # モノスペースフォント
  size: 14                  # ベースサイズ
  
ui:
  # コンポーネント設定
  border_radius: 8          # 角の丸み
  shadow_blur: 4            # シャドウぼかし
  animation_duration: 200   # アニメーション時間（ms）
  
  # パネル設定
  panel_spacing: 16         # パネル間隔
  panel_padding: 12         # パネル内余白
  header_height: 48         # ヘッダー高さ
  
  # グラフ設定
  chart_grid_opacity: 0.1   # グリッド透明度
  chart_line_width: 2       # 線の太さ
```

#### ダッシュボードレイアウトのカスタマイズ

```yaml
# ~/.config/wezterm-parallel/config.yaml
dashboard:
  layout: "custom"
  
  # カスタムレイアウト定義
  panels:
    - type: "system_metrics"
      position: { x: 0, y: 0, width: 50, height: 30 }
      config:
        show_cpu: true
        show_memory: true
        show_disk: false
        chart_type: "line"
        
    - type: "process_list"
      position: { x: 50, y: 0, width: 50, height: 50 }
      config:
        max_items: 10
        show_icons: true
        sortBy: "memory"
        
    - type: "log_viewer"
      position: { x: 0, y: 30, width: 100, height: 40 }
      config:
        max_lines: 50
        auto_scroll: true
        filter_level: "info"
        
    - type: "custom_widget"
      position: { x: 50, y: 50, width: 50, height: 30 }
      source: "./widgets/productivity-tracker.html"
```

### 2. WezTerm統合テーマ

```lua
-- ~/.config/wezterm/themes/wezterm-parallel.lua

local wezterm = require 'wezterm'

local theme = {}

-- カラーパレット
theme.colors = {
  foreground = "#cdd6f4",
  background = "#1e1e2e",
  
  -- カーソル
  cursor_bg = "#f5e0dc",
  cursor_fg = "#1e1e2e",
  cursor_border = "#f5e0dc",
  
  -- 選択
  selection_fg = "#1e1e2e",
  selection_bg = "#f5e0dc",
  
  -- スクロールバー
  scrollbar_thumb = "#585b70",
  
  -- 分割線
  split = "#6c7086",
  
  -- ANSI colors
  ansi = {
    "#45475a", -- black
    "#f38ba8", -- red
    "#a6e3a1", -- green
    "#f9e2af", -- yellow
    "#89b4fa", -- blue
    "#f5c2e7", -- magenta
    "#94e2d5", -- cyan
    "#bac2de", -- white
  },
  
  brights = {
    "#585b70", -- bright black
    "#f38ba8", -- bright red
    "#a6e3a1", -- bright green
    "#f9e2af", -- bright yellow
    "#89b4fa", -- bright blue
    "#f5c2e7", -- bright magenta
    "#94e2d5", -- bright cyan
    "#a6adc8", -- bright white
  },
}

-- WezTerm Parallel統合カラー
theme.wtp_colors = {
  status_bar_bg = "#313244",
  status_bar_fg = "#cdd6f4",
  active_workspace = "#89b4fa",
  inactive_workspace = "#6c7086",
  process_running = "#a6e3a1",
  process_stopped = "#f38ba8",
  process_starting = "#f9e2af",
}

-- タブバー設定
theme.tab_bar = {
  background = "#1e1e2e",
  active_tab = {
    bg_color = "#89b4fa",
    fg_color = "#1e1e2e",
    intensity = "Bold",
  },
  inactive_tab = {
    bg_color = "#313244", 
    fg_color = "#cdd6f4",
  },
  inactive_tab_hover = {
    bg_color = "#585b70",
    fg_color = "#cdd6f4",
  },
}

return theme
```

## 🔌 プラグインシステム

### 1. プラグイン開発基盤

```yaml
# ~/.config/wezterm-parallel/config.yaml
plugins:
  enabled: true
  auto_load: true
  plugin_directory: "~/.config/wezterm-parallel/plugins"
  
  # セキュリティ設定
  sandbox_enabled: true
  allowed_apis:
    - "workspace"
    - "process"
    - "metrics"
    # - "system"  # システムレベルAPIは制限
```

### 2. シンプルなプラグイン例

```yaml
# ~/.config/wezterm-parallel/plugins/time-tracker/plugin.yaml
plugin:
  name: "time-tracker"
  version: "1.0.0"
  description: "作業時間を追跡し、生産性レポートを生成"
  author: "Your Name"
  license: "MIT"
  
  # 依存関係
  dependencies:
    wezterm_parallel: ">=0.3.0"
    
  # 権限要求
  permissions:
    - "workspace:read"
    - "process:read"
    - "metrics:write"
    - "storage:read_write"
    
  # エントリーポイント
  entry_point: "src/main.py"
  
  # 設定スキーマ
  config_schema:
    daily_goal_hours:
      type: "number"
      default: 8
      min: 1
      max: 24
    reminder_interval_minutes:
      type: "number"
      default: 30
      min: 5
      max: 120
```

```python
# ~/.config/wezterm-parallel/plugins/time-tracker/src/main.py

import asyncio
import json
from datetime import datetime, timedelta
from wezterm_parallel_sdk import Plugin, WorkspaceManager, MetricsCollector

class TimeTrackerPlugin(Plugin):
    def __init__(self):
        super().__init__()
        self.active_sessions = {}
        self.daily_goal = self.config.get('daily_goal_hours', 8)
        
    async def on_workspace_created(self, workspace_id: str, template: str):
        """ワークスペース作成時にタイマー開始"""
        self.active_sessions[workspace_id] = {
            'start_time': datetime.now(),
            'total_duration': timedelta(0),
            'breaks': []
        }
        await self.log_info(f"Time tracking started for workspace: {workspace_id}")
        
    async def on_workspace_closed(self, workspace_id: str):
        """ワークスペース終了時にタイマー停止"""
        if workspace_id in self.active_sessions:
            session = self.active_sessions[workspace_id]
            session['end_time'] = datetime.now()
            duration = session['end_time'] - session['start_time']
            
            # メトリクス保存
            await self.save_session_data(workspace_id, session)
            await self.log_info(f"Time tracking stopped for workspace: {workspace_id}, Duration: {duration}")
            
            del self.active_sessions[workspace_id]
            
    async def on_process_started(self, workspace_id: str, process_name: str):
        """プロセス開始時のアクティビティ記録"""
        if workspace_id in self.active_sessions:
            self.active_sessions[workspace_id]['last_activity'] = datetime.now()
            
    async def save_session_data(self, workspace_id: str, session: dict):
        """セッションデータの保存"""
        metrics = {
            'workspace_id': workspace_id,
            'date': session['start_time'].date().isoformat(),
            'start_time': session['start_time'].isoformat(),
            'end_time': session['end_time'].isoformat(),
            'duration_minutes': (session['end_time'] - session['start_time']).total_seconds() / 60,
            'breaks': session.get('breaks', [])
        }
        
        await MetricsCollector.save('time_tracking', metrics)
        
    async def generate_daily_report(self) -> dict:
        """日次レポート生成"""
        today = datetime.now().date()
        sessions = await MetricsCollector.get_by_date('time_tracking', today)
        
        total_minutes = sum(s['duration_minutes'] for s in sessions)
        total_hours = total_minutes / 60
        
        return {
            'date': today.isoformat(),
            'total_hours': round(total_hours, 2),
            'goal_hours': self.daily_goal,
            'goal_progress': round((total_hours / self.daily_goal) * 100, 1),
            'sessions_count': len(sessions),
            'workspaces': [s['workspace_id'] for s in sessions]
        }

# プラグイン登録
plugin = TimeTrackerPlugin()
```

### 3. ダッシュボードウィジェット作成

```html
<!-- ~/.config/wezterm-parallel/plugins/time-tracker/widgets/daily-summary.html -->
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Time Tracker Widget</title>
    <style>
        .time-tracker-widget {
            background: var(--surface-color);
            border-radius: 8px;
            padding: 16px;
            color: var(--text-color);
        }
        
        .progress-ring {
            width: 100px;
            height: 100px;
            margin: 0 auto;
        }
        
        .progress-circle {
            stroke: var(--primary-color);
            stroke-width: 4;
            fill: transparent;
            stroke-dasharray: 283;
            stroke-dashoffset: 283;
            transition: stroke-dashoffset 0.5s ease;
        }
        
        .stats-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 12px;
            margin-top: 16px;
        }
        
        .stat-item {
            text-align: center;
        }
        
        .stat-value {
            font-size: 24px;
            font-weight: bold;
            color: var(--primary-color);
        }
        
        .stat-label {
            font-size: 12px;
            opacity: 0.7;
        }
    </style>
</head>
<body>
    <div class="time-tracker-widget" id="timeTrackerWidget">
        <h3>Today's Progress</h3>
        
        <!-- 進捗リング -->
        <svg class="progress-ring">
            <circle class="progress-circle" cx="50" cy="50" r="45" id="progressCircle"></circle>
            <text x="50" y="55" text-anchor="middle" font-size="16" fill="var(--text-color)" id="progressText">0%</text>
        </svg>
        
        <!-- 統計情報 -->
        <div class="stats-grid">
            <div class="stat-item">
                <div class="stat-value" id="totalHours">0.0</div>
                <div class="stat-label">Total Hours</div>
            </div>
            <div class="stat-item">
                <div class="stat-value" id="goalHours">8.0</div>
                <div class="stat-label">Goal Hours</div>
            </div>
            <div class="stat-item">
                <div class="stat-value" id="sessionsCount">0</div>
                <div class="stat-label">Sessions</div>
            </div>
            <div class="stat-item">
                <div class="stat-value" id="avgSession">0.0</div>
                <div class="stat-label">Avg Session</div>
            </div>
        </div>
    </div>

    <script>
        class TimeTrackerWidget {
            constructor() {
                this.ws = null;
                this.init();
            }
            
            async init() {
                await this.connectWebSocket();
                await this.loadTodayData();
                this.startPeriodicUpdate();
            }
            
            async connectWebSocket() {
                const wsUrl = 'ws://localhost:8081/plugins/time-tracker/ws';
                this.ws = new WebSocket(wsUrl);
                
                this.ws.onmessage = (event) => {
                    const data = JSON.parse(event.data);
                    if (data.type === 'daily_update') {
                        this.updateDisplay(data.payload);
                    }
                };
            }
            
            async loadTodayData() {
                try {
                    const response = await fetch('/api/plugins/time-tracker/daily-report');
                    const data = await response.json();
                    this.updateDisplay(data);
                } catch (error) {
                    console.error('Failed to load time tracking data:', error);
                }
            }
            
            updateDisplay(data) {
                // 進捗リング更新
                const progress = Math.min(data.goal_progress || 0, 100);
                const circumference = 283;
                const offset = circumference - (progress / 100 * circumference);
                
                document.getElementById('progressCircle').style.strokeDashoffset = offset;
                document.getElementById('progressText').textContent = `${Math.round(progress)}%`;
                
                // 統計情報更新
                document.getElementById('totalHours').textContent = data.total_hours?.toFixed(1) || '0.0';
                document.getElementById('goalHours').textContent = data.goal_hours?.toFixed(1) || '8.0';
                document.getElementById('sessionsCount').textContent = data.sessions_count || 0;
                
                const avgSession = data.sessions_count > 0 ? data.total_hours / data.sessions_count : 0;
                document.getElementById('avgSession').textContent = avgSession.toFixed(1);
                
                // プログレス色の変更
                const circle = document.getElementById('progressCircle');
                if (progress >= 100) {
                    circle.style.stroke = 'var(--success-color)';
                } else if (progress >= 75) {
                    circle.style.stroke = 'var(--primary-color)';
                } else if (progress >= 50) {
                    circle.style.stroke = 'var(--warning-color)';
                } else {
                    circle.style.stroke = 'var(--error-color)';
                }
            }
            
            startPeriodicUpdate() {
                setInterval(() => this.loadTodayData(), 60000); // 1分間隔
            }
        }
        
        // ウィジェット初期化
        document.addEventListener('DOMContentLoaded', () => {
            new TimeTrackerWidget();
        });
    </script>
</body>
</html>
```

## 🔧 API拡張

### 1. カスタムAPIエンドポイント

```rust
// ~/.config/wezterm-parallel/plugins/time-tracker/src/api.rs

use wezterm_parallel::api::{ApiResponse, ApiError};
use wezterm_parallel::plugin::{PluginApi, PluginEndpoint};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DailyReportResponse {
    pub date: String,
    pub total_hours: f64,
    pub goal_hours: f64,
    pub goal_progress: f64,
    pub sessions_count: usize,
    pub workspaces: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct WeeklyReportResponse {
    pub week_start: String,
    pub daily_reports: Vec<DailyReportResponse>,
    pub weekly_total: f64,
    pub weekly_average: f64,
    pub productivity_trend: f64,
}

pub struct TimeTrackerApi;

impl PluginApi for TimeTrackerApi {
    fn endpoints(&self) -> Vec<PluginEndpoint> {
        vec![
            PluginEndpoint {
                path: "/api/plugins/time-tracker/daily-report".to_string(),
                method: "GET".to_string(),
                handler: Box::new(Self::get_daily_report),
            },
            PluginEndpoint {
                path: "/api/plugins/time-tracker/weekly-report".to_string(),
                method: "GET".to_string(),
                handler: Box::new(Self::get_weekly_report),
            },
            PluginEndpoint {
                path: "/api/plugins/time-tracker/start-session".to_string(),
                method: "POST".to_string(),
                handler: Box::new(Self::start_session),
            },
            PluginEndpoint {
                path: "/api/plugins/time-tracker/stop-session".to_string(),
                method: "POST".to_string(),
                handler: Box::new(Self::stop_session),
            },
        ]
    }
}

impl TimeTrackerApi {
    async fn get_daily_report(req: &PluginRequest) -> Result<ApiResponse, ApiError> {
        let date = req.query_param("date")
            .unwrap_or_else(|| chrono::Local::now().date_naive().to_string());
            
        let sessions = TimeTrackingStorage::get_sessions_by_date(&date).await?;
        
        let total_minutes: f64 = sessions.iter()
            .map(|s| s.duration_minutes)
            .sum();
        let total_hours = total_minutes / 60.0;
        
        let goal_hours = 8.0; // 設定から取得
        let goal_progress = (total_hours / goal_hours) * 100.0;
        
        let report = DailyReportResponse {
            date,
            total_hours,
            goal_hours,
            goal_progress,
            sessions_count: sessions.len(),
            workspaces: sessions.into_iter()
                .map(|s| s.workspace_id)
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect(),
        };
        
        Ok(ApiResponse::success(report))
    }
    
    async fn get_weekly_report(req: &PluginRequest) -> Result<ApiResponse, ApiError> {
        let week_start = req.query_param("week_start")
            .unwrap_or_else(|| {
                let now = chrono::Local::now().date_naive();
                let days_from_monday = now.weekday().num_days_from_monday();
                (now - chrono::Duration::days(days_from_monday as i64)).to_string()
            });
            
        let mut daily_reports = Vec::new();
        let mut weekly_total = 0.0;
        
        for i in 0..7 {
            let date = chrono::NaiveDate::parse_from_str(&week_start, "%Y-%m-%d")?
                + chrono::Duration::days(i);
            let date_str = date.to_string();
            
            let sessions = TimeTrackingStorage::get_sessions_by_date(&date_str).await?;
            let total_minutes: f64 = sessions.iter().map(|s| s.duration_minutes).sum();
            let total_hours = total_minutes / 60.0;
            
            weekly_total += total_hours;
            
            daily_reports.push(DailyReportResponse {
                date: date_str,
                total_hours,
                goal_hours: 8.0,
                goal_progress: (total_hours / 8.0) * 100.0,
                sessions_count: sessions.len(),
                workspaces: sessions.into_iter()
                    .map(|s| s.workspace_id)
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect(),
            });
        }
        
        let weekly_average = weekly_total / 7.0;
        
        // 生産性トレンドの計算（簡単な例）
        let recent_average = daily_reports.iter()
            .rev()
            .take(3)
            .map(|r| r.total_hours)
            .sum::<f64>() / 3.0;
        let early_average = daily_reports.iter()
            .take(3)
            .map(|r| r.total_hours)
            .sum::<f64>() / 3.0;
        let productivity_trend = if early_average > 0.0 {
            ((recent_average - early_average) / early_average) * 100.0
        } else {
            0.0
        };
        
        let report = WeeklyReportResponse {
            week_start,
            daily_reports,
            weekly_total,
            weekly_average,
            productivity_trend,
        };
        
        Ok(ApiResponse::success(report))
    }
}
```

## 🎮 高度なワークフロー自動化

### 1. 条件分岐スクリプト

```bash
#!/bin/bash
# ~/.config/wezterm-parallel/scripts/smart-workspace-setup.sh

# プロジェクト自動検出とワークスペース設定

PROJECT_ROOT=$(pwd)
PROJECT_NAME=$(basename "$PROJECT_ROOT")

# プロジェクト種別の検出
detect_project_type() {
    if [[ -f "package.json" ]]; then
        echo "nodejs"
    elif [[ -f "Cargo.toml" ]]; then
        echo "rust"
    elif [[ -f "requirements.txt" ]] || [[ -f "pyproject.toml" ]]; then
        echo "python"
    elif [[ -f "go.mod" ]]; then
        echo "golang"
    elif [[ -f "Dockerfile" ]]; then
        echo "docker"
    else
        echo "generic"
    fi
}

# プロジェクト種別に応じたテンプレート選択
PROJECT_TYPE=$(detect_project_type)
case $PROJECT_TYPE in
    "nodejs")
        TEMPLATE="web-stack"
        ;;
    "rust")
        TEMPLATE="rust-dev"
        ;;
    "python")
        TEMPLATE="python-ml"
        ;;
    "golang")
        TEMPLATE="go-microservices"
        ;;
    "docker")
        TEMPLATE="containerized-app"
        ;;
    *)
        TEMPLATE="basic"
        ;;
esac

echo "🔍 検出されたプロジェクト種別: $PROJECT_TYPE"
echo "🎯 使用するテンプレート: $TEMPLATE"

# ワークスペース作成
curl -X POST http://localhost:8080/api/workspaces \
    -H "Content-Type: application/json" \
    -d "{\"name\": \"$PROJECT_NAME\", \"template\": \"$TEMPLATE\", \"working_dir\": \"$PROJECT_ROOT\"}"

echo "✅ ワークスペース '$PROJECT_NAME' を作成しました"
```

### 2. 環境依存設定

```yaml
# ~/.config/wezterm-parallel/environments/development.yaml
environment: "development"

overrides:
  logging:
    level: "debug"
    console_enabled: true
    
  dashboard:
    update_interval: 1000  # 高頻度更新
    
  process_management:
    auto_restart: true
    health_check_interval: 10
    
  claude_code:
    auto_start: true
    instances: 2
    
  monitoring:
    detailed_metrics: true
    performance_profiling: true
```

```yaml
# ~/.config/wezterm-parallel/environments/production.yaml
environment: "production"

overrides:
  logging:
    level: "warn"
    console_enabled: false
    
  dashboard:
    update_interval: 10000  # 低頻度更新
    
  process_management:
    auto_restart: false
    health_check_interval: 60
    
  claude_code:
    auto_start: false
    instances: 1
    
  monitoring:
    detailed_metrics: false
    performance_profiling: false
    
  security:
    api_key_required: true
    cors_enabled: false
```

### 3. マルチマシン設定同期

```bash
#!/bin/bash
# ~/.config/wezterm-parallel/scripts/sync-config.sh

# 設定ファイルの同期スクリプト

SYNC_SERVER="your-sync-server.com"
CONFIG_DIR="$HOME/.config/wezterm-parallel"
BACKUP_DIR="$HOME/.config/wezterm-parallel-backups"

sync_to_server() {
    echo "🔄 設定をサーバーに同期中..."
    
    # バックアップ作成
    mkdir -p "$BACKUP_DIR"
    cp -r "$CONFIG_DIR" "$BACKUP_DIR/backup-$(date +%Y%m%d-%H%M%S)"
    
    # サーバーに同期
    rsync -avz --delete \
        --exclude='logs/' \
        --exclude='cache/' \
        --exclude='*.lock' \
        "$CONFIG_DIR/" "$SYNC_SERVER:~/.config/wezterm-parallel/"
        
    echo "✅ 同期完了"
}

sync_from_server() {
    echo "📥 サーバーから設定を取得中..."
    
    # バックアップ作成
    mkdir -p "$BACKUP_DIR"
    cp -r "$CONFIG_DIR" "$BACKUP_DIR/backup-$(date +%Y%m%d-%H%M%S)"
    
    # サーバーから同期
    rsync -avz --delete \
        --exclude='logs/' \
        --exclude='cache/' \
        --exclude='*.lock' \
        "$SYNC_SERVER:~/.config/wezterm-parallel/" "$CONFIG_DIR/"
        
    echo "✅ 取得完了"
}

case "${1:-pull}" in
    "push")
        sync_to_server
        ;;
    "pull")
        sync_from_server
        ;;
    *)
        echo "使用方法: $0 [push|pull]"
        exit 1
        ;;
esac
```

## 📊 カスタムメトリクス・アナリティクス

### 1. 独自メトリクス収集

```python
# ~/.config/wezterm-parallel/plugins/custom-metrics/src/collector.py

import asyncio
import psutil
import time
from datetime import datetime
from wezterm_parallel_sdk import MetricsCollector, Plugin

class CustomMetricsCollector(Plugin):
    def __init__(self):
        super().__init__()
        self.collection_interval = 30  # 30秒間隔
        
    async def start_collection(self):
        """カスタムメトリクス収集開始"""
        while True:
            try:
                metrics = await self.collect_system_metrics()
                await MetricsCollector.save('custom_system', metrics)
                
                app_metrics = await self.collect_application_metrics()
                await MetricsCollector.save('custom_application', app_metrics)
                
                productivity_metrics = await self.collect_productivity_metrics()
                await MetricsCollector.save('custom_productivity', productivity_metrics)
                
            except Exception as e:
                await self.log_error(f"Metrics collection failed: {e}")
                
            await asyncio.sleep(self.collection_interval)
            
    async def collect_system_metrics(self) -> dict:
        """システムメトリクス収集"""
        cpu_percent = psutil.cpu_percent(interval=1)
        memory = psutil.virtual_memory()
        disk = psutil.disk_usage('/')
        
        return {
            'timestamp': datetime.now().isoformat(),
            'cpu': {
                'percent': cpu_percent,
                'count': psutil.cpu_count(),
                'frequency': psutil.cpu_freq()._asdict() if psutil.cpu_freq() else None,
            },
            'memory': {
                'total': memory.total,
                'available': memory.available,
                'percent': memory.percent,
                'used': memory.used,
            },
            'disk': {
                'total': disk.total,
                'used': disk.used,
                'free': disk.free,
                'percent': (disk.used / disk.total) * 100,
            },
            'network': self._get_network_stats(),
        }
        
    async def collect_application_metrics(self) -> dict:
        """アプリケーション固有メトリクス"""
        workspaces = await self.api.get_workspaces()
        processes = await self.api.get_processes()
        
        active_workspaces = [w for w in workspaces if w['status'] == 'active']
        running_processes = [p for p in processes if p['status'] == 'running']
        
        return {
            'timestamp': datetime.now().isoformat(),
            'workspaces': {
                'total': len(workspaces),
                'active': len(active_workspaces),
                'templates_used': list(set(w['template'] for w in workspaces)),
            },
            'processes': {
                'total': len(processes),
                'running': len(running_processes),
                'memory_usage': sum(p.get('memory_mb', 0) for p in running_processes),
                'cpu_usage': sum(p.get('cpu_percent', 0) for p in running_processes),
            },
            'api': {
                'requests_count': await self._get_api_request_count(),
                'average_response_time': await self._get_average_response_time(),
                'error_rate': await self._get_error_rate(),
            }
        }
        
    async def collect_productivity_metrics(self) -> dict:
        """生産性メトリクス"""
        # キーボード・マウス活動の検出
        activity_level = await self._detect_user_activity()
        
        # アクティブアプリケーションの追跡
        active_applications = await self._get_active_applications()
        
        # コード変更の追跡（Gitベース）
        code_activity = await self._get_code_activity()
        
        return {
            'timestamp': datetime.now().isoformat(),
            'user_activity': {
                'level': activity_level,  # 0-100
                'active_applications': active_applications,
                'screen_time': await self._get_screen_time(),
            },
            'code_activity': code_activity,
            'focus_score': await self._calculate_focus_score(),
        }
```

### 2. カスタムダッシュボード作成

```html
<!-- ~/.config/wezterm-parallel/dashboards/productivity-dashboard.html -->
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Productivity Dashboard</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            margin: 0;
            padding: 20px;
            background: var(--background-color, #1e1e2e);
            color: var(--text-color, #cdd6f4);
        }
        
        .dashboard-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 20px;
        }
        
        .dashboard-panel {
            background: var(--surface-color, #313244);
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
        }
        
        .panel-title {
            font-size: 18px;
            font-weight: 600;
            margin-bottom: 16px;
            color: var(--primary-color, #89b4fa);
        }
        
        .metric-value {
            font-size: 32px;
            font-weight: 700;
            color: var(--success-color, #a6e3a1);
        }
        
        .metric-label {
            font-size: 14px;
            opacity: 0.7;
            margin-top: 4px;
        }
        
        .chart-container {
            position: relative;
            height: 300px;
            margin-top: 16px;
        }
        
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 16px;
            margin-top: 16px;
        }
        
        .stat-item {
            text-align: center;
            padding: 12px;
            background: rgba(255, 255, 255, 0.05);
            border-radius: 8px;
        }
    </style>
</head>
<body>
    <div class="dashboard-grid" id="dashboardGrid">
        <!-- 今日の生産性サマリー -->
        <div class="dashboard-panel">
            <div class="panel-title">Today's Productivity</div>
            <div class="metric-value" id="productivityScore">85%</div>
            <div class="metric-label">Focus Score</div>
            
            <div class="stats-grid">
                <div class="stat-item">
                    <div class="metric-value" style="font-size: 24px;" id="activeHours">6.5</div>
                    <div class="metric-label">Active Hours</div>
                </div>
                <div class="stat-item">
                    <div class="metric-value" style="font-size: 24px;" id="codeCommits">12</div>
                    <div class="metric-label">Code Commits</div>
                </div>
                <div class="stat-item">
                    <div class="metric-value" style="font-size: 24px;" id="tasksCompleted">8</div>
                    <div class="metric-label">Tasks Completed</div>
                </div>
                <div class="stat-item">
                    <div class="metric-value" style="font-size: 24px;" id="distractions">3</div>
                    <div class="metric-label">Distractions</div>
                </div>
            </div>
        </div>
        
        <!-- 活動時間のトレンド -->
        <div class="dashboard-panel">
            <div class="panel-title">Activity Trend (7 Days)</div>
            <div class="chart-container">
                <canvas id="activityTrendChart"></canvas>
            </div>
        </div>
        
        <!-- プロジェクト時間配分 -->
        <div class="dashboard-panel">
            <div class="panel-title">Project Time Distribution</div>
            <div class="chart-container">
                <canvas id="projectTimeChart"></canvas>
            </div>
        </div>
        
        <!-- システムリソース -->
        <div class="dashboard-panel">
            <div class="panel-title">System Resources</div>
            <div class="chart-container">
                <canvas id="systemResourceChart"></canvas>
            </div>
        </div>
    </div>

    <script>
        class ProductivityDashboard {
            constructor() {
                this.charts = {};
                this.init();
            }
            
            async init() {
                await this.loadData();
                this.createCharts();
                this.startRealTimeUpdates();
            }
            
            async loadData() {
                try {
                    // 各種データの読み込み
                    this.productivityData = await this.fetchData('/api/plugins/custom-metrics/productivity');
                    this.activityData = await this.fetchData('/api/plugins/custom-metrics/activity-trend');
                    this.projectData = await this.fetchData('/api/plugins/custom-metrics/project-time');
                    this.systemData = await this.fetchData('/api/plugins/custom-metrics/system-resources');
                } catch (error) {
                    console.error('Failed to load dashboard data:', error);
                }
            }
            
            async fetchData(endpoint) {
                const response = await fetch(endpoint);
                return await response.json();
            }
            
            createCharts() {
                // 活動トレンドチャート
                const activityCtx = document.getElementById('activityTrendChart').getContext('2d');
                this.charts.activity = new Chart(activityCtx, {
                    type: 'line',
                    data: {
                        labels: this.activityData.labels,
                        datasets: [{
                            label: 'Active Hours',
                            data: this.activityData.activeHours,
                            borderColor: '#89b4fa',
                            backgroundColor: 'rgba(137, 180, 250, 0.1)',
                            tension: 0.4
                        }, {
                            label: 'Focus Score',
                            data: this.activityData.focusScore,
                            borderColor: '#a6e3a1',
                            backgroundColor: 'rgba(166, 227, 161, 0.1)',
                            tension: 0.4
                        }]
                    },
                    options: {
                        responsive: true,
                        maintainAspectRatio: false,
                        plugins: {
                            legend: {
                                labels: { color: '#cdd6f4' }
                            }
                        },
                        scales: {
                            x: { 
                                ticks: { color: '#cdd6f4' },
                                grid: { color: 'rgba(205, 214, 244, 0.1)' }
                            },
                            y: { 
                                ticks: { color: '#cdd6f4' },
                                grid: { color: 'rgba(205, 214, 244, 0.1)' }
                            }
                        }
                    }
                });
                
                // プロジェクト時間配分チャート
                const projectCtx = document.getElementById('projectTimeChart').getContext('2d');
                this.charts.project = new Chart(projectCtx, {
                    type: 'doughnut',
                    data: {
                        labels: this.projectData.labels,
                        datasets: [{
                            data: this.projectData.hours,
                            backgroundColor: [
                                '#89b4fa', '#a6e3a1', '#f9e2af', 
                                '#f38ba8', '#94e2d5', '#f5c2e7'
                            ]
                        }]
                    },
                    options: {
                        responsive: true,
                        maintainAspectRatio: false,
                        plugins: {
                            legend: {
                                position: 'right',
                                labels: { color: '#cdd6f4' }
                            }
                        }
                    }
                });
                
                // システムリソースチャート
                const systemCtx = document.getElementById('systemResourceChart').getContext('2d');
                this.charts.system = new Chart(systemCtx, {
                    type: 'line',
                    data: {
                        labels: this.systemData.timestamps,
                        datasets: [{
                            label: 'CPU Usage (%)',
                            data: this.systemData.cpu,
                            borderColor: '#f38ba8',
                            yAxisID: 'y'
                        }, {
                            label: 'Memory Usage (%)',
                            data: this.systemData.memory,
                            borderColor: '#f9e2af',
                            yAxisID: 'y'
                        }, {
                            label: 'Network (MB/s)',
                            data: this.systemData.network,
                            borderColor: '#94e2d5',
                            yAxisID: 'y1'
                        }]
                    },
                    options: {
                        responsive: true,
                        maintainAspectRatio: false,
                        plugins: {
                            legend: {
                                labels: { color: '#cdd6f4' }
                            }
                        },
                        scales: {
                            x: { 
                                ticks: { color: '#cdd6f4' },
                                grid: { color: 'rgba(205, 214, 244, 0.1)' }
                            },
                            y: {
                                type: 'linear',
                                display: true,
                                position: 'left',
                                ticks: { color: '#cdd6f4' },
                                grid: { color: 'rgba(205, 214, 244, 0.1)' }
                            },
                            y1: {
                                type: 'linear',
                                display: true,
                                position: 'right',
                                ticks: { color: '#cdd6f4' },
                                grid: { drawOnChartArea: false }
                            }
                        }
                    }
                });
            }
            
            updateSummaryStats() {
                if (this.productivityData) {
                    document.getElementById('productivityScore').textContent = 
                        `${Math.round(this.productivityData.focusScore)}%`;
                    document.getElementById('activeHours').textContent = 
                        this.productivityData.activeHours.toFixed(1);
                    document.getElementById('codeCommits').textContent = 
                        this.productivityData.codeCommits;
                    document.getElementById('tasksCompleted').textContent = 
                        this.productivityData.tasksCompleted;
                    document.getElementById('distractions').textContent = 
                        this.productivityData.distractions;
                }
            }
            
            startRealTimeUpdates() {
                // 30秒間隔でデータ更新
                setInterval(async () => {
                    await this.loadData();
                    this.updateCharts();
                    this.updateSummaryStats();
                }, 30000);
            }
            
            updateCharts() {
                // チャートデータの更新ロジック
                Object.values(this.charts).forEach(chart => {
                    chart.update('none');
                });
            }
        }
        
        // ダッシュボード初期化
        document.addEventListener('DOMContentLoaded', () => {
            new ProductivityDashboard();
        });
    </script>
</body>
</html>
```

---

## 🔮 次のステップ

1. **[プラグイン開発ガイド](PLUGIN-DEVELOPMENT.md)**: 詳細なプラグイン開発手順
2. **[テーマ作成チュートリアル](THEME-TUTORIAL.md)**: カスタムテーマの作成方法
3. **[API リファレンス](API-REFERENCE.md)**: 完全なAPI仕様
4. **[コミュニティプラグイン](COMMUNITY-PLUGINS.md)**: 公開プラグイン集

---

🎨 **カスタマイズを楽しんでください！** 

作成したテーマやプラグインは [GitHub Discussions](https://github.com/daktu32/wezterm-parallel/discussions) でぜひ共有してください。