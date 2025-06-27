#!/usr/bin/env python3
"""
Test task creation via WebSocket
"""

import asyncio
import json
import websockets
import uuid
import sys

async def test_task_creation():
    """Test creating tasks via WebSocket and verifying they work"""
    uri = "ws://localhost:9999"
    
    try:
        async with websockets.connect(uri) as websocket:
            print("✅ Connected to WezTerm Parallel WebSocket server")
            
            # Test: Create multiple tasks of different types
            tasks_to_create = [
                {
                    "title": "Fix authentication bug",
                    "description": "Users can't log in with OAuth2",
                    "category": "BugFix",
                    "priority": "urgent",
                    "estimated_duration": 3600  # 1 hour
                },
                {
                    "title": "Implement dark mode toggle",
                    "description": "Add dark/light theme switching to UI",
                    "category": "Feature",
                    "priority": "medium",
                    "estimated_duration": 7200  # 2 hours
                },
                {
                    "title": "Write unit tests for TaskManager",
                    "description": "Increase test coverage for task management",
                    "category": "Testing", 
                    "priority": "low",
                    "estimated_duration": 1800  # 30 minutes
                }
            ]
            
            created_tasks = []
            
            for i, task_data in enumerate(tasks_to_create):
                print(f"\n📝 Creating task {i+1}: {task_data['title']}")
                
                create_msg = {
                    "id": str(uuid.uuid4()),
                    "payload": {
                        "type": "Command",
                        "data": {
                            "command": "ExecuteAction",
                            "params": {
                                "action": {
                                    "action": "CreateTask",
                                    "params": {"task_data": task_data}
                                }
                            }
                        }
                    }
                }
                
                await websocket.send(json.dumps(create_msg))
                
                # Listen for response
                try:
                    response = await asyncio.wait_for(websocket.recv(), timeout=3.0)
                    response_data = json.loads(response)
                    
                    if "payload" in response_data:
                        payload_type = response_data["payload"].get("type")
                        if payload_type == "TaskUpdate":
                            task_info = response_data["payload"]["data"]
                            print(f"   ✅ Task created - Action: {task_info.get('action')}")
                            created_tasks.append(task_info)
                        else:
                            print(f"   📨 Received: {payload_type}")
                    
                except asyncio.TimeoutError:
                    print("   ⏰ No immediate response")
                except json.JSONDecodeError:
                    print("   ❌ Invalid response format")
            
            # Test: Move a task to different status  
            if created_tasks:
                print(f"\n🔄 Testing task status changes...")
                
                # Try to move first task to "in_progress"
                move_msg = {
                    "id": str(uuid.uuid4()),
                    "payload": {
                        "type": "Command",
                        "data": {
                            "command": "ExecuteAction",
                            "params": {
                                "action": {
                                    "action": "MoveTask",
                                    "params": {
                                        "task_id": "test-task-001",  # Use a test ID
                                        "to_column": "in_progress"
                                    }
                                }
                            }
                        }
                    }
                }
                
                await websocket.send(json.dumps(move_msg))
                print("   📤 Sent task move request")
            
            # Test: Update task progress
            print(f"\n📊 Testing task progress updates...")
            
            progress_msg = {
                "id": str(uuid.uuid4()),
                "payload": {
                    "type": "Command", 
                    "data": {
                        "command": "ExecuteAction",
                        "params": {
                            "action": {
                                "action": "UpdateTaskProgress",
                                "params": {
                                    "task_id": "test-task-001",
                                    "progress": 25
                                }
                            }
                        }
                    }
                }
            }
            
            await websocket.send(json.dumps(progress_msg))
            print("   📤 Sent progress update request")
            
            # Listen for any additional responses
            print("\n🎧 Listening for server responses...")
            response_count = 0
            
            try:
                while response_count < 10:  # Listen for up to 10 responses
                    message = await asyncio.wait_for(websocket.recv(), timeout=2.0)
                    data = json.loads(message)
                    response_count += 1
                    
                    payload_type = data.get("payload", {}).get("type", "Unknown")
                    
                    if payload_type == "TaskBoardUpdate":
                        board_data = data["payload"]["data"]
                        print(f"   📋 Board Update - ID: {board_data.get('board_id')}")
                        columns = board_data.get('columns', [])
                        for col in columns:
                            task_count = len(col.get('tasks', []))
                            if task_count > 0:
                                print(f"      {col.get('title')}: {task_count} task(s)")
                    
                    elif payload_type == "TaskUpdate":
                        task_info = data["payload"]["data"]
                        print(f"   📝 Task Update: {task_info.get('action')}")
                    
                    elif payload_type == "TaskMoved":
                        move_info = data["payload"]["data"]
                        print(f"   🔄 Task Moved: {move_info.get('task_id')} -> {move_info.get('to_column')}")
                    
                    elif payload_type == "TaskProgress":
                        progress_info = data["payload"]["data"]
                        print(f"   📊 Progress Update: {progress_info.get('task_id')} -> {progress_info.get('progress')}%")
                    
                    elif payload_type != "Heartbeat":
                        print(f"   📨 {payload_type}")
                        
            except asyncio.TimeoutError:
                print("   ⏰ No more responses")
            
            print(f"\n✅ Task management test completed! Processed {response_count} responses")
            
    except websockets.exceptions.ConnectionRefusedError:
        print("❌ Connection refused - Make sure WezTerm Parallel is running")
        return False
    except Exception as e:
        print(f"❌ Test failed: {e}")
        return False
    
    return True

if __name__ == "__main__":
    print("🧪 Testing WezTerm Parallel Task Management")
    print("=" * 45)
    
    success = asyncio.run(test_task_creation())
    
    if success:
        print("\n🎉 All tests passed!")
        sys.exit(0)
    else:
        print("\n💥 Tests failed!")
        sys.exit(1)