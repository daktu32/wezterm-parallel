//! Minimal task dispatch system for wezterm-parallel.

pub mod task_store;
pub mod simple_bridge;

pub use task_store::{Task, TaskStore};
pub use simple_bridge::SimpleBridge;
