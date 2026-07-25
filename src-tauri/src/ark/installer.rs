#![allow(dead_code, unused_imports)]

use async_trait::async_trait;
use crate::error::{Error, Result};
use std::path::{Path, PathBuf};
use tokio::process::Command;

#[async_trait]
pub trait Installer: Send + Sync {
    async fn is_installed(&self) -> Result<bool>;
    async fn install(&self) -> Result<()>;
    async fn update(&self) -> Result<()>;
}

pub struct SteamCmdInstaller {
    steam_cmd_dir: PathBuf,
    server_dir: PathBuf,
    app_id: u32,
}

impl SteamCmdInstaller {
    pub fn new(steam_cmd_dir: impl AsRef<Path>, server_dir: impl AsRef<Path>) -> Self {
        Self {
            steam_cmd_dir: steam_cmd_dir.as_ref().to_path_buf(),
            server_dir: server_dir.as_ref().to_path_buf(),
            app_id: 2430930, // ARK Survival Ascended Dedicated Server
        }
    }

    async fn run_steamcmd(&self, args: &[&str]) -> Result<()> {
        #[cfg(target_os = "windows")]
        let steamcmd_exe = self.steam_cmd_dir.join("steamcmd.exe");
        #[cfg(not(target_os = "windows"))]
        let steamcmd_exe = self.steam_cmd_dir.join("steamcmd.sh");

        if !steamcmd_exe.exists() {
            return Err(Error::ProcessError(
                "SteamCMD not found. Please install SteamCMD first.".to_string(),
            ));
        }

        let mut cmd = Command::new(&steamcmd_exe);
        for arg in args {
            cmd.arg(arg);
        }
        // Windows: SteamCMD is a console app. Without `CREATE_NO_WINDOW`
        // (0x08000000) it pops a black CMD box every time we update.
        // Microsoft Learn — Process Creation Flags:
        //   https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
        // On Linux there is no equivalent, so we just inherit stdio.
        // The helper centralises this so P6 (diagnostics::steam_validate
        // repair) and any future subprocess can fly under the same flag.
        super::set_no_window_flag(&mut cmd);

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::ProcessError(format!("Failed to run SteamCMD: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::ProcessError(format!("SteamCMD error: {}", stderr)));
        }

        Ok(())
    }

    async fn ensure_directory(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.server_dir)
            .await
            .map_err(|e| Error::IoError(e))?;
        Ok(())
    }
}

#[async_trait]
impl Installer for SteamCmdInstaller {
    async fn is_installed(&self) -> Result<bool> {
        let binaries_path = self.server_dir.join("ShooterGame").join("Binaries");
        Ok(binaries_path.exists())
    }

    async fn install(&self) -> Result<()> {
        log::info!("Starting ARK server installation...");
        self.ensure_directory().await?;

        let app_id_str = self.app_id.to_string();
        let server_dir_str = self.server_dir.to_string_lossy();

        let args = vec![
            "+@sSteamCmdForcePlatformType windows",
            "+force_install_dir",
            server_dir_str.as_ref(),
            "+login",
            "anonymous",
            "+app_update",
            app_id_str.as_str(),
            "validate",
            "+quit",
        ];

        self.run_steamcmd(&args).await?;
        log::info!("ARK server installation completed");
        Ok(())
    }

    async fn update(&self) -> Result<()> {
        log::info!("Updating ARK server...");
        self.install().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_creation() {
        let installer = SteamCmdInstaller::new("C:\\steamcmd", "C:\\asa_server");
        assert_eq!(installer.app_id, 2430930);
    }

    #[test]
    fn test_paths_are_set() {
        let installer = SteamCmdInstaller::new("C:\\steamcmd", "C:\\asa_server");
        assert_eq!(installer.steam_cmd_dir.to_string_lossy(), "C:\\steamcmd");
        assert_eq!(installer.server_dir.to_string_lossy(), "C:\\asa_server");
    }
}
