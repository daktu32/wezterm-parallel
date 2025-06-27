// WezTerm Multi-Process Development Framework - Process Management Module

pub mod manager;
pub mod pool;
pub mod monitor;
pub mod coordinator;
pub mod router;

pub use manager::{ProcessManager, ProcessConfig, ProcessEvent};
pub use crate::workspace::state::ProcessInfo;
pub use pool::ProcessPool;
pub use monitor::ProcessMonitor;
pub use coordinator::ProcessCoordinator;
pub use router::MessageRouter;