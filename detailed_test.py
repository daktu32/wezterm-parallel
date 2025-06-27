#!/usr/bin/env python3
"""
Detailed test of the WezTerm Parallel task management system
"""

import asyncio
import json
import websockets
import uuid
import time

async def test_task_board_functionality():
    """Test comprehensive task board functionality"""
    uri = "ws://localhost:9999"
    
    try:
        async with websockets.connect(uri) as websocket:
            print("🔗 Connected to WezTerm Parallel WebSocket server")
            
            # Test 1: Request board state
            print("\n📋 Test 1: Requesting task board state...")
            board_request = {
                "id": str(uuid.uuid4()),
                "payload": {
                    "type": "Command", 
                    "data": {
                        "command": "QueryHistory",
                        "params": {
                            "metric_type": "task_board",
                            "start_time": int(time.time()) - 3600,
                            "end_time": int(time.time()),
                            "limit": 10
                        }
                    }
                }
            }
            
            await websocket.send(json.dumps(board_request))
            
            # Test 2: Create a development task
            print("\n📝 Test 2: Creating a development task...")
            task_data = {
                "title": "Implement user authentication",
                "description": "Add OAuth2 integration for user login",
                "category": "Development", 
                "priority": "high",
                "workspace": "backend",
                "estimated_duration": 7200  # 2 hours
            }
            
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
            
            # Test 3: Create a bug fix task
            print("\n🐛 Test 3: Creating a bug fix task...")
            bug_task_data = {
                "title": "Fix memory leak in WebSocket handler",
                "description": "Memory usage increases over time in long-running connections",
                "category": "BugFix",
                "priority": "critical",
                "workspace": "backend"
            }
            
            bug_create_msg = {
                "id": str(uuid.uuid4()),
                "payload": {
                    "type": "Command",
                    "data": {
                        "command": "ExecuteAction", 
                        "params": {
                            "action": {
                                "action": "CreateTask",
                                "params": {"task_data": bug_task_data}
                            }
                        }
                    }
                }
            }
            
            await websocket.send(json.dumps(bug_create_msg))
            
            # Test 4: Request workspace metrics
            print("\n📊 Test 4: Requesting workspace metrics...")
            metrics_msg = {
                "id": str(uuid.uuid4()),
                "payload": {
                    "type": "Command",
                    "data": {
                        "command": "RequestFullUpdate"
                    }
                }
            }
            
            await websocket.send(json.dumps(metrics_msg))
            
            # Listen for comprehensive responses
            print("\n🎧 Monitoring server responses...")
            responses = []
            start_time = time.time()
            
            while time.time() - start_time < 10:  # Listen for 10 seconds
                try:
                    message = await asyncio.wait_for(websocket.recv(), timeout=2.0)
                    data = json.loads(message)
                    responses.append(data)
                    
                    payload_type = data.get("payload", {}).get("type", "Unknown")
                    
                    if payload_type == "TaskBoardUpdate":
                        board_info = data["payload"]["data"] 
                        print(f"   📋 Task Board Update:")
                        print(f"      Board ID: {board_info.get('board_id')}")
                        print(f"      Columns: {len(board_info.get('columns', []))}")
                        for col in board_info.get('columns', []):
                            print(f"         - {col.get('title', 'Unknown')}: {len(col.get('tasks', []))} tasks")
                    
                    elif payload_type == "TaskUpdate":
                        task_info = data["payload"]["data"]
                        print(f"   📝 Task Update: {task_info.get('action', 'Unknown action')}")
                    
                    elif payload_type == "MetricsUpdate":
                        metrics = data["payload"]["data"]
                        print(f"   📊 Metrics Update:")
                        if "framework" in metrics:
                            fw = metrics["framework"]
                            print(f"      Workspaces: {fw.get('total_workspaces', 0)}")
                            print(f"      Processes: {fw.get('total_processes', 0)}")
                    
                    elif payload_type != "Heartbeat":  # Skip heartbeats for cleaner output
                        print(f"   📨 {payload_type}")
                        
                except asyncio.TimeoutError:
                    break  # No more messages
                except json.JSONDecodeError as e:
                    print(f"   ❌ JSON decode error: {e}")
                except Exception as e:
                    print(f"   ⚠️  Error: {e}")
            
            print(f"\n✅ Test completed! Processed {len(responses)} messages")
            
            # Summary
            task_updates = [r for r in responses if r.get("payload", {}).get("type") == "TaskUpdate"]
            board_updates = [r for r in responses if r.get("payload", {}).get("type") == "TaskBoardUpdate"]
            metrics_updates = [r for r in responses if r.get("payload", {}).get("type") == "MetricsUpdate"]
            
            print(f"\n📊 Summary:")
            print(f"   Task Updates: {len(task_updates)}")
            print(f"   Board Updates: {len(board_updates)}")
            print(f"   Metrics Updates: {len(metrics_updates)}")
            
    except Exception as e:
        print(f"❌ Test failed: {e}")

if __name__ == "__main__":
    print("🧪 Detailed WezTerm Parallel System Test")
    print("=" * 45)
    asyncio.run(test_task_board_functionality())