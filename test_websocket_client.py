#!/usr/bin/env python3
"""
Simple WebSocket client to test the WezTerm Parallel task management system
"""

import asyncio
import json
import websockets
import uuid

async def test_task_management():
    """Test the task management functionality via WebSocket"""
    uri = "ws://localhost:9999"
    
    try:
        async with websockets.connect(uri) as websocket:
            print("✅ Connected to WezTerm Parallel WebSocket server")
            
            # Test 1: Subscribe to task board updates
            subscribe_msg = {
                "id": str(uuid.uuid4()),
                "payload": {
                    "type": "Command",
                    "data": {
                        "command": "Subscribe",
                        "params": {
                            "subscriptions": ["All"]
                        }
                    }
                }
            }
            
            await websocket.send(json.dumps(subscribe_msg))
            print("📡 Sent subscription request")
            
            # Test 2: Create a new task
            task_data = {
                "title": "Test Task via WebSocket",
                "description": "Testing the task management system",
                "category": "Development",
                "priority": "high"
            }
            
            create_task_msg = {
                "id": str(uuid.uuid4()),
                "payload": {
                    "type": "Command",
                    "data": {
                        "command": "ExecuteAction",
                        "params": {
                            "action": {
                                "action": "CreateTask",
                                "params": {
                                    "task_data": task_data
                                }
                            }
                        }
                    }
                }
            }
            
            await websocket.send(json.dumps(create_task_msg))
            print("📝 Sent task creation request")
            
            # Test 3: Request full update
            full_update_msg = {
                "id": str(uuid.uuid4()),
                "payload": {
                    "type": "Command",
                    "data": {
                        "command": "RequestFullUpdate"
                    }
                }
            }
            
            await websocket.send(json.dumps(full_update_msg))
            print("🔄 Requested full update")
            
            # Listen for responses
            print("\n🎧 Listening for server responses...")
            response_count = 0
            
            async for message in websocket:
                try:
                    data = json.loads(message)
                    response_count += 1
                    
                    print(f"\n📨 Response {response_count}:")
                    if "payload" in data:
                        payload_type = data["payload"].get("type", "Unknown")
                        print(f"   Type: {payload_type}")
                        
                        if payload_type == "TaskBoardUpdate":
                            board_data = data["payload"]["data"]
                            print(f"   Board ID: {board_data.get('board_id')}")
                            print(f"   Columns: {len(board_data.get('columns', []))}")
                            
                        elif payload_type == "TaskUpdate":
                            task_info = data["payload"]["data"]
                            print(f"   Action: {task_info.get('action')}")
                            
                        elif payload_type == "MetricsUpdate":
                            print("   📊 Metrics update received")
                            
                        elif payload_type == "Heartbeat":
                            print("   💓 Heartbeat received")
                    
                    # Stop after receiving a few responses
                    if response_count >= 5:
                        break
                        
                except json.JSONDecodeError:
                    print(f"   ❌ Invalid JSON: {message}")
                except Exception as e:
                    print(f"   ⚠️  Error processing message: {e}")
            
            print(f"\n✅ Test completed! Received {response_count} responses")
            
    except websockets.exceptions.ConnectionRefusedError:
        print("❌ Connection refused - Make sure the WezTerm Parallel server is running")
    except Exception as e:
        print(f"❌ Error: {e}")

if __name__ == "__main__":
    print("🚀 Testing WezTerm Parallel Task Management System")
    print("=" * 50)
    asyncio.run(test_task_management())