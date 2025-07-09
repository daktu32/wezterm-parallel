// WezTerm Multi-Process Development Framework - Process Management Module

pub mod claude_config;
pub mod claude_health;
pub mod claude_logger;
pub mod coordinator;
pub mod detector;
pub mod manager;
pub mod monitor;
pub mod pool;
pub mod router;

pub use crate::room::state::ProcessInfo;
pub use claude_config::{ClaudeCodeConfig, ClaudeCodeConfigBuilder, WorkspaceSpecificConfig};
pub use claude_health::{ClaudeHealthMonitor, HealthConfig, HealthState, HealthStatus};
pub use claude_logger::{
    ClaudeLogger, DebugInfo, DebugType, LogConfig, LogEntry, LogLevel, LogSource, LogStatistics,
};
pub use coordinator::ProcessCoordinator;
pub use detector::ClaudeCodeDetector;
pub use manager::{ProcessConfig, ProcessEvent, ProcessManager};
pub use monitor::ProcessMonitor;
pub use pool::ProcessPool;
pub use router::MessageRouter;
