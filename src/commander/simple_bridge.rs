use super::{Task, TaskStore};
use std::io;
use std::path::Path;

/// Very small wrapper to send and receive tasks via a shared file.
#[derive(Debug)]
pub struct SimpleBridge {
    store: TaskStore,
}

impl SimpleBridge {
    /// Create a new bridge using the task store file at `path`.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self { store: TaskStore::new(path.as_ref()) }
    }

    /// Send a task from the commander pane.
    pub fn send_task(&self, command: String) -> io::Result<()> {
        let task = Task { id: chrono::Utc::now().timestamp_millis() as u64, command };
        self.store.add_task(&task)
    }

    /// Retrieve all pending tasks.
    pub fn fetch_tasks(&self) -> io::Result<Vec<Task>> {
        self.store.read_tasks()
    }
}
