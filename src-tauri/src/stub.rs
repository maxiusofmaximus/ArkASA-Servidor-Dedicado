//! On-demand server stub
//!
//! Holds game_port + query_port so the server appears in the ARK browser
//! even when ARK isn't running ("DORMIDO" = dormant).
//!
//! Lifecycle for one map:
//!   1. Stub binds both ports, responds to A2S_INFO queries.
//!   2. First UDP packet on game_port → stub drops both sockets (releases ports).
//!   3. After a short OS-level delay, ARK is launched.
//!   4. Stub polls query_port until ARK responds → server is ready.
//!   5. Every 60 s stub queries the player count.
//!      If players = 0 for `auto_shutdown_min` minutes → kill ARK → goto 1.
//!   6. If the shutdown_tx fires (user clicked Stop) → kill ARK if running → exit.

use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::watch;
use tokio::time::sleep;

// ARK: Survival Ascended Steam AppID
const ARK_ASA_APP_ID: u32 = 2_399_830;

// ─────────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────────

/// Parameters needed to launch ARK for one map.
#[derive(Clone, Debug)]
pub struct MapLaunchParams {
    /// ARK map id, e.g. "TheIsland_WP"
    pub map: String,
    /// Absolute path to ArkAscendedServer.exe
    pub exe: String,
    /// Full URL/param string passed as first arg to the exe
    pub launch_params: String,
    /// Extra CLI flags ("-NoBattlEye", "-mods=...", etc.)
    pub extra_args: Vec<String>,
    pub game_port: u16,
    pub query_port: u16,
    /// Minutes of zero players before ARK is shut down (0 = never)
    pub auto_shutdown_min: u64,
}

/// Status reported to the frontend via `get_on_demand_status`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StubStatus {
    pub map: String,
    pub state: String,  // "dormant" | "starting" | "running" | "stopped"
    pub game_port: u16,
    pub query_port: u16,
    pub players: u8,
}

// ─────────────────────────────────────────────────────────────────────────────
// Managed state stored in Tauri
// ─────────────────────────────────────────────────────────────────────────────

pub struct OnDemandHandle {
    pub map: String,
    pub game_port: u16,
    pub query_port: u16,
    pub shutdown_tx: watch::Sender<bool>,
    pub task: tokio::task::JoinHandle<()>,
}

pub struct OnDemandState(pub Mutex<Vec<OnDemandHandle>>);

impl OnDemandState {
    pub fn new() -> Self {
        OnDemandState(Mutex::new(vec![]))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// A2S protocol helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Returns true if the datagram is a Steam A2S_INFO query.
fn is_a2s_query(data: &[u8]) -> bool {
    data.len() >= 5
        && data[..4] == [0xFF, 0xFF, 0xFF, 0xFF]
        && data[4] == 0x54 // 'T' = A2S_INFO
}

/// Build a minimal but valid A2S_INFO response packet.
///
/// EDF 0x90 = 0x80 (game port) | 0x10 (full AppID as u64)
fn build_a2s_info(display_name: &str, map: &str, game_port: u16) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(160);

    // ── Fixed header ─────────────────────────────────────────────────────────
    buf.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // header
    buf.push(b'I');  // A2S_INFO response type
    buf.push(17);    // protocol version

    // ── Null-terminated strings ──────────────────────────────────────────────
    buf.extend_from_slice(display_name.as_bytes()); buf.push(0); // server name
    let map_display = map.trim_end_matches("_WP");
    buf.extend_from_slice(map_display.as_bytes()); buf.push(0);  // map name
    buf.extend_from_slice(b"ShooterGame"); buf.push(0);           // game folder
    buf.extend_from_slice(b"ARK: Survival Ascended"); buf.push(0); // game description

    // ── Short AppID (lower 16 bits, LE) ─────────────────────────────────────
    buf.extend_from_slice(&(ARK_ASA_APP_ID as u16).to_le_bytes());

    // ── Server info bytes ────────────────────────────────────────────────────
    buf.push(0);      // current players (0 while dormant)
    buf.push(70);     // max players
    buf.push(0);      // bots
    buf.push(b'd');   // server type: dedicated
    buf.push(b'w');   // environment: Windows
    buf.push(0);      // visibility: public
    buf.push(0);      // VAC: unsecured

    // ── Version string ───────────────────────────────────────────────────────
    buf.extend_from_slice(b"1.0.0.0"); buf.push(0);

    // ── EDF flags + extra data ───────────────────────────────────────────────
    buf.push(0x90); // 0x80 (port) | 0x10 (gameID)
    buf.extend_from_slice(&game_port.to_le_bytes());               // 2 bytes LE
    buf.extend_from_slice(&(ARK_ASA_APP_ID as u64).to_le_bytes()); // 8 bytes LE

    buf
}

// ─────────────────────────────────────────────────────────────────────────────
// ARK process helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Spawn ARK as a detached process.
fn spawn_ark(params: &MapLaunchParams) -> Result<u32, String> {
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    let mut cmd = std::process::Command::new(&params.exe);
    cmd.arg(&params.launch_params);
    for arg in &params.extra_args {
        cmd.arg(arg);
    }
    #[cfg(windows)]
    cmd.creation_flags(0x0000_0008); // DETACHED_PROCESS

    let child = cmd.spawn().map_err(|e| format!("Failed to spawn ARK: {}", e))?;
    let pid = child.id();
    // Don't wait — let it run independently
    std::mem::forget(child);
    Ok(pid)
}

/// Kill ARK by PID (Windows).
fn kill_ark_pid(pid: u32) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
            .spawn();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("kill")
            .args(["-9", &pid.to_string()])
            .spawn();
    }
}

