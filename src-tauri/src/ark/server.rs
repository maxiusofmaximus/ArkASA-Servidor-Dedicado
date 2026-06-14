#![allow(dead_code, unused_imports)]

use async_trait::async_trait;
use crate::config::ServerConfig;
use crate::error::{Error, Result};
use crate::ark::process::{ProcessMgr, ServerStatus};
use crate::ark::installer::Installer;
use std::sync::Arc;
use std::path::Path;

#[async_trait]
pub trait ArkServer: Send + Sync {
    async fn install(&self) -> Result<()>;
    async fn update(&self) -> Result<()>;
    async fn start(&self, config: &ServerConfig) -> Result<u32>;
    async fn stop(&self) -> Result<()>;
    async fn restart(&self, config: &ServerConfig) -> Result<()>;
    async fn status(&self) -> Result<ServerStatus>;
}

pub struct ArkServerImpl {
    installer: Arc<dyn Installer>,
    process_mgr: Arc<dyn ProcessMgr>,
}

impl ArkServerImpl {
    pub fn new(installer: Arc<dyn Installer>, process_mgr: Arc<dyn ProcessMgr>) -> Self {
        Self {
            installer,
            process_mgr,
        }
    }

    fn build_server_args(&self, config: &ServerConfig) -> Vec<String> {
        vec![
            format!("-SessionName={}", config.identification.session_name),
            format!("-Port={}", config.network.port),
            format!("-QueryPort={}", config.network.query_port),
            format!("-RCONPort={}", config.network.rcon_port),
            format!("-RCONEnabled=True"),
            "-NoTransferFromFilesystem".to_string(),
            format!("-MaxPlayers={}", config.gameplay.max_players),
            if config.gameplay.server_pve {
                "-ServerPVE".to_string()
            } else {
                "-ServerPvP".to_string()
            },
        ]
    }

    fn get_server_executable(&self, config: &ServerConfig) -> String {
        format!(
            "{}\\ShooterGame\\Binaries\\Win64\\ArkAscendedServer.exe",
            config.paths.server_dir
        )
    }
}

#[async_trait]
impl ArkServer for ArkServerImpl {
    async fn install(&self) -> Result<()> {
        log::info!("Installing ARK server...");
        self.installer.install().await?;
        log::info!("ARK server installed successfully");
        Ok(())
    }

    async fn update(&self) -> Result<()> {
        log::info!("Updating ARK server...");
        self.installer.update().await?;
        log::info!("ARK server updated successfully");
        Ok(())
    }

    async fn start(&self, config: &ServerConfig) -> Result<u32> {
        log::info!("Starting ARK server: {}", config.identification.session_name);

        // Verify installation
        if !self.installer.is_installed().await? {
            return Err(Error::ServerNotFound {
                path: config.paths.server_dir.clone(),
            });
        }

        let executable = self.get_server_executable(config);
        if !Path::new(&executable).exists() {
            return Err(Error::ServerNotFound {
                path: executable.clone(),
            });
        }

        let args = self.build_server_args(config);
        let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let pid = self.process_mgr.start(&executable, &args).await?;
        log::info!("ARK server started with PID {}", pid);
        Ok(pid)
    }

    async fn stop(&self) -> Result<()> {
        let status = self.status().await?;
        if let Some(pid) = status.process_id {
            log::info!("Stopping ARK server (PID: {})", pid);
            self.process_mgr.stop(pid).await?;
            log::info!("ARK server stopped");
        } else {
            log::warn!("ARK server is not running");
        }
        Ok(())
    }

    async fn restart(&self, config: &ServerConfig) -> Result<()> {
        log::info!("Restarting ARK server...");
        self.stop().await?;

        // Wait a bit for graceful shutdown
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        self.start(config).await?;
        log::info!("ARK server restarted");
        Ok(())
    }

    async fn status(&self) -> Result<ServerStatus> {
        self.process_mgr.get_status().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    // Mock implementations for testing would go here
    // Using mockall crate for trait mocking

    #[test]
    fn test_build_server_args() {
        let config = ServerConfig::default();
        let pm = Arc::new(crate::ark::process::ProcessManager::new());
        let installer = Arc::new(TestInstaller);
        let server = ArkServerImpl::new(installer, pm);

        let args = server.build_server_args(&config);
        assert!(args.iter().any(|arg| arg.contains("SessionName")));
        assert!(args.iter().any(|arg| arg.contains("Port")));
        assert!(args.iter().any(|arg| arg.contains("MaxPlayers")));
    }
}

#[cfg(test)]
struct TestInstaller;

#[cfg(test)]
#[async_trait]
impl Installer for TestInstaller {
    async fn is_installed(&self) -> Result<bool> {
        Ok(true)
    }

    async fn install(&self) -> Result<()> {
        Ok(())
    }

    async fn update(&self) -> Result<()> {
        Ok(())
    }
}
