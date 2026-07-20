#![allow(unused_imports)]

use crate::{auth, backup, config, integrations, plugins, receipts, stub};
use crate::ark;
use crate::config::{ConfigLoader, ConfigPersister, ServerConfig, CompositeValidator};
use crate::ark::{build_launch_args, RconClient};
use serde_json::json;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, LazyLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::process::Command;
use tauri::{Manager, Emitter};
use crate::commands::app::PingState;

pub async fn detect_public_ip() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build().ok()?;
    let ip = client.get("https://api4.ipify.org")
        .send().await.ok()?
        .text().await.ok()?
        .trim()
        .to_string();
    // Sanity: non-empty and looks like an IPv4 address (≤15 chars)
    if ip.is_empty() || ip.len() > 15 { None } else { Some(ip) }
}

pub async fn detect_tailscale_ip() -> Option<String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        if let Ok(out) = std::process::Command::new("tailscale")
            .args(["ip", "-4"])
            .creation_flags(0x08000000)
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() && !s.to_lowercase().contains("error") && is_tailscale_range(&s) {
                return Some(s);
            }
        }
    }

    #[cfg(not(windows))]
    {
        if let Ok(out) = std::process::Command::new("tailscale")
            .args(["ip", "-4"])
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() && !s.to_lowercase().contains("error") && is_tailscale_range(&s) {
                return Some(s);
            }
        }
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        if let Ok(out) = std::process::Command::new("ipconfig")
            .creation_flags(0x08000000)
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if line.contains("IPv4") {
                    if let Some(pos) = line.rfind(':') {
                        let ip = line[pos + 1..].trim();
                        if is_tailscale_range(ip) {
                            return Some(ip.to_string());
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(windows))]
    {
        if let Ok(out) = std::process::Command::new("ip")
            .args(["addr", "show"])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if let Some(ip) = line.trim().strip_prefix("inet ") {
                    if let Some(ip) = ip.split('/').next() {
                        if is_tailscale_range(ip) {
                            return Some(ip.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn is_tailscale_range(ip: &str) -> bool {
    // Tailscale uses 100.64.0.0/10 → second octet 64-127
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 { return false; }
    let a: u8 = parts[0].parse().unwrap_or(0);
    let b: u8 = parts[1].parse().unwrap_or(0);
    a == 100 && b >= 64 && b <= 127
}

pub fn detect_local_ip() -> Option<String> {
    // UDP connect trick: the OS selects the right outbound interface without
    // actually sending any packets
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let ip = socket.local_addr().ok()?.ip().to_string();
    if ip.starts_with("127.") { None } else { Some(ip) }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tailscale / ping keep-alive
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn open_external_url(url: String) -> std::result::Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "start", "", url.as_str()]);
        cmd.creation_flags(0x08000000);
        cmd.spawn().map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    #[cfg(not(windows))]
    {
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn start_ping(ip: String, state: tauri::State<'_, PingState>) -> std::result::Result<(), String> {
    // Abort any running ping task first
    if let Some(h) = state.inner().0.lock().map_err(|e| e.to_string())?.take() {
        h.abort();
    }

    let ip = ip.trim().to_string();
    let handle = tokio::spawn(async move {
        loop {
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                let _ = std::process::Command::new("ping")
                    .args(["-n", "4", "-w", "3000", &ip])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .creation_flags(0x08000000)
                    .spawn();
            }
            #[cfg(not(windows))]
            {
                let _ = std::process::Command::new("ping")
                    .args(["-c", "4", "-W", "3", &ip])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn();
            }
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });

    *state.0.lock().map_err(|e| e.to_string())? = Some(handle);
    Ok(())
}

#[tauri::command]
pub fn stop_ping(state: tauri::State<'_, PingState>) -> std::result::Result<(), String> {
    if let Some(h) = state.inner().0.lock().map_err(|e| e.to_string())?.take() {
        h.abort();
    }
    Ok(())
}
