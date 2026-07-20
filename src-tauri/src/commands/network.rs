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

#[derive(serde::Serialize, Clone, Debug)]
pub struct DetectedIps {
    pub public_ip:    Option<String>,
    pub tailscale_ip: Option<String>,
    pub local_ip:     Option<String>,
}

// NLM_CONNECTIVITY values from Windows netlistmgr.h. Local subnet, routed
// network, DNS, and gateway access are not proof of internet reachability.
// Only the explicit IPv4/IPv6 internet flags should unblock ARK startup.
const NLM_IPV4_INTERNET: u32 = 0x40;
const NLM_IPV6_INTERNET: u32 = 0x400;

fn has_internet_connectivity(bits: u32) -> bool {
    bits & (NLM_IPV4_INTERNET | NLM_IPV6_INTERNET) != 0
}

#[tauri::command]
pub async fn check_internet() -> bool {
    // NLM is useful but can lag behind the real network state. Pair it with
    // short HTTPS probes so a stale Windows connectivity bit cannot leave the
    // UI stuck in "offline" while external traffic is working.
    let os_reports_internet = os_reports_internet();
    let http_probe_succeeded = http_probe_succeeded().await;
    let online = internet_is_online(os_reports_internet, http_probe_succeeded);
    log::debug!(
        "internet check: network_list_manager={os_reports_internet}, https_probe={http_probe_succeeded}, online={online}"
    );
    online
}

fn internet_is_online(os_reports_internet: bool, http_probe_succeeded: bool) -> bool {
    os_reports_internet || http_probe_succeeded
}

async fn http_probe_succeeded() -> bool {
    let Ok(client) = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(2))
        .timeout(std::time::Duration::from_secs(3))
        .user_agent("ARK-ASA-Config-Manager connectivity check")
        .build()
    else {
        return false;
    };

    let ipv4_probe = client
        .get("https://www.msftconnecttest.com/connecttest.txt")
        .send();
    let neutral_probe = client
        .get("https://connectivitycheck.gstatic.com/generate_204")
        .send();
    let (ipv4_result, neutral_result) = tokio::join!(ipv4_probe, neutral_probe);

    ipv4_result.is_ok_and(|response| response.status().is_success())
        || neutral_result.is_ok_and(|response| response.status().is_success())
}

#[cfg(windows)]
fn os_reports_internet() -> bool {
    use windows::Win32::Networking::NetworkListManager::INetworkListManager;
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

    unsafe {
        // Tauri commands and the event publisher run on Tokio worker threads.
        // COM is apartment/thread initialised, so initialise it before using
        // NetworkListManager and undo that initialisation when we own it.
        let com_init = CoInitializeEx(None, COINIT_MULTITHREADED);
        let uninitialize_com = com_init.is_ok();
        if com_init.is_err() {
            log::debug!("CoInitializeEx for NetworkListManager returned {com_init:?}");
        }

        let clsid: windows_core::GUID =
            windows_core::GUID::from("DCB00C01-570F-4A9B-8D69-199FDBA5723B");
        let online = match windows::Win32::System::Com::CoCreateInstance(
            &clsid,
            None,
            windows::Win32::System::Com::CLSCTX_INPROC_SERVER,
        ) as windows_core::Result<INetworkListManager> {
            Ok(manager) => manager
                .GetConnectivity()
                .map(|connectivity| has_internet_connectivity(connectivity.0 as u32))
                .unwrap_or(false),
            Err(error) => {
                log::debug!("NetworkListManager unavailable: {error:?}");
                false
            }
        };
        if uninitialize_com {
            CoUninitialize();
        }
        online
    }
}

#[cfg(not(windows))]
fn os_reports_internet() -> bool {
    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn local_or_gateway_connectivity_is_not_internet() {
        assert!(!super::has_internet_connectivity(0x10 | 0x20));
    }

    #[test]
    fn ipv4_or_ipv6_internet_connectivity_is_online() {
        assert!(super::has_internet_connectivity(0x40));
        assert!(super::has_internet_connectivity(0x400));
    }

    #[test]
    fn http_probe_recovers_when_windows_status_is_stale() {
        assert!(super::internet_is_online(false, true));
        assert!(!super::internet_is_online(false, false));
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn reports_active_windows_connection() {
        assert!(
            super::check_internet().await,
            "NetworkListManager reported no connectivity while the test host is online"
        );
    }
}
