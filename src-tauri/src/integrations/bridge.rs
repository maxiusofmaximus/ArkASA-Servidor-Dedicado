//! Bridge layer between adapters (Tauri commands, Telegram bots, HTTP API)
//! and the actual launcher / RCON lifecycle. Returns a normalized
//! [`RouterOutcome`] so every channel speaks the same wire contract.

use crate::ark::{build_launch_args, RconClient};
use crate::config::schema::ServerConfig;
use crate::integrations::command_router::{
    CommandKind, MapDigest, IpDigest, RouterError, RouterOutcome,
};
use std::process::Command;

#[cfg(windows)]
fn kill_all_ark() {
    let _ = std::process::Command::new("taskkill")
        .args(["/F", "/IM", "ArkAscendedServer.exe"])
        .output();
}

#[cfg(not(windows))]
fn kill_all_ark() {
    let _ = std::process::Command::new("pkill")
        .args(["-f", "ArkAscendedServer"])
        .output();
}

/// Start one map instance directly (no cluster delays).
async fn start_instance_inner(config: &ServerConfig, map_index: usize) -> Result<RouterOutcome, RouterError> {
    let maps = config.effective_maps();
    if map_index >= maps.len() {
        return Err(RouterError::Invalid(format!(
            "Invalid map index {} (cluster has {} maps)",
            map_index,
            maps.len()
        )));
    }

    let raw_map = &maps[map_index];
    let map = if raw_map.trim().is_empty() {
        "TheIsland_WP".to_string()
    } else {
        raw_map.trim().to_string()
    };
    let exe = config.paths.ark_exe();
    if exe.is_empty() {
        return Err(RouterError::Internal(
            "[paths] ark_exe is empty".to_string(),
        ));
    }
    let args = build_launch_args(config, &map, map_index);
    let (game_port, _peer_port, _query_port, _rcon_port) = config.network.ports_for_index(map_index);

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

    log::info!("[bridge] launching {} port={}", map, game_port);

    match cmd.spawn() {
        Ok(child) => Ok(RouterOutcome::Started {
            pid: child.id(),
            map: crate::ark::map_label(&map),
        }),
        Err(e) => Err(RouterError::Internal(format!(
            "Failed to start {}: {}. Exe: {}",
            map, e, exe
        ))),
    }
}

/// Stop one map instance via graceful RCON shutdown.
async fn stop_instance_inner(config: &ServerConfig, map_index: usize) -> Result<RouterOutcome, RouterError> {
    let maps = config.effective_maps();
    if map_index >= maps.len() {
        return Err(RouterError::Invalid(format!(
            "Invalid map index {} (cluster has {} maps)",
            map_index,
            maps.len()
        )));
    }

    let map_id = maps[map_index].clone();
    let label = crate::ark::map_label(&map_id);
    let password = &config.identification.admin_password;
    let (_, _, _, rcon_port) = config.network.ports_for_index(map_index);
    let client = RconClient::new(rcon_port, password.as_str());

    match client.graceful_shutdown().await {
        Ok(()) => {
            log::info!("[bridge] RCON shutdown OK for {} port={}", label, rcon_port);
            Ok(RouterOutcome::Stopped { map: label })
        }
        Err(e) => {
            log::warn!("[bridge] RCON shutdown failed for {}: {}", label, e);
            Err(RouterError::Internal(format!("RCON shutdown failed: {}", e)))
        }
    }
}

/// Start the entire cluster (or single map).
async fn start_full_cluster_inner(
    config: &ServerConfig,
    delay_sec: Option<u64>,
) -> Result<RouterOutcome, RouterError> {
    let maps = config.effective_maps();
    let is_cluster = maps.len() > 1;
    let exe = config.paths.ark_exe();
    if exe.is_empty() {
        return Err(RouterError::Internal(
            "[paths] ark_exe is empty".to_string(),
        ));
    }

    let mut launched: Vec<String> = Vec::with_capacity(maps.len());

    for (i, raw_map) in maps.iter().enumerate() {
        let map = if raw_map.trim().is_empty() {
            "TheIsland_WP".to_string()
        } else {
            raw_map.trim().to_string()
        };
        let args = build_launch_args(config, &map, i);
        let (game_port, _peer_port, _query_port, _rcon_port) = config.network.ports_for_index(i);

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

        log::info!("[bridge] launching {} (cluster={}) port={}", map, is_cluster, game_port);

        match cmd.spawn() {
            Ok(child) => launched.push(format!("{} PID {}", map, child.id())),
            Err(e) => {
                return Err(RouterError::Internal(format!(
                    "Failed to start {}: {}. Exe: {}",
                    map, e, exe
                )));
            }
        }

        if is_cluster && i < maps.len() - 1 {
            let delay_ms = delay_sec.unwrap_or(60) * 1000;
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        }
    }

    if is_cluster {
        Ok(RouterOutcome::Error {
            reason: format!(
                "Cluster iniciado [{} instancias]: {}",
                launched.len(),
                launched.join(" | ")
            ),
        })
    } else {
        Ok(RouterOutcome::Error {
            reason: format!("Server started ({})", launched[0]),
        })
    }
}

