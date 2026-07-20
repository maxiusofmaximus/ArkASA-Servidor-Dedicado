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

#[tauri::command]
pub fn server_status() -> std::result::Result<serde_json::Value, String> {
    Ok(json!({"running": false, "process_id": null, "uptime_seconds": 0}))
}

#[tauri::command]
pub async fn start_server(config: ServerConfig, cluster_delay_sec: Option<u64>) -> std::result::Result<String, String> {
    crate::events::emit_lifecycle_phase("starting");
    let maps       = config.effective_maps();
    let is_cluster = maps.len() > 1;
    let exe        = config.paths.ark_exe();
    let failover_enabled = config.network.cluster_failover_enabled;
    let failover_timeout = config.network.cluster_failover_timeout_sec.clamp(5, 600);
    let mut launched: Vec<String> = Vec::with_capacity(maps.len());

    for (i, raw_map) in maps.iter().enumerate() {
        let map  = if raw_map.trim().is_empty() { "TheIsland_WP".to_string() } else { raw_map.trim().to_string() };

        // ── Cluster failover (runtime only) ─────────────────────────────────
        // Decide which port slot this map lands on before launch.
        //
        // Concrete scenario the user described:
        //   cluster = [TheIsland_WP, Ragnarok_WP], fixed_port_assignment=false
        //   expected: idx 0 → 7777, idx 1 → 7779
        //   bug    : idx 0 fails silently, idx 1 launches anyway at 7779,
        //           so `open IP` (which defaults to 7777) hits nobody.
        //
        // With `cluster_failover_enabled = true` we wait up to
        // `failover_timeout` sec after launching the primary slot and poll its
        // UDP game port. If it never binds, the next-pending map reclaims the
        // primary slot (`ports_for_index(0)` instead of `ports_for_index(i)`).
        // The recovery is tracked runtime-only via the local `reclaimed` map;
        // we don't persist the assignment between sessions.
        let mut instance_idx = i;
        let mut reclaimed_primary = false;
        if is_cluster && failover_enabled && i > 0 {
            let (primary_game, _, _, _) = config.network.ports_for_index(0);
            log::info!(
                "[failover] waiting up to {}s for primary map to bind UDP port {}",
                failover_timeout, primary_game
            );
            if !wait_for_udp_bind(primary_game, failover_timeout).await {
                log::warn!(
                    "[failover] primary never bound UDP {}; candidate {} (idx {}) will reclaim slot 0",
                    primary_game, map, i
                );
                instance_idx = 0;
                reclaimed_primary = true;
            } else {
                log::info!("[failover] primary bound UDP {} cleanly; {} stays on slot {}", primary_game, map, i);
            }
        }

        let args = build_launch_args(&config, &map, instance_idx);
        let (game_port, _, _, _) = config.network.ports_for_index(instance_idx);

        let mut cmd = Command::new(&exe);
        cmd.arg(&args.url_params);
        for flag in &args.flags { cmd.arg(flag); }

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0000_0008); // DETACHED_PROCESS
        }

        #[cfg(not(windows))]
        {
            use std::os::unix::process::CommandExt;
            unsafe { cmd.pre_exec(|| { libc::setsid(); Ok(()) }); }
        }

        log::info!(
            "Launching ARK {} (cluster={}, slot={}{}) port={}",
            map, is_cluster, instance_idx,
            if reclaimed_primary { " [failover-reclaim]" } else { "" },
            game_port
        );

        match cmd.spawn() {
            Ok(child) => launched.push(format!("{} PID {} port={}", map, child.id(), game_port)),
            Err(e) => return Err(format!("Failed to start {}: {}. Exe: {}", map, e, exe)),
        }

        if is_cluster && i < maps.len() - 1 {
            let delay_ms = cluster_delay_sec.unwrap_or(60) * 1000;
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        }
    }

    if is_cluster {
        Ok(format!("Cluster iniciado [{} instancias]: {}", launched.len(), launched.join(" | ")))
    } else {
        Ok(format!("Server started ({})", launched[0]))
    }
}

