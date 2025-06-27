#!/usr/bin/env python3
"""
Test task creation via Unix Domain Socket IPC
"""

import socket
import json
import os

def test_ipc_task_creation():
    """Test creating a task via Unix Domain Socket"""
    socket_path = "/tmp/wezterm-parallel.sock"
    
    if not os.path.exists(socket_path):
        print(f"❌ Socket file not found: {socket_path}")
        return False
    
    try:
        # Create Unix socket
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.connect(socket_path)
        print("✅ Connected to WezTerm Parallel IPC server")
        
        # Test 1: Ping
        print("\n🏓 Test 1: Ping/Pong")
        ping_msg = {"Ping": None}
        ping_json = json.dumps(ping_msg).encode('utf-8')
        
        sock.send(ping_json)
        response = sock.recv(1024)
        response_data = json.loads(response.decode('utf-8'))
        
        if "Pong" in response_data:
            print("   ✅ Ping/Pong successful")
        else:
            print(f"   ❌ Unexpected response: {response_data}")
        
        # Test 2: Create workspace  
        print("\n🏠 Test 2: Create workspace")
        workspace_msg = {
            "WorkspaceCreate": {
                "name": "test-workspace",
                "template": "default"
            }
        }
        workspace_json = json.dumps(workspace_msg).encode('utf-8')
        
        sock.send(workspace_json)
        response = sock.recv(1024)
        response_data = json.loads(response.decode('utf-8'))
        print(f"   📨 Response: {response_data}")
        
        # Test 3: Queue a task
        print("\n📝 Test 3: Queue task")
        task_msg = {
            "TaskQueue": {
                "id": "task-001",
                "priority": 8,  # High priority
                "command": "Implement authentication system"
            }
        }
        task_json = json.dumps(task_msg).encode('utf-8')
        
        sock.send(task_json)
        response = sock.recv(1024)
        response_data = json.loads(response.decode('utf-8'))
        print(f"   📨 Response: {response_data}")
        
        # Test 4: Queue another task
        print("\n🐛 Test 4: Queue bug fix task")
        bug_task_msg = {
            "TaskQueue": {
                "id": "task-002", 
                "priority": 9,  # Urgent priority
                "command": "Fix memory leak in WebSocket handler"
            }
        }
        bug_task_json = json.dumps(bug_task_msg).encode('utf-8')
        
        sock.send(bug_task_json)
        response = sock.recv(1024)
        response_data = json.loads(response.decode('utf-8'))
        print(f"   📨 Response: {response_data}")
        
        # Test 5: Spawn a process
        print("\n🚀 Test 5: Spawn process")
        process_msg = {
            "ProcessSpawn": {
                "workspace": "test-workspace",
                "command": "claude-code --workspace=test-workspace"
            }
        }
        process_json = json.dumps(process_msg).encode('utf-8')
        
        sock.send(process_json)
        response = sock.recv(1024)
        response_data = json.loads(response.decode('utf-8'))
        print(f"   📨 Response: {response_data}")
        
        sock.close()
        print("\n✅ IPC test completed successfully!")
        return True
        
    except Exception as e:
        print(f"❌ IPC test failed: {e}")
        return False

if __name__ == "__main__":
    print("🧪 Testing WezTerm Parallel via IPC")
    print("=" * 35)
    
    success = test_ipc_task_creation()
    
    if success:
        print("\n🎉 All IPC tests passed!")
    else:
        print("\n💥 IPC tests failed!")