/// Send one A2S_INFO query to `127.0.0.1:query_port` and parse the player count.
/// Returns `None` on timeout or parse failure (ARK not ready / not responding).
async fn query_ark_players(query_port: u16) -> Option<u8> {
    let sock = UdpSocket::bind("0.0.0.0:0").await.ok()?;
    let addr = format!("127.0.0.1:{}", query_port);
    let query = b"\xFF\xFF\xFF\xFF\x54Source Engine Query\x00";
    sock.send_to(query, &addr).await.ok()?;

    let mut buf = [0u8; 1400];
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        sock.recv_from(&mut buf),
    )
    .await;

    let (n, _) = result.ok()?.ok()?;

    // A2S_INFO response: [4×FF][49='I'][protocol][name\0][map\0][folder\0][game\0][appid 2B][players] ...
    if n < 6 || buf[4] != b'I' {
        return None;
    }
    let mut pos = 6usize; // skip 4-byte header + 'I' + protocol byte
    for _ in 0..4 {       // skip 4 null-terminated strings
        while pos < n && buf[pos] != 0 { pos += 1; }
        pos += 1;          // skip the null itself
    }
    pos += 2; // skip 2-byte AppID
    if pos < n { Some(buf[pos]) } else { None }
}

// ─────────────────────────────────────────────────────────────────────────────
// Main stub task
// ─────────────────────────────────────────────────────────────────────────────

