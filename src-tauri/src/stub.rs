//! On-demand server stub
//!
//! Holds game_port + query_port so the server appears in the ARK browser
//! even when ARK isn't running ("DORMIDO" = dormant).
//!
//! Lifecycle for one map:
//!   1. Stub binds both ports, responds to A2S_INFO queries.
//!   2. First UDP packet on game_port → drop game_sock, launch ARK.
//!   3. While ARK loads (takes 3-5 min), stub continues serving A2S on
//!      query_port in burst-bind windows so the obelisk keeps seeing the server.
//!   4. Once ARK is fully running (RCON accepts connections), drop query_sock.
//!   5. Every 60 s monitor player count.
//!      If players = 0 for `auto_shutdown_min` minutes → RCON saveworld+doexit → goto 1.
//!   6. Shutdown signal fires → RCON saveworld+doexit (falls back to kill if RCON fails).

use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::Emitter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::watch;
use tokio::time::sleep;

// ARK: Survival Ascended Steam AppID
const ARK_ASA_APP_ID: u32 = 2_399_830;

/// Timeout constants for RCON in the stub (independent of ark/rcon.rs).
const RCON_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const RCON_AUTH_TIMEOUT:    Duration = Duration::from_secs(4);
const RCON_CMD_TIMEOUT:     Duration = Duration::from_secs(8);
const SAVE_GRACE_SECS:      u64      = 4;

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
    pub rcon_port: u16,
    pub admin_password: String,
    /// Minutes of zero players before ARK is shut down (0 = never)
    pub auto_shutdown_min: u64,
    /// Optional Tauri AppHandle for emitting lifecycle events to the frontend.
    pub app: Option<tauri::AppHandle>,
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
/// EDF 0x81 = 0x80 (game port) | 0x01 (full 64-bit GameID/AppID)
fn build_a2s_info(display_name: &str, map: &str, game_port: u16) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(160);

    // ── Fixed header ─────────────────────────────────────────────────────────
    buf.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // header
    buf.push(b'I');  // A2S_INFO response type
    buf.push(17);    // protocol version

    // ── Null-terminated strings ──────────────────────────────────────────────
    buf.extend_from_slice(display_name.as_bytes()); buf.push(0); // server name
    let map_display = crate::ark::map_key_stem(map);
    buf.extend_from_slice(map_display.as_bytes()); buf.push(0);  // map name
    buf.extend_from_slice(b"ShooterGame"); buf.push(0);           // game folder
    buf.extend_from_slice(b"ARK: Survival Ascended"); buf.push(0); // game description

    // ── Short AppID field (2 bytes, legacy) ─────────────────────────────────
    // ARK ASA AppID (2_399_830) exceeds u16::MAX (65_535). Sending the
    // truncated value causes Steam to reject the connection with "invalid
    // application ID". Use 0 here and rely on the full 8-byte GameID in the
    // EDF section below — which is what Steam actually uses for app lookup.
    buf.extend_from_slice(&0u16.to_le_bytes());

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
    // Valve EDF bits (A2S_INFO spec):
    //   0x80 = game port (2 bytes)
    //   0x10 = server's Steam ID (8 bytes)  ← NOT the game AppID
    //   0x01 = full 64-bit GameID / AppID   ← this is what we want
    // EDF = 0x81 = 0x80 (port) | 0x01 (GameID)
    buf.push(0x81);
    buf.extend_from_slice(&game_port.to_le_bytes());               // 2 bytes: game port
    buf.extend_from_slice(&(ARK_ASA_APP_ID as u64).to_le_bytes()); // 8 bytes: full AppID

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

/// Kill ARK by PID.
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

// ─────────────────────────────────────────────────────────────────────────────
// RCON helpers (standalone — no dependency on ark/rcon.rs)
// ─────────────────────────────────────────────────────────────────────────────

/// Build a Source RCON framed packet.
fn rcon_packet(id: i32, ptype: i32, body: &str) -> Vec<u8> {
    let body_bytes = body.as_bytes();
    let size = 4 + 4 + body_bytes.len() + 2;
    let mut pkt = Vec::with_capacity(4 + size);
    pkt.extend_from_slice(&(size as i32).to_le_bytes());
    pkt.extend_from_slice(&id.to_le_bytes());
    pkt.extend_from_slice(&ptype.to_le_bytes());
    pkt.extend_from_slice(body_bytes);
    pkt.push(0);
    pkt.push(0);
    pkt
}

