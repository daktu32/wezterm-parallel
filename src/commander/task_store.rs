use serde::{Deserialize, Serialize};
use std::fs::{OpenOptions};
use std::io::{self, Write, Read};
use std::path::PathBuf;

/// Simple representation of a command task.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: u64,
    pub command: String,
}

/// Store tasks in a JSON file that can be shared across panes.
#[derive(Debug)]
pub struct TaskStore {
    path: PathBuf,
}

impl TaskStore {
    /// Create a new task store at the given path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Append a task to the store file.
    pub fn add_task(&self, task: &Task) -> io::Result<()> {
        let mut tasks = self.read_tasks().unwrap_or_else(|_| Vec::new());
        tasks.push(task.clone());
        self.write_tasks(&tasks)
    }

    /// Read all tasks from the store.
    pub fn read_tasks(&self) -> io::Result<Vec<Task>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let mut file = OpenOptions::new().read(true).open(&self.path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        let tasks: Vec<Task> = serde_json::from_str(&data).unwrap_or_default();
        Ok(tasks)
    }

    fn write_tasks(&self, tasks: &[Task]) -> io::Result<()> {
        let json = serde_json::to_string(tasks).unwrap();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;
        file.write_all(json.as_bytes())
    }
}