/// Run the full on-demand lifecycle for one map.  Loops until shutdown fires.
pub async fn run_stub(params: MapLaunchParams, shutdown: watch::Receiver<bool>) {
    let display_name = {
        // Extract SessionName from launch_params string, append " [DORMIDO]"
        let s = &params.launch_params;
        let start = s.find("SessionName=").map(|i| i + 12).unwrap_or(0);
        let end = s[start..].find('?').map(|i| start + i).unwrap_or(s.len());
        format!("{} [DORMIDO]", &s[start..end])
    };

    'lifecycle: loop {
        // ── Phase 1: Stub dormant — bind ports, serve A2S queries ─────────────
        let query_sock = match UdpSocket::bind(format!("0.0.0.0:{}", params.query_port)).await {
            Ok(s) => s,
            Err(e) => {
                log::warn!("[Stub {}] cannot bind query port {}: {}", params.map, params.query_port, e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        let game_sock = match UdpSocket::bind(format!("0.0.0.0:{}", params.game_port)).await {
            Ok(s) => s,
            Err(e) => {
                log::warn!("[Stub {}] cannot bind game port {}: {}", params.map, params.game_port, e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        let a2s_resp = build_a2s_info(&display_name, &params.map, params.game_port);
        let mut qbuf = [0u8; 2048]; // query socket buffer
        let mut gbuf = [0u8; 2048]; // game socket buffer
        let mut shutdown_rx = shutdown.clone();

        log::info!("[Stub {}] DORMANT (query={} game={})", params.map, params.query_port, params.game_port);

        let player_tried = loop {
            let mut player_connecting = false;

            tokio::select! {
                // Shutdown requested
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        log::info!("[Stub {}] shutdown signal — exiting", params.map);
                        return;
                    }
                }
                // A2S query on query_port → respond
                result = query_sock.recv_from(&mut qbuf) => {
                    if let Ok((n, src)) = result {
                        if is_a2s_query(&qbuf[..n]) {
                            let _ = query_sock.send_to(&a2s_resp, src).await;
                        }
                    }
                }
                // Any UDP packet on game_port → player connecting
                result = game_sock.recv_from(&mut gbuf) => {
                    if let Ok((_n, src)) = result {
                        log::info!("[Stub {}] connection attempt from {} — waking ARK", params.map, src);
                        player_connecting = true;
                    }
                }
            }

            if player_connecting {
                break true;
            }
        };

        if !player_tried {
            return; // shutdown fired inside the loop
        }

        // Drop sockets so the OS releases the ports before ARK binds them
        drop(query_sock);
        drop(game_sock);
        sleep(Duration::from_millis(400)).await;

        // ── Phase 2: Launch ARK ───────────────────────────────────────────────
        let ark_pid = match spawn_ark(&params) {
            Ok(pid) => {
                log::info!("[Stub {}] ARK launched (PID {})", params.map, pid);
                pid
            }
            Err(e) => {
                log::error!("[Stub {}] failed to launch ARK: {}", params.map, e);
                sleep(Duration::from_secs(10)).await;
                continue 'lifecycle;
            }
        };

        // ── Phase 3: Wait for ARK to become ready (up to 10 min) ─────────────
        log::info!("[Stub {}] waiting for ARK query port {}…", params.map, params.query_port);
        let mut ready = false;
        for _ in 0..120u32 {
            let mut sd = shutdown.clone();
            tokio::select! {
                _ = sd.changed() => {
                    if *sd.borrow() {
                        kill_ark_pid(ark_pid);
                        return;
                    }
                }
                _ = sleep(Duration::from_secs(5)) => {}
            }
            if query_ark_players(params.query_port).await.is_some() {
                ready = true;
                break;
            }
        }

        if !ready {
            log::warn!("[Stub {}] ARK did not become ready — restarting stub", params.map);
            kill_ark_pid(ark_pid);
            sleep(Duration::from_secs(5)).await;
            continue 'lifecycle;
        }
        log::info!("[Stub {}] ARK is READY", params.map);

        // ── Phase 4: Monitor player count, auto-shutdown ──────────────────────
        let auto_shutdown_dur = if params.auto_shutdown_min > 0 {
            Some(Duration::from_secs(params.auto_shutdown_min * 60))
        } else {
            None
        };
        let mut empty_since: Option<Instant> = None;

        loop {
            let mut sd = shutdown.clone();
            tokio::select! {
                _ = sd.changed() => {
                    if *sd.borrow() {
                        log::info!("[Stub {}] shutdown — killing ARK (PID {})", params.map, ark_pid);
                        kill_ark_pid(ark_pid);
                        return;
                    }
                }
                _ = sleep(Duration::from_secs(60)) => {}
            }

            match query_ark_players(params.query_port).await {
                None => {
                    // ARK stopped responding (crash or external stop)
                    log::info!("[Stub {}] ARK stopped responding — restarting stub", params.map);
                    continue 'lifecycle;
                }
                Some(0) => {
                    let since = empty_since.get_or_insert_with(Instant::now);
                    if let Some(limit) = auto_shutdown_dur {
                        if since.elapsed() >= limit {
                            log::info!(
                                "[Stub {}] auto-shutdown after {}min empty",
                                params.map,
                                params.auto_shutdown_min
                            );
                            kill_ark_pid(ark_pid);
                            sleep(Duration::from_secs(3)).await;
                            continue 'lifecycle;
                        }
                    }
                }
                Some(_n) => {
                    empty_since = None;
                }
            }
        }
    }
}