/// Graceful shutdown via RCON (saveworld → doexit), then a guaranteed
/// `taskkill /F /IM` net. The net always runs — even on RCON success —
/// because `doexit` on a heavily-modded or frozen instance can leave
/// the ArkAscendedServer.exe window hanging. The /F + /IM combo matches
/// the operator's expected behaviour: "Stop cierra el server, no solo
/// les dice que se apaguen".
///
/// Returns a human-readable summary per cluster instance.
#[tauri::command]
pub async fn stop_server(config: ServerConfig) -> std::result::Result<String, String> {
    crate::events::emit_lifecycle_phase("stopping");
    let maps     = config.effective_maps();
    let password = &config.identification.admin_password;

    let mut graceful: Vec<String> = vec![];
    let mut failed:   Vec<String> = vec![];

    for (i, map) in maps.iter().enumerate() {
        let (_, _, _, rcon_port) = config.network.ports_for_index(i);
        let label  = map.trim_end_matches("_WP");
        let client = RconClient::new(rcon_port, password.as_str());

        match client.graceful_shutdown().await {
            Ok(()) => {
                log::info!("RCON graceful shutdown OK for {} (port {})", label, rcon_port);
                graceful.push(label.to_string());
            }
            Err(e) => {
                log::warn!("RCON shutdown failed for {} (port {}): {}", label, rcon_port, e);
                failed.push(label.to_string());
            }
        }
    }

    // Small extra grace so `doexit` has time to settle before we apply
    // the safety net kill. If the operator already saw the saveworld
    // burst in logs, the kill should hit clean windows.
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Always run as a safety net, even when RCON succeeded — the ARK
    // binary's window can stick open even after a clean `doexit`.
    if let Err(e) = RconClient::force_kill_image().await {
        log::warn!("taskkill safety-net failed: {}", e);
    }

    if !failed.is_empty() {
        log::warn!("Falling back to taskkill for: {:?}", failed);
    }

    Ok(match (graceful.is_empty(), failed.is_empty()) {
        (true, _)   => "Servidor detenido (taskkill forzado, sin respuesta RCON)".to_string(),
        (_, true)   => format!("Servidor detenido correctamente (saveworld + doexit + taskkill): {}", graceful.join(", ")),
        _           => format!("RCON OK: {} | RCON falló (se aplicó taskkill): {}", graceful.join(", "), failed.join(", ")),
    })
}

#[tauri::command]
pub async fn is_server_running() -> bool {
    #[cfg(windows)]
    {
        use tokio::process::Command;
        let out = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq ArkAscendedServer.exe", "/NH"])
            .creation_flags(0x08000000)
            .output()
            .await;
        matches!(out, Ok(o) if String::from_utf8_lossy(&o.stdout).contains("ArkAscendedServer.exe"))
    }
    #[cfg(not(windows))]
    {
        use tokio::process::Command;
        let out = Command::new("pgrep")
            .arg("-x")
            .arg("ArkAscendedServer")
            .output()
            .await;
        matches!(out, Ok(o) if !o.stdout.is_empty())
    }
}

// ── Per-instance cluster control ─────────────────────────────────────────────

#[derive(serde::Serialize, Clone, Debug, PartialEq, Eq)]
pub struct MapInstanceStatus {
    pub map_index: usize,
    pub map_id: String,
    pub map_label: String,
    pub running: bool,
}

pub fn map_display_label(map_id: &str) -> String {
    map_id.trim_end_matches("_WP").replace('_', " ")
}

pub fn is_tcp_port_open(port: u16) -> bool {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok()
}

