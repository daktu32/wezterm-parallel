// WezTerm Multi-Process Development Framework - Library

pub mod workspace;
pub mod process;
pub mod config;
pub mod metrics;
pub mod dashboard;
pub mod task;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Message {
    WorkspaceCreate { name: String, template: String },
    ProcessSpawn { workspace: String, command: String },
    StatusUpdate { process_id: String, status: String },
    TaskQueue { id: String, priority: u8, command: String },
    Ping,
    Pong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let message = Message::Ping;
        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(message, deserialized);
    }

    #[test]
    fn test_workspace_create_message() {
        let message = Message::WorkspaceCreate {
            name: "test-workspace".to_string(),
            template: "default".to_string(),
        };
        
        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            Message::WorkspaceCreate { name, template } => {
                assert_eq!(name, "test-workspace");
                assert_eq!(template, "default");
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn test_process_spawn_message() {
        let message = Message::ProcessSpawn {
            workspace: "frontend".to_string(),
            command: "claude-code --workspace=frontend".to_string(),
        };
        
        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            Message::ProcessSpawn { workspace, command } => {
                assert_eq!(workspace, "frontend");
                assert_eq!(command, "claude-code --workspace=frontend");
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn test_task_queue_message() {
        let message = Message::TaskQueue {
            id: "task-001".to_string(),
            priority: 5,
            command: "build project".to_string(),
        };
        
        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            Message::TaskQueue { id, priority, command } => {
                assert_eq!(id, "task-001");
                assert_eq!(priority, 5);
                assert_eq!(command, "build project");
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn test_status_update_message() {
        let message = Message::StatusUpdate {
            process_id: "claude-001".to_string(),
            status: "running".to_string(),
        };
        
        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            Message::StatusUpdate { process_id, status } => {
                assert_eq!(process_id, "claude-001");
                assert_eq!(status, "running");
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn test_ping_pong_messages() {
        let ping = Message::Ping;
        let pong = Message::Pong;
        
        let ping_serialized = serde_json::to_string(&ping).unwrap();
        let pong_serialized = serde_json::to_string(&pong).unwrap();
        
        let ping_deserialized: Message = serde_json::from_str(&ping_serialized).unwrap();
        let pong_deserialized: Message = serde_json::from_str(&pong_serialized).unwrap();
        
        assert_eq!(ping, ping_deserialized);
        assert_eq!(pong, pong_deserialized);
    }

    #[test]
    fn test_invalid_json_handling() {
        let invalid_json = r#"{"invalid": "json structure"}"#;
        let result: Result<Message, _> = serde_json::from_str(invalid_json);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_message_size_limits() {
        // Test with very long strings to ensure we handle large messages
        let long_name = "x".repeat(1000);
        let long_template = "y".repeat(1000);
        
        let message = Message::WorkspaceCreate {
            name: long_name.clone(),
            template: long_template.clone(),
        };
        
        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            Message::WorkspaceCreate { name, template } => {
                assert_eq!(name, long_name);
                assert_eq!(template, long_template);
            }
            _ => panic!("Unexpected message type"),
        }
    }
}