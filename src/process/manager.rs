// WezTerm Multi-Process Development Framework - Process Manager

use crate::logging::enhancer::process;
use crate::logging::LogContext;
use crate::{log_debug, log_error, log_info, log_warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;

use crate::room::state::{ProcessInfo, ProcessStatus};

#[derive(Debug)]
pub struct ProcessManager {
    processes: RwLock<HashMap<String, ManagedProcess>>,
    config: ProcessConfig,
    event_sender: mpsc::UnboundedSender<ProcessEvent>,
}

#[derive(Debug)]
pub struct ManagedProcess {
    info: ProcessInfo,
    child: Option<Child>,
    output_monitor: Option<tokio::task::JoinHandle<()>>,
    health_monitor: Option<tokio::task::JoinHandle<()>>,
    #[allow(dead_code)]
    restart_policy: RestartPolicy,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessConfig {
    pub claude_code_binary: String,
    pub max_processes: usize,
    pub health_check_interval_secs: u64,
    pub restart_delay_secs: u64,
    pub max_restart_attempts: u32,
    pub process_timeout_secs: u64,
    pub default_restart_policy: RestartPolicy,
    pub environment_vars: HashMap<String, String>,
    pub working_directory: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RestartPolicy {
    Never,
    Always,
    OnFailure,
    OnFailureWithLimit(u32),
}

#[derive(Debug, Clone)]
pub enum ProcessEvent {
    Started {
        process_id: String,
        pid: u32,
        workspace: String,
    },
    Stopped {
        process_id: String,
        exit_code: Option<i32>,
        workspace: String,
    },
    Failed {
        process_id: String,
        error: String,
        workspace: String,
    },
    OutputLine {
        process_id: String,
        line: String,
        is_stderr: bool,
    },
    HealthCheck {
        process_id: String,
        is_healthy: bool,
    },
    Restarting {
        process_id: String,
        attempt: u32,
    },
}

impl Default for ProcessConfig {
    fn default() -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("RUST_LOG".to_string(), "info".to_string());

        Self {
            claude_code_binary: "claude-code".to_string(),
            max_processes: 16,
            health_check_interval_secs: 30,
            restart_delay_secs: 5,
            max_restart_attempts: 3,
            process_timeout_secs: 300, // 5 minutes
            default_restart_policy: RestartPolicy::OnFailureWithLimit(3),
            environment_vars: env_vars,
            working_directory: None,
        }
    }
}

impl ProcessConfig {
    #[cfg(test)]
    pub fn default_for_testing() -> Self {
        use std::collections::HashMap;
        Self {
            claude_code_binary: "echo".to_string(), // Use echo for testing instead of actual claude-code
            max_processes: 16,
            health_check_interval_secs: 1,
            restart_delay_secs: 1,
            max_restart_attempts: 3,
            process_timeout_secs: 10,
            default_restart_policy: RestartPolicy::OnFailure,
            environment_vars: HashMap::new(),
            working_directory: None,
        }
    }
}

impl ProcessManager {
    pub fn new(config: ProcessConfig) -> (Self, mpsc::UnboundedReceiver<ProcessEvent>) {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let manager = Self {
            processes: RwLock::new(HashMap::new()),
            config,
            event_sender,
        };

        (manager, event_receiver)
    }

    pub async fn spawn_process(
        &self,
        process_id: String,
        workspace: String,
        command_args: Vec<String>,
    ) -> Result<(), String> {
        // Check if process already exists
        {
            let processes = self.processes.read().await;
            if processes.contains_key(&process_id) {
                return Err(format!("Process '{process_id}' already exists"));
            }
        }

        // Check process limit
        {
            let processes = self.processes.read().await;
            if processes.len() >= self.config.max_processes {
                return Err(format!(
                    "Maximum process limit ({}) reached",
                    self.config.max_processes
                ));
            }
        }

        // 統一ログ: プロセス起動開始
        let command_string = format!(
            "{} {}",
            self.config.claude_code_binary,
            command_args.join(" ")
        );
        process::log_process_start(&process_id, &command_string);

        let context = LogContext::new("process", "spawn")
            .with_entity_id(&process_id)
            .with_metadata("workspace", serde_json::json!(workspace));
        log_info!(
            context,
            "Spawning process '{}' in workspace '{}'",
            process_id,
            workspace
        );

        let mut cmd = Command::new(&self.config.claude_code_binary);
        cmd.args(&command_args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);

        // Set environment variables
        for (key, value) in &self.config.environment_vars {
            cmd.env(key, value);
        }

        // Set working directory
        if let Some(ref wd) = self.config.working_directory {
            cmd.current_dir(wd);
        }

        // Add workspace-specific environment
        cmd.env("CLAUDE_WORKSPACE", &workspace);
        cmd.env("CLAUDE_PROCESS_ID", &process_id);

        let start_time = std::time::Instant::now();
        let mut child = cmd.spawn().map_err(|e| {
            let context = LogContext::new("process", "spawn_error").with_entity_id(&process_id);
            log_error!(context, "Failed to spawn process '{}': {}", process_id, e);
            // 統一ログ: プロセス起動エラー
            process::log_process_error(&process_id, &format!("Spawn failed: {e}"));
            format!("Failed to spawn process: {e}")
        })?;

        let pid = child.id().unwrap_or(0);
        let _spawn_duration = start_time.elapsed();

        // 統一ログ: プロセス起動成功
        let context = LogContext::new("process", "spawn_success")
            .with_entity_id(&process_id)
            .with_metadata("workspace", serde_json::json!(workspace))
            .with_metadata("pid", serde_json::json!(pid))
            .with_metadata("command", serde_json::json!(command_string));
        log_info!(context, "Process spawned successfully");

        // Create process info
        let process_info = ProcessInfo {
            id: process_id.clone(),
            command: format!(
                "{} {}",
                self.config.claude_code_binary,
                command_args.join(" ")
            ),
            workspace: workspace.clone(),
            pane_id: None,
            status: ProcessStatus::Starting,
            pid: Some(pid),
            started_at: SystemTime::now(),
            last_heartbeat: SystemTime::now(),
            restart_count: 0,
        };

        // Setup output monitoring
        let output_monitor = self.spawn_output_monitor(&process_id, &mut child).await;
        let health_monitor = self.spawn_health_monitor(&process_id).await;

        let managed_process = ManagedProcess {
            info: process_info,
            child: Some(child),
            output_monitor: Some(output_monitor),
            health_monitor: Some(health_monitor),
            restart_policy: self.config.default_restart_policy.clone(),
        };

        // Add to collection
        {
            let mut processes = self.processes.write().await;
            processes.insert(process_id.clone(), managed_process);
        }

        // Update status to running
        self.update_process_status(&process_id, ProcessStatus::Running)
            .await;

        // Send event
        let _ = self.event_sender.send(ProcessEvent::Started {
            process_id,
            pid,
            workspace,
        });

        Ok(())
    }

    pub async fn kill_process(&self, process_id: &str) -> Result<(), String> {
        // 統一ログ: プロセス停止開始
        let context = LogContext::new("process", "kill_start").with_entity_id(process_id);
        log_info!(context, "Initiating process termination");

        let kill_context = LogContext::new("process", "kill").with_entity_id(process_id);
        log_info!(kill_context, "Killing process '{}'", process_id);

        let mut processes = self.processes.write().await;

        if let Some(managed_process) = processes.get_mut(process_id) {
            // Cancel monitors
            if let Some(output_monitor) = managed_process.output_monitor.take() {
                output_monitor.abort();
            }
            if let Some(health_monitor) = managed_process.health_monitor.take() {
                health_monitor.abort();
            }

            // Kill child process
            if let Some(mut child) = managed_process.child.take() {
                if let Err(e) = child.kill().await {
                    let warn_context =
                        LogContext::new("process", "kill_failure").with_entity_id(process_id);
                    log_warn!(
                        warn_context,
                        "Failed to kill process '{}': {}",
                        process_id,
                        e
                    );
                    // 統一ログ: プロセス停止エラー
                    process::log_process_error(process_id, &format!("Kill failed: {e}"));
                } else {
                    // 統一ログ: プロセス停止成功
                    process::log_process_stop(process_id, None);
                }
            }

            managed_process.info.status = ProcessStatus::Stopped;

            let _ = self.event_sender.send(ProcessEvent::Stopped {
                process_id: process_id.to_string(),
                exit_code: None,
                workspace: managed_process.info.workspace.clone(),
            });

            Ok(())
        } else {
            // 統一ログ: プロセス未発見エラー
            let context = LogContext::new("process", "kill_not_found").with_entity_id(process_id);
            log_warn!(context, "Process not found for termination");
            Err(format!("Process '{process_id}' not found"))
        }
    }

    pub async fn restart_process(&self, process_id: &str) -> Result<(), String> {
        let restart_context = LogContext::new("process", "restart").with_entity_id(process_id);
        log_info!(restart_context, "Restarting process '{}'", process_id);

        // Get process info before killing
        let (workspace, command_args, restart_count) = {
            let processes = self.processes.read().await;
            if let Some(managed_process) = processes.get(process_id) {
                let command_parts: Vec<String> = managed_process
                    .info
                    .command
                    .split_whitespace()
                    .skip(1) // Skip binary name
                    .map(|s| s.to_string())
                    .collect();

                (
                    managed_process.info.workspace.clone(),
                    command_parts,
                    managed_process.info.restart_count,
                )
            } else {
                return Err(format!("Process '{process_id}' not found"));
            }
        };

        // Send restart event
        let _ = self.event_sender.send(ProcessEvent::Restarting {
            process_id: process_id.to_string(),
            attempt: restart_count + 1,
        });

        // Kill existing process
        self.kill_process(process_id).await?;

        // Remove from collection
        {
            let mut processes = self.processes.write().await;
            processes.remove(process_id);
        }

        // Wait for restart delay
        sleep(Duration::from_secs(self.config.restart_delay_secs)).await;

        // Spawn new process
        self.spawn_process(process_id.to_string(), workspace, command_args)
            .await?;

        // Update restart count
        {
            let mut processes = self.processes.write().await;
            if let Some(managed_process) = processes.get_mut(process_id) {
                managed_process.info.restart_count = restart_count + 1;
            }
        }

        Ok(())
    }

    pub async fn get_process_info(&self, process_id: &str) -> Option<ProcessInfo> {
        let processes = self.processes.read().await;
        processes.get(process_id).map(|p| p.info.clone())
    }

    pub async fn list_processes(&self) -> Vec<ProcessInfo> {
        let processes = self.processes.read().await;
        processes.values().map(|p| p.info.clone()).collect()
    }

    pub async fn get_processes_by_workspace(&self, workspace: &str) -> Vec<ProcessInfo> {
        let processes = self.processes.read().await;
        processes
            .values()
            .filter(|p| p.info.workspace == workspace)
            .map(|p| p.info.clone())
            .collect()
    }

    async fn update_process_status(&self, process_id: &str, status: ProcessStatus) {
        let mut processes = self.processes.write().await;
        if let Some(managed_process) = processes.get_mut(process_id) {
            managed_process.info.status = status;
            managed_process.info.last_heartbeat = SystemTime::now();
        }
    }

    async fn spawn_output_monitor(
        &self,
        process_id: &str,
        child: &mut Child,
    ) -> tokio::task::JoinHandle<()> {
        let process_id = process_id.to_string();
        let event_sender = self.event_sender.clone();

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        tokio::spawn(async move {
            let mut stdout_reader = BufReader::new(stdout).lines();
            let mut stderr_reader = BufReader::new(stderr).lines();

            loop {
                tokio::select! {
                    line = stdout_reader.next_line() => {
                        match line {
                            Ok(Some(line)) => {
                                let debug_context = LogContext::new("process", "stdout")
                                    .with_entity_id(&process_id);
                                log_debug!(debug_context, "Process '{}' stdout: {}", process_id, line);
                                let _ = event_sender.send(ProcessEvent::OutputLine {
                                    process_id: process_id.clone(),
                                    line,
                                    is_stderr: false,
                                });
                            }
                            Ok(None) => break, // EOF
                            Err(e) => {
                                let error_context = LogContext::new("process", "stdout_error")
                                    .with_entity_id(&process_id);
                                log_error!(error_context, "Error reading stdout for process '{}': {}", process_id, e);
                                break;
                            }
                        }
                    }
                    line = stderr_reader.next_line() => {
                        match line {
                            Ok(Some(line)) => {
                                let debug_context = LogContext::new("process", "stderr")
                                    .with_entity_id(&process_id);
                                log_debug!(debug_context, "Process '{}' stderr: {}", process_id, line);
                                let _ = event_sender.send(ProcessEvent::OutputLine {
                                    process_id: process_id.clone(),
                                    line,
                                    is_stderr: true,
                                });
                            }
                            Ok(None) => break, // EOF
                            Err(e) => {
                                let error_context = LogContext::new("process", "stderr_error")
                                    .with_entity_id(&process_id);
                                log_error!(error_context, "Error reading stderr for process '{}': {}", process_id, e);
                                break;
                            }
                        }
                    }
                }
            }

            let debug_context =
                LogContext::new("process", "monitor_terminated").with_entity_id(&process_id);
            log_debug!(
                debug_context,
                "Output monitor for process '{}' terminated",
                process_id
            );
        })
    }

    async fn spawn_health_monitor(&self, process_id: &str) -> tokio::task::JoinHandle<()> {
        let process_id = process_id.to_string();
        let event_sender = self.event_sender.clone();
        let check_interval = Duration::from_secs(self.config.health_check_interval_secs);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);

            loop {
                interval.tick().await;

                // TODO: Implement actual health check (e.g., ping process via IPC)
                let is_healthy = true; // Placeholder

                let _ = event_sender.send(ProcessEvent::HealthCheck {
                    process_id: process_id.clone(),
                    is_healthy,
                });

                let health_context = LogContext::new("process", "health_check")
                    .with_entity_id(&process_id)
                    .with_metadata("is_healthy", serde_json::json!(is_healthy));
                log_debug!(
                    health_context,
                    "Health check for process '{}': {}",
                    process_id,
                    if is_healthy { "healthy" } else { "unhealthy" }
                );
            }
        })
    }

    pub async fn cleanup_finished_processes(&self) -> usize {
        let mut processes = self.processes.write().await;
        let initial_count = processes.len();

        // Check for finished processes
        let mut to_remove = Vec::new();

        for (process_id, managed_process) in processes.iter_mut() {
            if let Some(ref mut child) = managed_process.child {
                match child.try_wait() {
                    Ok(Some(exit_status)) => {
                        let finish_context = LogContext::new("process", "finished")
                            .with_entity_id(process_id)
                            .with_metadata(
                                "exit_status",
                                serde_json::json!(exit_status.to_string()),
                            );
                        log_info!(
                            finish_context,
                            "Process '{}' finished with exit status: {:?}",
                            process_id,
                            exit_status
                        );

                        managed_process.info.status = if exit_status.success() {
                            ProcessStatus::Stopped
                        } else {
                            ProcessStatus::Failed
                        };

                        let _ = self.event_sender.send(ProcessEvent::Stopped {
                            process_id: process_id.clone(),
                            exit_code: exit_status.code(),
                            workspace: managed_process.info.workspace.clone(),
                        });

                        to_remove.push(process_id.clone());
                    }
                    Ok(None) => {
                        // Process still running
                    }
                    Err(e) => {
                        let error_context = LogContext::new("process", "status_check_error")
                            .with_entity_id(process_id);
                        log_error!(
                            error_context,
                            "Error checking process '{}' status: {}",
                            process_id,
                            e
                        );
                        managed_process.info.status = ProcessStatus::Failed;
                        to_remove.push(process_id.clone());
                    }
                }
            }
        }

        // Remove finished processes
        for process_id in &to_remove {
            if let Some(mut managed_process) = processes.remove(process_id) {
                // Cancel monitors
                if let Some(output_monitor) = managed_process.output_monitor.take() {
                    output_monitor.abort();
                }
                if let Some(health_monitor) = managed_process.health_monitor.take() {
                    health_monitor.abort();
                }
            }
        }

        let cleaned_count = initial_count - processes.len();
        if cleaned_count > 0 {
            let cleanup_context = LogContext::new("process", "cleanup")
                .with_metadata("cleaned_count", serde_json::json!(cleaned_count));
            log_info!(
                cleanup_context,
                "Cleaned up {} finished processes",
                cleaned_count
            );
        }

        cleaned_count
    }

    pub async fn get_process_count(&self) -> usize {
        let processes = self.processes.read().await;
        processes.len()
    }

    pub async fn shutdown_all(&self) {
        let shutdown_context = LogContext::new("process", "shutdown_all");
        log_info!(shutdown_context, "Shutting down all processes");

        let process_ids: Vec<String> = {
            let processes = self.processes.read().await;
            processes.keys().cloned().collect()
        };

        for process_id in process_ids {
            if let Err(e) = self.kill_process(&process_id).await {
                let warn_context =
                    LogContext::new("process", "shutdown_kill_failure").with_entity_id(&process_id);
                log_warn!(
                    warn_context,
                    "Failed to kill process '{}' during shutdown: {}",
                    process_id,
                    e
                );
            }
        }

        // Wait a bit for processes to terminate
        sleep(Duration::from_secs(2)).await;

        // Clean up any remaining processes
        self.cleanup_finished_processes().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ProcessConfig {
        ProcessConfig {
            claude_code_binary: "echo".to_string(), // Use echo for testing
            max_processes: 2,
            health_check_interval_secs: 1,
            restart_delay_secs: 1,
            max_restart_attempts: 1,
            process_timeout_secs: 10,
            default_restart_policy: RestartPolicy::Never,
            environment_vars: HashMap::new(),
            working_directory: None,
        }
    }

    #[tokio::test]
    async fn test_process_manager_creation() {
        let config = create_test_config();
        let (manager, _receiver) = ProcessManager::new(config);

        assert_eq!(manager.get_process_count().await, 0);
    }

    #[tokio::test]
    async fn test_spawn_process() {
        let config = create_test_config();
        let (manager, mut receiver) = ProcessManager::new(config);

        let result = manager
            .spawn_process(
                "test-process".to_string(),
                "test-workspace".to_string(),
                vec!["hello".to_string()],
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(manager.get_process_count().await, 1);

        // Check for started event
        if let Some(event) = receiver.recv().await {
            match event {
                ProcessEvent::Started {
                    process_id,
                    workspace,
                    ..
                } => {
                    assert_eq!(process_id, "test-process");
                    assert_eq!(workspace, "test-workspace");
                }
                _ => panic!("Expected Started event"),
            }
        }
    }

    #[tokio::test]
    async fn test_process_limit() {
        let config = create_test_config();
        let (manager, _receiver) = ProcessManager::new(config);

        // Spawn up to limit
        for i in 0..2 {
            let result = manager
                .spawn_process(
                    format!("test-process-{i}"),
                    "test-workspace".to_string(),
                    vec!["hello".to_string()],
                )
                .await;
            assert!(result.is_ok());
        }

        // Try to exceed limit
        let result = manager
            .spawn_process(
                "test-process-overflow".to_string(),
                "test-workspace".to_string(),
                vec!["hello".to_string()],
            )
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Maximum process limit"));
    }

    #[tokio::test]
    async fn test_duplicate_process_id() {
        let config = create_test_config();
        let (manager, _receiver) = ProcessManager::new(config);

        let result1 = manager
            .spawn_process(
                "test-process".to_string(),
                "test-workspace".to_string(),
                vec!["hello".to_string()],
            )
            .await;
        assert!(result1.is_ok());

        let result2 = manager
            .spawn_process(
                "test-process".to_string(),
                "test-workspace".to_string(),
                vec!["hello".to_string()],
            )
            .await;
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("already exists"));
    }

    #[tokio::test]
    async fn test_get_process_info() {
        let config = create_test_config();
        let (manager, _receiver) = ProcessManager::new(config);

        manager
            .spawn_process(
                "test-process".to_string(),
                "test-workspace".to_string(),
                vec!["hello".to_string()],
            )
            .await
            .unwrap();

        let info = manager.get_process_info("test-process").await;
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.id, "test-process");
        assert_eq!(info.workspace, "test-workspace");
    }

    #[tokio::test]
    async fn test_list_processes() {
        let config = create_test_config();
        let (manager, _receiver) = ProcessManager::new(config);

        manager
            .spawn_process(
                "test-process-1".to_string(),
                "workspace-1".to_string(),
                vec!["hello".to_string()],
            )
            .await
            .unwrap();

        manager
            .spawn_process(
                "test-process-2".to_string(),
                "workspace-2".to_string(),
                vec!["world".to_string()],
            )
            .await
            .unwrap();

        let processes = manager.list_processes().await;
        assert_eq!(processes.len(), 2);

        let workspace_1_processes = manager.get_processes_by_workspace("workspace-1").await;
        assert_eq!(workspace_1_processes.len(), 1);
        assert_eq!(workspace_1_processes[0].id, "test-process-1");
    }
}