/// Stop the entire cluster via RCON graceful shutdown.
async fn stop_full_cluster_inner(
    config: &ServerConfig,
) -> Result<RouterOutcome, RouterError> {
    let maps = config.effective_maps();
    let password = &config.identification.admin_password;

    let mut graceful: Vec<String> = vec![];
    let mut failed: Vec<String> = vec![];

    for (i, map) in maps.iter().enumerate() {
        let (_, _, _, rcon_port) = config.network.ports_for_index(i);
        let label = crate::ark::map_key_stem(map);
        let client = RconClient::new(rcon_port, password.as_str());

        match client.graceful_shutdown().await {
            Ok(()) => graceful.push(label.to_string()),
            Err(_) => failed.push(label.to_string()),
        }
    }

    if !failed.is_empty() {
        log::warn!("[bridge] RCON failed for {:?}, falling back to taskkill", failed);
        kill_all_ark();
    }

    let summary = match (graceful.is_empty(), failed.is_empty()) {
        (true, _) => "Servidor detenido (taskkill)".to_string(),
        (_, true) => format!(
            "Servidor detenido correctamente (saveworld + doexit): {}",
            graceful.join(", ")
        ),
        _ => format!(
            "RCON OK: {} | taskkill: {}",
            graceful.join(", "),
            failed.join(", ")
        ),
    };

    Ok(RouterOutcome::Error { reason: summary })
}

/// Lightweight status report.
async fn status_inner(config: &ServerConfig) -> Result<RouterOutcome, RouterError> {
    let maps = config.effective_maps();
    let mut digests = Vec::with_capacity(maps.len());
    let mut any_running = false;

    for (i, map_id) in maps.iter().enumerate() {
        let (_, _, _, rcon_port) = config.network.ports_for_index(i);
        let label = crate::ark::map_label(map_id);
        let running = is_tcp_port_open_quiet(rcon_port);
        if running {
            any_running = true;
        }
        digests.push(MapDigest {
            map_index: i as u32,
            map_id: map_id.clone(),
            map_label: label,
            running,
        });
    }

    Ok(RouterOutcome::Status {
        running: any_running,
        maps: digests,
    })
}

fn is_tcp_port_open_quiet(port: u16) -> bool {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok()
}

/// Top-level dispatcher for a normalized `CommandKind` against a config.
/// Returns a `RouterOutcome` regardless of adapter.
pub async fn dispatch(
    config: &ServerConfig,
    kind: CommandKind,
    map_index: Option<u32>,
    tail: Option<u32>,
) -> Result<RouterOutcome, RouterError> {
    match kind {
        CommandKind::Start => start_full_cluster_inner(config, Some(60)).await,
        CommandKind::Stop => stop_full_cluster_inner(config).await,
        CommandKind::Restart => {
            stop_full_cluster_inner(config).await.ok();
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            start_full_cluster_inner(config, Some(60)).await
        }
        CommandKind::Status => status_inner(config).await,
        CommandKind::Logs => {
            let n = tail.unwrap_or(20).min(500) as usize;
            let p = std::path::PathBuf::from(format!(
                "C:\\ASA\\server\\ShooterGame\\Saved\\Logs\\ShooterGame.log"
            ));
            let reader = crate::ark::logs::LogReader::new(&p);
            let lines = reader.read_last_n(n).await.unwrap_or_default();
            let mut raw: Vec<String> = Vec::with_capacity(lines.len());
            for entry in lines {
                raw.push(format!("{} [{}] {}", entry.timestamp, entry.level, entry.message));
            }
            Ok(RouterOutcome::Logs { lines: raw })
        }
        CommandKind::Ip => {
            use crate::config::schema::ConnectionType;
            let entries = config.network.connection_entries.iter().map(|e| {
                IpDigest {
                    id: e.id.clone(),
                    address: e.address.clone(),
                    primary: e.is_primary
                        || matches!(e.conn_type, ConnectionType::PublicIp)
                        || matches!(e.conn_type, ConnectionType::DuckDns),
                }
            }).collect::<Vec<_>>();
            let primary = entries.iter().find(|e| e.primary).map(|e| e.address.clone());
            Ok(RouterOutcome::Ip { primary, entries })
        }
        CommandKind::ConfigGet => {
            let toml = match toml::to_string(config) {
                Ok(s) => s,
                Err(e) => return Err(RouterError::Internal(format!("toml serialize: {}", e))),
            };
            Ok(RouterOutcome::ConfigGet { toml })
        }
        CommandKind::ConfigSet => Err(RouterError::Invalid(
            "config_set via bot not yet wired (use HTTP API)".into(),
        )),
        CommandKind::StartInstance => {
            if let Some(idx) = map_index {
                start_instance_inner(config, idx as usize).await
            } else {
                Err(RouterError::Invalid("map_index required for StartInstance".into()))
            }
        }
        CommandKind::StopInstance => {
            if let Some(idx) = map_index {
                stop_instance_inner(config, idx as usize).await
            } else {
                Err(RouterError::Invalid("map_index required for StopInstance".into()))
            }
        }
    }
}