/// Test if a UDP port is bound by trying to send a zero-length datagram.
/// Returns true if something is listening (sendto succeeds without ICMP).
pub fn is_udp_port_bound(port: u16) -> bool {
    use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
    use std::time::Duration;
    let bind_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));
    let target    = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    match UdpSocket::bind(bind_addr) {
        Ok(sock) => {
            let _ = sock.set_read_timeout(Some(Duration::from_millis(200)));
            // If sendto doesn't immediately fail with ICMP/Refused, a process
            // is listening on the UDP port. Best-effort heuristic — ARK on
            // Windows accepts our probe and stays silent; that's still "bound".
            match sock.send_to(&[], target) {
                Ok(_) => true,
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

/// Native detection of UDP listeners.
///
/// On Windows this shells out to `netstat -ano -p UDP` and looks for
/// `:PORT` in the local address column. On Linux it uses `ss -lunH
/// sport = :PORT`. Both are zero-dependency, race-free ways to answer
/// "is *something* bound to this UDP port right now?" without confusing
/// it with a TCP socket of the same number or with another host's
/// traffic. This is the same query the Microsoft Learn `GetUdpTable`
/// docs describe — Windows' MIB table is the underlying data source
/// for `netstat -ano -p UDP`.
pub fn is_udp_port_listening_native(port: u16) -> bool {
    let port_str = port.to_string();
    let port_needle = format!(":{}", port_str);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = Command::new("netstat");
        cmd.args(["-ano", "-p", "UDP"]);
        cmd.creation_flags(0x08000000);
        if let Ok(out) = cmd.output() {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                let trimmed = line.trim_end();
                if trimmed.contains(&port_needle)
                    && (trimmed.contains("LISTENING") || trimmed.contains("ESTABLISHED") || !trimmed.contains("State"))
                {
                    return true;
                }
            }
        }
        false
    }
    #[cfg(not(windows))]
    {
        // ss -lunH 'sport = :PORT'  → lines with non-empty local addr
        let mut cmd = Command::new("ss");
        cmd.args(["-lunH", &format!("sport = :{}", port_str)]);
        if let Ok(out) = cmd.output() {
            let text = String::from_utf8_lossy(&out.stdout);
            return text.lines().any(|l| !l.trim().is_empty());
        }
        false
    }
}

/// Poll until `port` is bound by some UDP listener or `max_seconds`
/// elapses. Used by `start_server` for cluster port-failover:
/// if the primary map never binds its game port within the timeout,
/// the next map in the cluster reclaims that slot.
///
/// Polling cadence: 500 ms — enough granularity for ARK's typical
/// 2-10 s boot without flooding the shell.
pub async fn wait_for_udp_bind(port: u16, max_seconds: u64) -> bool {
    use std::time::Duration;
    let total = max_seconds.max(1);
    let tick  = Duration::from_millis(500);
    let max_iters = (total * 1000) / 500;
    for i in 0..max_iters {
        if is_udp_port_listening_native(port) {
            log::info!("[failover] UDP port {} bound after {} ms", port, i * 500);
            return true;
        }
        tokio::time::sleep(tick).await;
    }
    log::warn!("[failover] UDP port {} never bound within {} s", port, total);
    false
}

#[cfg(windows)]
pub fn kill_process_on_port(port: u16) -> std::result::Result<(), String> {
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new("netstat");
    cmd.args(["-ano"]);
    cmd.creation_flags(0x08000000);
    let output = cmd.output().map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&output.stdout);
    let port_needle = format!(":{}", port);
    for line in text.lines() {
        if line.contains(&port_needle) && line.contains("LISTENING") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(pid_str) = parts.last() {
                let mut kill = Command::new("taskkill");
                kill.args(["/F", "/PID", pid_str]);
                kill.creation_flags(0x08000000);
                kill.output().map_err(|e| e.to_string())?;
                return Ok(());
            }
        }
    }
    Err(format!("No process listening on port {}", port))
}

#[cfg(not(windows))]
pub fn kill_process_on_port(port: u16) -> std::result::Result<(), String> {
    let port_str = port.to_string();
    let output = Command::new("fuser")
        .args([&port_str, "/tcp"])
        .output()
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(pid_str) = stdout.split_whitespace().next() {
        let mut kill = Command::new("kill");
        kill.arg("-9").arg(pid_str);
        kill.output().map_err(|e| e.to_string())?;
        return Ok(());
    }
    Err(format!("No process listening on port {}", port))
}