/// Send one RCON command. Returns Err if auth fails or connection times out.
async fn rcon_exec(rcon_port: u16, password: &str, command: &str) -> Result<(), String> {
    let addr = format!("127.0.0.1:{}", rcon_port);
    let mut stream = tokio::time::timeout(RCON_CONNECT_TIMEOUT, TcpStream::connect(&addr))
        .await
        .map_err(|_| format!("RCON connect timeout (port {})", rcon_port))?
        .map_err(|e| format!("RCON connect failed: {}", e))?;

    // Authenticate
    stream.write_all(&rcon_packet(1, 3, password)).await.map_err(|e| e.to_string())?;
    let mut buf = [0u8; 4096];
    let _ = tokio::time::timeout(RCON_AUTH_TIMEOUT, stream.read(&mut buf))
        .await
        .map_err(|_| "RCON auth timeout")?
        .map_err(|e| e.to_string())?;

    if buf.len() >= 8 {
        let resp_id = i32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        if resp_id == -1 {
            return Err("RCON auth rejected".to_string());
        }
    }

    // Send command
    stream.write_all(&rcon_packet(2, 2, command)).await.map_err(|e| e.to_string())?;
    let _ = tokio::time::timeout(RCON_CMD_TIMEOUT, stream.read(&mut buf))
        .await
        .unwrap_or(Ok(0));

    Ok(())
}

/// Graceful shutdown: saveworld → wait → doexit.
/// Falls back silently — caller should kill_ark_pid if this returns Err.
async fn rcon_graceful_shutdown(rcon_port: u16, password: &str) -> Result<(), String> {
    rcon_exec(rcon_port, password, "saveworld").await?;
    sleep(Duration::from_secs(SAVE_GRACE_SECS)).await;
    // doexit causes the server to close the TCP connection before responding — that's OK
    let _ = rcon_exec(rcon_port, password, "doexit").await;
    Ok(())
}

/// Returns true if ARK's RCON port is accepting TCP connections (ARK fully loaded).
async fn is_rcon_accepting(rcon_port: u16) -> bool {
    tokio::time::timeout(
        Duration::from_secs(2),
        TcpStream::connect(format!("127.0.0.1:{}", rcon_port)),
    )
    .await
    .is_ok_and(|r| r.is_ok())
}

/// Send one A2S_INFO query to `127.0.0.1:query_port` and parse the player count.
/// Returns `None` on timeout or parse failure.
async fn query_ark_players(query_port: u16) -> Option<u8> {
    let sock = UdpSocket::bind("0.0.0.0:0").await.ok()?;
    let addr = format!("127.0.0.1:{}", query_port);
    let query = b"\xFF\xFF\xFF\xFF\x54Source Engine Query\x00";
    sock.send_to(query, &addr).await.ok()?;

    let mut buf = [0u8; 1400];
    let result = tokio::time::timeout(Duration::from_secs(2), sock.recv_from(&mut buf)).await;
    let (n, _) = result.ok()?.ok()?;

    if n < 6 || buf[4] != b'I' { return None; }
    let mut pos = 6usize;
    for _ in 0..4 {
        while pos < n && buf[pos] != 0 { pos += 1; }
        pos += 1;
    }
    pos += 2; // skip 2-byte AppID
    if pos < n { Some(buf[pos]) } else { None }
}

// ─────────────────────────────────────────────────────────────────────────────
// Main stub task
// ─────────────────────────────────────────────────────────────────────────────

/// Emit a Tauri event if an AppHandle was provided.
fn emit_event(app: &Option<tauri::AppHandle>, event: &str, payload: &str) {
    if let Some(handle) = app {
        let _ = handle.emit(event, payload.to_string());
    }
}

