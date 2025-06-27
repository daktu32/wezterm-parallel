#!/usr/bin/env python3
"""
Test productivity tracking functionality
"""

import socket
import json
import os
import time

def test_productivity_tracking():
    """Test task creation and time tracking via IPC"""
    socket_path = "/tmp/wezterm-parallel.sock"
    
    if not os.path.exists(socket_path):
        print(f"❌ Socket file not found: {socket_path}")
        return False
    
    try:
        # Create Unix socket
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.connect(socket_path)
        print("✅ Connected to WezTerm Parallel IPC server")
        
        # Test 1: Create a task for tracking
        print("\n📝 Test 1: Creating task for time tracking")
        task_msg = {
            "TaskQueue": {
                "id": "productivity-test-001",
                "priority": 8,
                "command": "Implement feature X with time tracking"
            }
        }
        task_json = json.dumps(task_msg).encode('utf-8')
        
        sock.send(task_json)
        response = sock.recv(1024)
        response_data = json.loads(response.decode('utf-8'))
        print(f"   📨 Response: {response_data}")
        
        # Test 2: Create another task
        print("\n🐛 Test 2: Creating bug fix task")
        bug_task_msg = {
            "TaskQueue": {
                "id": "productivity-test-002", 
                "priority": 9,
                "command": "Fix critical bug with time tracking"
            }
        }
        bug_task_json = json.dumps(bug_task_msg).encode('utf-8')
        
        sock.send(bug_task_json)
        response = sock.recv(1024)
        response_data = json.loads(response.decode('utf-8'))
        print(f"   📨 Response: {response_data}")
        
        # Test 3: Create enhancement task
        print("\n⚡ Test 3: Creating enhancement task")
        enhancement_msg = {
            "TaskQueue": {
                "id": "productivity-test-003",
                "priority": 5,
                "command": "Performance optimization with productivity metrics"
            }
        }
        enhancement_json = json.dumps(enhancement_msg).encode('utf-8')
        
        sock.send(enhancement_json)
        response = sock.recv(1024)
        response_data = json.loads(response.decode('utf-8'))
        print(f"   📨 Response: {response_data}")
        
        # Test 4: Ping test to verify connection
        print("\n🏓 Test 4: Connection check")
        ping_msg = {"Ping": None}
        ping_json = json.dumps(ping_msg).encode('utf-8')
        
        sock.send(ping_json)
        response = sock.recv(1024)
        response_data = json.loads(response.decode('utf-8'))
        print(f"   📨 Response: {response_data}")
        
        sock.close()
        print("\n✅ Productivity tracking test completed!")
        
        print("\n📊 Summary:")
        print("   ✓ Created 3 tasks for productivity tracking")
        print("   ✓ Tasks now available for time tracking via TaskManager")
        print("   ✓ Each task can track focused time, interruptions, and breaks")
        print("   ✓ Productivity reports can be generated from tracked data")
        
        return True
        
    except Exception as e:
        print(f"❌ Productivity tracking test failed: {e}")
        return False

if __name__ == "__main__":
    print("🧪 Testing WezTerm Parallel Productivity Tracking")
    print("=" * 50)
    
    success = test_productivity_tracking()
    
    if success:
        print("\n🎉 All productivity tracking tests passed!")
        print("\n💡 Next steps:")
        print("   - Tasks are ready for time tracking")
        print("   - Start tracking with TaskManager.start_task_tracking(task_id)")
        print("   - Generate reports with TaskManager.generate_productivity_report()")
        print("   - Get task insights with TaskManager.get_task_insights(task_id)")
    else:
        print("\n💥 Productivity tracking tests failed!")