# API仕様書

## 概要

WezTermマルチプロセス開発補助ツールのAPI仕様を定義します。

## 1. IPC API (Unix Domain Socket)

### エンドポイント
- Socket Path: `/tmp/wezterm-parallel.sock`
- Protocol: JSON over Unix Socket
- Encoding: UTF-8

### メッセージフォーマット

#### リクエスト
```json
{
  "MessageType": {
    "field": "value"
  }
}
```

#### レスポンス
```json
{
  "Success": {
    "data": {}
  }
}
// または
{
  "Error": "error message"
}
```

## 2. コアメッセージタイプ

### 2.1 システム管理

#### Ping
```json
{ "Ping": null }
```
レスポンス:
```json
{ "Pong": null }
```

#### GetStatus
```json
{ "GetStatus": null }
```
レスポンス:
```json
{
  "Status": {
    "rooms": [...],
    "processes": [...],
    "system_metrics": {...}
  }
}
```

### 2.2 Room管理

#### RoomCreate
```json
{
  "RoomCreate": {
    "name": "project-name",
    "template": "default|web_dev|rust_dev|research"
  }
}
```

#### RoomSwitch
```json
{
  "RoomSwitch": {
    "name": "room-name"
  }
}
```

#### RoomDelete
```json
{
  "RoomDelete": {
    "name": "room-name"
  }
}
```

### 2.3 プロセス管理

#### ProcessSpawn
```json
{
  "ProcessSpawn": {
    "id": "process-id",
    "command": "claude-code",
    "args": ["--room", "main"],
    "room": "room-name",
    "env": {
      "CLAUDE_SESSION": "session-id"
    }
  }
}
```

#### ProcessKill
```json
{
  "ProcessKill": {
    "id": "process-id"
  }
}
```

### 2.4 協調メッセージ (Issue #17)

#### CoordinationMessage
```json
{
  "CoordinationMessage": {
    "TaskAssignment": {
      "from": "coordinator",
      "to": "process-1",
      "task": {
        "id": "task-123",
        "command": "implement feature X",
        "dependencies": []
      }
    }
  }
}
```

#### TaskResult
```json
{
  "CoordinationMessage": {
    "TaskResult": {
      "from": "process-1",
      "to": "coordinator",
      "result": {
        "task_id": "task-123",
        "status": "completed",
        "output": "..."
      }
    }
  }
}
```

### 2.5 ファイル同期

#### FileSync
```json
{
  "CoordinationMessage": {
    "SyncRequest": {
      "from": "process-1",
      "files": ["src/main.rs", "tests/test.rs"]
    }
  }
}
```

## 3. WebSocket API

### エンドポイント
- URL: `ws://localhost:9999`
- Protocol: WebSocket
- Message Format: JSON

### メッセージタイプ

#### MetricsUpdate
```json
{
  "type": "metrics",
  "data": {
    "cpu": 25.5,
    "memory": 1024000,
    "processes": [...]
  }
}
```

#### TaskBoardUpdate
```json
{
  "type": "task_board",
  "data": {
    "todo": [...],
    "in_progress": [...],
    "done": [...]
  }
}
```

## 4. Lua API (WezTerm統合)

### 4.1 Room操作

```lua
-- Room作成
wezterm_parallel.create_room({
  name = "my-project",
  template = "claude-dev"
})

-- Room切り替え
wezterm_parallel.switch_room("my-project")

-- Room一覧
local rooms = wezterm_parallel.list_rooms()
```

### 4.2 テンプレート適用 (Issue #18)

```lua
-- テンプレート適用
wezterm_parallel.apply_template({
  template_path = "~/.config/wezterm/templates/claude-dev.yaml"
})

-- テンプレート保存
wezterm_parallel.save_current_layout({
  name = "my-layout",
  description = "My custom development layout"
})
```

### 4.3 プロセス管理

```lua
-- プロセス起動
wezterm_parallel.spawn_process({
  id = "claude-main",
  command = "claude-code",
  room = "current"
})

-- プロセス状態取得
local status = wezterm_parallel.get_process_status("claude-main")
```

## 5. エラーコード

| コード | 説明 |
|-------|------|
| 1001 | Roomが見つからない |
| 1002 | Room作成失敗 |
| 2001 | プロセスが見つからない |
| 2002 | プロセス起動失敗 |
| 3001 | タスクが見つからない |
| 3002 | タスク実行失敗 |
| 4001 | ファイル同期エラー |
| 4002 | マージコンフリクト |

## 6. レート制限

- IPC: 無制限（ローカル）
- WebSocket: 100メッセージ/秒
- ファイル監視: 1000ファイル/プロジェクト

## 7. セキュリティ

- Unix Socket: ファイルシステム権限による保護
- WebSocket: ローカルホストのみ
- プロセス分離: 各プロセスは独立したセッション

## 8. バージョニング

現在のAPIバージョン: v0.1.0

互換性ポリシー:
- マイナーバージョン: 後方互換性維持
- メジャーバージョン: 破壊的変更あり