/// Run the full on-demand lifecycle for one map. Loops until shutdown fires.
pub async fn run_stub(params: MapLaunchParams, shutdown: watch::Receiver<bool>) {
    // Extract the session name from the launch_params URL string.
    let base_name = {
        let s = &params.launch_params;
        let start = s.find("SessionName=").map(|i| i + 12).unwrap_or(0);
        let end = s[start..].find('?').map(|i| start + i).unwrap_or(s.len());
        s[start..end].to_string()
    };
    // Pre-build A2S responses for each lifecycle state.
    let a2s_dormant  = build_a2s_info(&format!("{} [DORMIDO]",   base_name), &params.map, params.game_port);
    let a2s_starting = build_a2s_info(&format!("{} [INICIANDO]", base_name), &params.map, params.game_port);

    'lifecycle: loop {
        // ── Phase 1: Stub dormant — bind both ports, serve A2S queries ─────────
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

        let mut qbuf = [0u8; 2048];
        let mut gbuf = [0u8; 2048];

        log::info!("[Stub {}] DORMANT (query={} game={})", params.map, params.query_port, params.game_port);

        // Phase 1 dormant loop: serve A2S on query_port, watch for game_port trigger.
        // Check shutdown level at top of each iteration (watch::Receiver::changed()
        // is edge-triggered, so we use borrow() for level-triggered check).
        loop {
            if *shutdown.borrow() {
                log::info!("[Stub {}] shutdown signal while dormant", params.map);
                return;
            }
            tokio::select! {
                result = query_sock.recv_from(&mut qbuf) => {
                    if let Ok((n, src)) = result {
                        if is_a2s_query(&qbuf[..n]) {
                            let _ = query_sock.send_to(&a2s_dormant, src).await;
                        }
                    }
                }
                result = game_sock.recv_from(&mut gbuf) => {
                    if let Ok((_n, src)) = result {
                        log::info!("[Stub {}] connection attempt from {} — waking ARK", params.map, src);
                        break;
                    }
                }
                // Wake up every 2s to recheck shutdown level
                _ = sleep(Duration::from_secs(2)) => {}
            }
        }

        // Release ONLY game_sock so the OS gives game_port to ARK.
        // query_sock stays so we can keep serving A2S while ARK loads.
        // Notify the frontend that this map is waking up.
        emit_event(&params.app, "on-demand-waking", &params.map);
        drop(game_sock);
        sleep(Duration::from_millis(200)).await;

        // ── Phase 2: Launch ARK ───────────────────────────────────────────────
        let ark_pid = match spawn_ark(&params) {
            Ok(pid) => {
                log::info!("[Stub {}] ARK launched (PID {})", params.map, pid);
                pid
            }
            Err(e) => {
                log::error!("[Stub {}] failed to launch ARK: {}", params.map, e);
                drop(query_sock);
                sleep(Duration::from_secs(10)).await;
                continue 'lifecycle;
            }
        };

        // ── Phase 3: Keep serving A2S on query_port while ARK starts ──────────
        //
        // ARK will try to bind query_port shortly after start. The moment it
        // does, our bind fails and we yield that port. We detect ARK fully ready
        // via RCON (which becomes available only after the map finishes loading),
        // then stop trying to rebind query_port.
        //
        // Strategy: hold query_sock open, serve A2S until ARK tries to bind
        // query_port (our recv will fail or sock will be taken). Once ARK is
        // responding via RCON, we're done.
        //
        // Since ARK on Windows might fail to bind our held port and keep
        // retrying, we flip between "hold + serve" and "yield" in 5-second
        // windows so ARK can grab the port when it's ready.
        //
        log::info!("[Stub {}] serving A2S on query_port {} while ARK loads…", params.map, params.query_port);

        // Phase 3a: Hold query_sock and keep serving A2S for up to 60s.
        // ARK typically doesn't try to bind query_port until after ~30-60s of load.
        // We check every 10s if ARK's RCON port is accepting (= fully loaded).
        let mut ark_ready = false;
        let hold_until = tokio::time::Instant::now() + Duration::from_secs(60);
        let mut tbuf = [0u8; 2048];

        'hold: loop {
            if *shutdown.borrow() {
                log::info!("[Stub {}] shutdown while starting — RCON then kill", params.map);
                drop(query_sock);
                let _ = rcon_graceful_shutdown(params.rcon_port, &params.admin_password).await;
                kill_ark_pid(ark_pid);
                return;
            }
            if tokio::time::Instant::now() >= hold_until { break 'hold; }

            tokio::select! {
                result = query_sock.recv_from(&mut tbuf) => {
                    if let Ok((n, src)) = result {
                        if is_a2s_query(&tbuf[..n]) {
                            // Show INICIANDO so players know to wait and reconnect
                            let _ = query_sock.send_to(&a2s_starting, src).await;
                        }
                    }
                }
                _ = sleep(Duration::from_secs(10)) => {
                    // Check if ARK RCON is up (= map fully loaded)
                    if is_rcon_accepting(params.rcon_port).await {
                        log::info!("[Stub {}] ARK RCON up — releasing query_port", params.map);
                        ark_ready = true;
                        break 'hold;
                    }
                }
            }
        }

        // Release query_sock so ARK can own it (if it hasn't already taken it)
        drop(query_sock);

        if ark_ready {
            log::info!("[Stub {}] ARK is READY", params.map);
            emit_event(&params.app, "on-demand-ready", &params.map);
        } else {
            // Phase 3b: burst-bind windows.
            // Alternate between "hold query_port and serve A2S" and "yield it to ARK",
            // so the server keeps appearing in the obelisk/browser during the long load.
            log::info!("[Stub {}] burst-bind phase for query_port {}", params.map, params.query_port);
            let mut burst_iter = 0u32;

            while burst_iter < 120 && !ark_ready {
                if *shutdown.borrow() {
                    log::info!("[Stub {}] shutdown in burst phase — RCON then kill", params.map);
                    let _ = rcon_graceful_shutdown(params.rcon_port, &params.admin_password).await;
                    kill_ark_pid(ark_pid);
                    return;
                }

                // Try to grab query_port for a 4-second serving window
                match UdpSocket::bind(format!("0.0.0.0:{}", params.query_port)).await {
                    Ok(temp_sock) => {
                        // Got it — ARK hasn't bound query_port yet. Serve A2S briefly.
                        let end = tokio::time::Instant::now() + Duration::from_secs(4);
                        loop {
                            let remaining = end.saturating_duration_since(tokio::time::Instant::now());
                            if remaining.is_zero() { break; }
                            let mut bbuf = [0u8; 2048];
                            match tokio::time::timeout(remaining, temp_sock.recv_from(&mut bbuf)).await {
                                Ok(Ok((n, src))) => {
                                    if is_a2s_query(&bbuf[..n]) {
                                        // Show INICIANDO so players know to wait and reconnect
                                        let _ = temp_sock.send_to(&a2s_starting, &src).await;
                                    }
                                }
                                _ => break,
                            }
                        }
                        drop(temp_sock); // Release so ARK can grab it
                        sleep(Duration::from_millis(500)).await; // brief gap for ARK
                    }
                    Err(_) => {
                        // ARK has query_port — check if it's actually responding (RCON up)
                        if is_rcon_accepting(params.rcon_port).await {
                            log::info!("[Stub {}] ARK RCON up in burst phase — ready", params.map);
                            emit_event(&params.app, "on-demand-ready", &params.map);
                            ark_ready = true;
                        } else {
                            sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
                burst_iter += 1;
            }

            if !ark_ready {
                log::warn!("[Stub {}] ARK did not become ready in time — restarting stub", params.map);
                kill_ark_pid(ark_pid);
                sleep(Duration::from_secs(5)).await;
                continue 'lifecycle;
            }
            log::info!("[Stub {}] ARK is READY", params.map);
        }

        // ── Phase 4: Monitor player count, auto-shutdown ──────────────────────
        let auto_shutdown_dur = if params.auto_shutdown_min > 0 {
            Some(Duration::from_secs(params.auto_shutdown_min * 60))
        } else {
            None
        };
        let mut empty_since: Option<Instant> = None;

        loop {
            // Level-triggered shutdown check — catches signals sent before we entered this loop
            if *shutdown.borrow() {
                log::info!("[Stub {}] shutdown signal — graceful RCON shutdown", params.map);
                if let Err(e) = rcon_graceful_shutdown(params.rcon_port, &params.admin_password).await {
                    log::warn!("[Stub {}] RCON graceful shutdown failed ({}), killing PID {}", params.map, e, ark_pid);
                    kill_ark_pid(ark_pid);
                } else {
                    log::info!("[Stub {}] RCON graceful shutdown complete", params.map);
                }
                return;
            }

            sleep(Duration::from_secs(60)).await;

            match query_ark_players(params.query_port).await {
                None => {
                    log::info!("[Stub {}] ARK stopped responding — restarting stub", params.map);
                    continue 'lifecycle;
                }
                Some(0) => {
                    let since = empty_since.get_or_insert_with(Instant::now);
                    if let Some(limit) = auto_shutdown_dur {
                        if since.elapsed() >= limit {
                            log::info!(
                                "[Stub {}] auto-shutdown after {}min empty — RCON graceful",
                                params.map, params.auto_shutdown_min
                            );
                            if let Err(e) = rcon_graceful_shutdown(params.rcon_port, &params.admin_password).await {
                                log::warn!("[Stub {}] RCON auto-shutdown failed ({}), killing", params.map, e);
                                kill_ark_pid(ark_pid);
                            }
                            sleep(Duration::from_secs(5)).await;
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
