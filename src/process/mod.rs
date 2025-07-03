// WezTerm Multi-Process Development Framework - Process Management Module

pub mod manager;
pub mod pool;
pub mod monitor;
pub mod coordinator;
pub mod router;
pub mod detector;
pub mod claude_config;
pub mod claude_health;
pub mod claude_logger;

pub use manager::{ProcessManager, ProcessConfig, ProcessEvent};
pub use crate::room::state::ProcessInfo;
pub use pool::ProcessPool;
pub use monitor::ProcessMonitor;
pub use coordinator::ProcessCoordinator;
pub use router::MessageRouter;
pub use detector::ClaudeCodeDetector;
pub use claude_config::{ClaudeCodeConfig, ClaudeCodeConfigBuilder, WorkspaceSpecificConfig};
pub use claude_health::{ClaudeHealthMonitor, HealthState, HealthStatus, HealthConfig};
pub use claude_logger::{ClaudeLogger, LogConfig, LogEntry, LogLevel, LogSource, DebugInfo, DebugType, LogStatistics};