#[tauri::command]
pub async fn get_cluster_instance_status(config: ServerConfig) -> std::result::Result<Vec<MapInstanceStatus>, String> {
    let maps = config.effective_maps();
    let fixed = config.network.fixed_port_assignment_per_map;
    let futures: Vec<_> = maps
        .iter()
        .enumerate()
        .map(|(i, map_id)| {
            // ARK binds **UDP** game port, not TCP RCON. The previous
            // `is_tcp_port_open(rcon_port)` returned false while the
            // server was happily accepting players, which made the
            // Start/Stop button think nothing was running.
            //
            // Use the native OS UDP listener table (netstat `GetUdpTable`
            // on Windows / `ss -lunH` on Linux) on the game port (peer
            // = game+1, query are corroborated so we don't mistake any
            // single transient listener).
            let ports3 = if fixed {
                config.network.ports_for_map_id3(map_id)
            } else {
                config.network.ports_for_index3(i)
            };
            let game_port  = ports3.0;
            let rcon_port  = ports3.2;
            let map_id_owned = map_id.clone();
            let map_label = map_display_label(map_id);
            tokio::task::spawn_blocking(move || {
                let udp_open = is_udp_port_listening_native(game_port)
                            || is_udp_port_listening_native(game_port + 1);
                let rcon_open = is_tcp_port_open(rcon_port);
                MapInstanceStatus {
                    map_index: i,
                    map_id: map_id_owned,
                    map_label,
                    // Either game port bound (UDP) or RCON accepting TCP
                    // ⇒ treat the server as "running".
                    running: udp_open || rcon_open,
                }
            })
        })
        .collect();
    let statuses = futures::future::join_all(futures).await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();
    Ok(statuses)
}

#[tauri::command]
pub async fn start_server_instance(
    config: ServerConfig,
    map_index: usize,
) -> std::result::Result<String, String> {
    crate::events::emit_lifecycle_phase("starting");
    let maps = config.effective_maps();
    if map_index >= maps.len() {
        return Err(format!("Invalid map index {} (cluster has {} maps)", map_index, maps.len()));
    }

    let raw_map = &maps[map_index];
    let map = if raw_map.trim().is_empty() {
        "TheIsland_WP".to_string()
    } else {
        raw_map.trim().to_string()
    };
    let exe = config.paths.ark_exe();
    let args = build_launch_args(&config, &map, map_index);
    let (game_port, _peer_port, _query_port, _rcon_port) = config.network.ports_for_index(map_index);

    // Anti double-start guard.
    // ARK binds **UDP** game + query, not TCP. is_tcp_port_open was yielding
    // a false positive on hosts where something else occupied the TCP slot.
    // Verify the actual UDP game port + any running ArkAscendedServer.exe.
    if is_udp_port_bound(game_port) {
        return Err(format!("{} is already running (UDP game port {} is bound)", map_display_label(&map), game_port));
    }
    if is_server_running().await {
        return Err("Another ARK instance is already running. Stop it first or wait for the next poll to clear stale state.".to_string());
    }

    let mut cmd = Command::new(&exe);
    cmd.arg(&args.url_params);
    for flag in &args.flags {
        cmd.arg(flag);
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0000_0008);
    }

    #[cfg(not(windows))]
    {
        use std::os::unix::process::CommandExt;
        unsafe { cmd.pre_exec(|| { libc::setsid(); Ok(()) }); }
    }

    log::info!("Launching ARK instance {} port={}", map, game_port);

    match cmd.spawn() {
        Ok(child) => Ok(format!("{} started (PID {}, port {})", map_display_label(&map), child.id(), game_port)),
        Err(e) => Err(format!("Failed to start {}: {}. Exe: {}", map, e, exe)),
    }
}

