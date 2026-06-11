use async_trait::async_trait;
use crate::error::{Error, Result};
use std::path::Path;
use tokio::process::{Child, Command};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub running: bool,
    pub process_id: Option<u32>,
    pub uptime_seconds: u64,
}

#[async_trait]
pub trait ProcessMgr: Send + Sync {
    async fn start(&self, executable: &str, args: &[&str]) -> Result<u32>;
    async fn stop(&self, process_id: u32) -> Result<()>;
    async fn is_running(&self, process_id: u32) -> Result<bool>;
    async fn get_status(&self) -> Result<ServerStatus>;
}

pub struct ProcessManager {
    process_id: Mutex<Option<u32>>,
    start_time: Mutex<Option<std::time::SystemTime>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            process_id: Mutex::new(None),
            start_time: Mutex::new(None),
        }
    }

    pub async fn kill_process(process_id: u32) -> Result<()> {
        let output = Command::new("taskkill")
            .arg("/PID")
            .arg(process_id.to_string())
            .arg("/F")
            .output()
            .await
            .map_err(|e| Error::ProcessError(format!("Failed to kill process: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::ProcessError(format!("Failed to kill process: {}", stderr)));
        }

        Ok(())
    }

    pub async fn get_child_processes(parent_id: u32) -> Result<Vec<u32>> {
        let output = Command::new("wmic")
            .arg("process")
            .arg("where")
            .arg(format!("ParentProcessId={}", parent_id))
            .arg("get")
            .arg("ProcessId")
            .output()
            .await
            .map_err(|e| Error::ProcessError(format!("Failed to get child processes: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let child_pids: Vec<u32> = stdout
            .lines()
            .skip(1)
            .filter_map(|line| line.trim().parse::<u32>().ok())
            .collect();

        Ok(child_pids)
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProcessMgr for ProcessManager {
    async fn start(&self, executable: &str, args: &[&str]) -> Result<u32> {
        let mut cmd = Command::new(executable);
        for arg in args {
            cmd.arg(arg);
        }

        let child = cmd
            .spawn()
            .map_err(|e| Error::ProcessError(format!("Failed to start process: {}", e)))?;

        let pid = child
            .id()
            .ok_or_else(|| Error::ProcessError("Failed to get process ID".to_string()))?;

        let mut process_id = self.process_id.lock().unwrap();
        *process_id = Some(pid);

        let mut start_time = self.start_time.lock().unwrap();
        *start_time = Some(std::time::SystemTime::now());

        log::info!("Started process {} with PID {}", executable, pid);
        Ok(pid)
    }

    async fn stop(&self, process_id: u32) -> Result<()> {
        // Get child processes and kill them first
        match Self::get_child_processes(process_id).await {
            Ok(children) => {
                for child_id in children {
                    let _ = Self::kill_process(child_id).await;
                }
            }
            Err(e) => log::warn!("Failed to get child processes: {}", e),
        }

        // Kill parent process
        Self::kill_process(process_id).await?;

        let mut pid = self.process_id.lock().unwrap();
        *pid = None;

        let mut start_time = self.start_time.lock().unwrap();
        *start_time = None;

        log::info!("Stopped process {}", process_id);
        Ok(())
    }

    async fn is_running(&self, process_id: u32) -> Result<bool> {
        let output = Command::new("tasklist")
            .arg("/FI")
            .arg(format!("PID eq {}", process_id))
            .output()
            .await
            .map_err(|e| Error::ProcessError(format!("Failed to check process: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains(&process_id.to_string()))
    }

    async fn get_status(&self) -> Result<ServerStatus> {
        let (pid, start) = {
            let process_id = self.process_id.lock().unwrap();
            let start_time = self.start_time.lock().unwrap();
            (*process_id, *start_time)
        };

        match (pid, start) {
            (Some(pid), Some(start)) => {
                let running = self.is_running(pid).await.unwrap_or(false);
                let uptime = if let Ok(elapsed) = start.elapsed() {
                    elapsed.as_secs()
                } else {
                    0
                };

                Ok(ServerStatus {
                    running,
                    process_id: Some(pid),
                    uptime_seconds: uptime,
                })
            }
            _ => Ok(ServerStatus {
                running: false,
                process_id: None,
                uptime_seconds: 0,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_manager_creation() {
        let pm = ProcessManager::new();
        let process_id = pm.process_id.lock().unwrap();
        assert_eq!(*process_id, None);
    }

    #[tokio::test]
    async fn test_get_status_when_empty() {
        let pm = ProcessManager::new();
        let status = pm.get_status().await.unwrap();
        assert!(!status.running);
        assert_eq!(status.process_id, None);
    }
}
