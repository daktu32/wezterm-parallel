use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    ProcessUpdate {
        process_id: String,
        status: String,
        timestamp: u64,
    },
    MetricsUpdate {
        cpu_usage: f64,
        memory_usage: f64,
        process_count: u32,
        timestamp: u64,
    },
    WorkspaceUpdate {
        workspace_id: String,
        status: String,
        timestamp: u64,
    },
    SystemAlert {
        level: String,
        message: String,
        timestamp: u64,
    },
}

impl Message {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}