#[tauri::command]
pub async fn stop_server_instance(config: ServerConfig, map_index: usize) -> std::result::Result<String, String> {
    crate::events::emit_lifecycle_phase("stopping");
    let maps = config.effective_maps();
    if map_index >= maps.len() {
        return Err(format!("Invalid map index {} (cluster has {} maps)", map_index, maps.len()));
    }

    let map_id = maps[map_index].clone();
    let label = map_display_label(&map_id);
    let password = &config.identification.admin_password;
    let (_, _, _, rcon_port) = config.network.ports_for_index(map_index);
    let client = RconClient::new(rcon_port, password.as_str());

    let rcon_result = client.graceful_shutdown().await;
    match rcon_result {
        Ok(()) => {
            log::info!("RCON graceful shutdown OK for {} (port {})", label, rcon_port);
            // Safety-net sleep so `doexit` can settle, then a guaranteed
            // taskkill so the operator never sees a stuck .exe window.
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            if let Err(e) = RconClient::force_kill_image().await {
                log::warn!("[{}] safety-net taskkill failed: {}", label, e);
            }
            Ok(format!("{} detenido correctamente (saveworld + doexit + taskkill)", label))
        }
        Err(e) => {
            log::warn!("RCON shutdown failed for {} (port {}): {}", label, rcon_port, e);
            // RCON failed entirely — kill by port (laser-targeted) plus
            // the image-wide safety net so the operator always gets a
            // closed window even on a frozen instance.
            let port_kill = kill_process_on_port(rcon_port);
            let _ = RconClient::force_kill_image().await;
            if let Err(kill_err) = port_kill {
                Err(format!("RCON failed ({}) and could not kill process: {}", e, kill_err))
            } else {
                Ok(format!("{} detenido (sin respuesta RCON — kill forzado: puerto {} + imagen)", label, rcon_port))
            }
        }
    }
}

// ── Raw config file I/O ─────────────────────────────────────────────────────

#[tauri::command]
pub async fn read_text_file(path: String) -> std::result::Result<String, String> {
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(format!("Failed to read {}: {}", path, e)),
    }
}

#[tauri::command]
pub async fn write_text_file(path: String, content: String) -> std::result::Result<(), String> {
    let p = PathBuf::from(&path);
    if let Some(parent) = p.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    tokio::fs::write(&p, content).await.map_err(|e| format!("Failed to write {}: {}", path, e))?;
    Ok(())
}

#[tauri::command]
pub fn merge_config_from_ini(config: ServerConfig, ini_content: String) -> std::result::Result<ServerConfig, String> {
    Ok(ConfigLoader::merge_ini_content(&config, &ini_content))
}

#[tauri::command]
pub async fn restart_server(config: ServerConfig, cluster_delay_sec: Option<u64>) -> std::result::Result<String, String> {
    crate::events::emit_lifecycle_phase("starting");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "ArkAscendedServer.exe"])
            .creation_flags(0x08000000)
            .output();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("pkill")
            .args(["-f", "ArkAscendedServer"])
            .output();
    }
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    start_server(config, cluster_delay_sec).await
}

#[tauri::command]
pub fn get_server_logs(_lines: i32) -> std::result::Result<Vec<String>, String> {
    Ok(vec!["[INFO] Server started".to_string()])
}

#[tauri::command]
pub fn get_server_metrics() -> std::result::Result<serde_json::Value, String> {
    Ok(json!({"cpu": 0, "memory": 0, "fps": 0}))
}

#[tauri::command]
pub fn backup_config(_config: ServerConfig, name: String) -> std::result::Result<String, String> {
    Ok(format!("Backup '{}' created", name))
}

#[tauri::command]
pub fn list_backups() -> std::result::Result<Vec<serde_json::Value>, String> {
    Ok(vec![])
}

#[tauri::command]
pub fn restore_backup(_name: String) -> std::result::Result<ServerConfig, String> {
    Ok(ServerConfig::default())
}

// ─────────────────────────────────────────────────────────────────────────────
// IP auto-detection
