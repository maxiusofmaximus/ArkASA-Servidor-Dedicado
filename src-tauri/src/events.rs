//! Live Tauri event publisher.
//!
//! The UI receives an initial snapshot through `invoke` and then subscribes to
//! these topics. The publisher deliberately emits only changed snapshots; the
//! frontend keeps a slow safety fallback for the rare case where the event
//! bridge is interrupted.

use crate::commands::{self, integrations::ServerVersionInfo, server::MapInstanceStatus};
use crate::config::ServerConfig;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

pub const SERVER_STATUS_EVENT: &str = "server://status";
pub const SERVER_LIFECYCLE_EVENT: &str = "server://lifecycle";
pub const INTERNET_STATUS_EVENT: &str = "internet://status";
pub const SERVER_VERSION_EVENT: &str = "server://version";
pub const LOG_APPEND_EVENT: &str = "logs://append";

const OFFLINE_CONFIRMATION_SAMPLES: u8 = 3;

#[derive(Debug, Default)]
struct InternetStability {
    current: Option<bool>,
    offline_streak: u8,
}

impl InternetStability {
    fn observe(&mut self, online: bool) -> Option<bool> {
        if online {
            self.offline_streak = 0;
            if self.current != Some(true) {
                self.current = Some(true);
                return Some(true);
            }
            return None;
        }

        self.offline_streak = self.offline_streak.saturating_add(1);
        if self.offline_streak < OFFLINE_CONFIRMATION_SAMPLES {
            return None;
        }
        if self.current != Some(false) {
            self.current = Some(false);
            return Some(false);
        }
        None
    }

    #[cfg(test)]
    fn current(&self) -> Option<bool> {
        self.current
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ServerStatusEvent {
    pub running: bool,
    pub pid: Option<u32>,
    pub uptime_secs: Option<u64>,
    pub maps: Vec<MapInstanceStatus>,
    pub observed_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ServerLifecycleEvent {
    pub phase: String,
    pub observed_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InternetStatusEvent {
    pub online: bool,
    pub observed_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LogAppendEvent {
    pub server_dir: String,
    pub map: String,
    pub lines: Vec<String>,
    pub tail: Vec<String>,
    pub observed_at_ms: u64,
}

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
static LAST_LIFECYCLE_INTENT: OnceLock<std::sync::Mutex<Option<String>>> = OnceLock::new();

pub fn register_app_handle(app: AppHandle) {
    let _ = APP_HANDLE.set(app);
}

/// Used by start/stop/update commands to make the UI react before the next
/// monitor tick. The publisher will reconcile this optimistic transition with
/// the next real process snapshot.
pub fn emit_lifecycle_phase(phase: &str) {
    if let Ok(mut intent) = LAST_LIFECYCLE_INTENT
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
    {
        *intent = Some(phase.to_string());
    }
    if let Some(app) = APP_HANDLE.get() {
        let _ = app.emit(
            SERVER_LIFECYCLE_EVENT,
            ServerLifecycleEvent {
                phase: phase.to_string(),
                observed_at_ms: now_ms(),
            },
        );
    }
}

pub fn emit_version(info: &ServerVersionInfo) {
    if let Some(app) = APP_HANDLE.get() {
        let _ = app.emit(SERVER_VERSION_EVENT, info);
    }
}

pub fn spawn_event_publisher(app: AppHandle) {
    register_app_handle(app.clone());
    tauri::async_runtime::spawn(async move {
        publisher_loop(app).await;
    });
}

async fn publisher_loop(app: AppHandle) {
    let mut config: Option<ServerConfig> = None;
    let mut config_refresh = tokio::time::interval(Duration::from_secs(5));
    let mut status_tick = tokio::time::interval(Duration::from_secs(2));
    let mut internet_tick = tokio::time::interval(Duration::from_secs(5));
    let mut version_tick = tokio::time::interval(Duration::from_secs(60));
    let mut log_tick = tokio::time::interval(Duration::from_secs(2));

    let mut previous_status: Option<ServerStatusEvent> = None;
    let mut internet_stability = InternetStability::default();
    let mut previous_internet: Option<bool> = None;
    let mut previous_version: Option<ServerVersionInfo> = None;
    let mut previous_logs: HashMap<(String, String), Vec<String>> = HashMap::new();
    let mut running_since: Option<Instant> = None;
    let mut last_status_error = false;
    let mut previous_phase: Option<&'static str> = None;

    loop {
        tokio::select! {
            _ = config_refresh.tick() => {
                config = load_config(&app).await;
            }
            _ = status_tick.tick() => {
                let Some(cfg) = config.clone() else { continue };
                let maps = match commands::server::get_cluster_instance_status(cfg).await {
                    Ok(maps) => {
                        last_status_error = false;
                        maps
                    }
                    Err(_) => {
                        if !last_status_error {
                            emit_lifecycle_phase("error");
                        }
                        last_status_error = true;
                        Vec::new()
                    }
                };
                let process_running = commands::server::is_server_running().await;
                let running = process_running || maps.iter().any(|map| map.running);
                if running && running_since.is_none() {
                    running_since = Some(Instant::now());
                } else if !running {
                    running_since = None;
                }
                let snapshot = ServerStatusEvent {
                    running,
                    pid: if process_running { current_server_pid().await } else { None },
                    uptime_secs: running_since.map(|start| start.elapsed().as_secs()),
                    maps,
                    observed_at_ms: now_ms(),
                };
                let changed = previous_status.as_ref().map_or(true, |old| {
                        old.running != snapshot.running
                        || old.pid != snapshot.pid
                        || old.uptime_secs != snapshot.uptime_secs
                        || old.maps != snapshot.maps
                });
                if changed {
                    let manual_intent = LAST_LIFECYCLE_INTENT
                        .get_or_init(|| std::sync::Mutex::new(None))
                        .lock()
                        .ok()
                        .and_then(|mut intent| intent.take());
                    let phase = lifecycle_phase(
                        previous_status.as_ref().map(|old| old.running),
                        snapshot.running,
                        manual_intent.as_deref(),
                    );
                    emit(&app, SERVER_STATUS_EVENT, &snapshot);
                    if previous_phase != Some(phase) {
                        emit(&app, SERVER_LIFECYCLE_EVENT, &ServerLifecycleEvent {
                            phase: phase.to_string(),
                            observed_at_ms: snapshot.observed_at_ms,
                        });
                        previous_phase = Some(phase);
                    }
                    previous_status = Some(snapshot);
                } else {
                    // Uptime and timestamp are intentionally refreshed only
                    // when the observable state changes, avoiding a noisy UI.
                    previous_status = previous_status.or(Some(snapshot));
                }
            }
            _ = internet_tick.tick() => {
                let sample = commands::network::check_internet().await;
                let Some(online) = internet_stability.observe(sample) else { continue };
                if previous_internet != Some(online) {
                    let event = InternetStatusEvent { online, observed_at_ms: now_ms() };
                    emit(&app, INTERNET_STATUS_EVENT, &event);
                    previous_internet = Some(online);
                }
            }
            _ = version_tick.tick() => {
                let Some(cfg) = config.clone() else { continue };
                if let Ok(info) = commands::integrations::check_server_version(cfg).await {
                    if previous_version.as_ref() != Some(&info) {
                        emit(&app, SERVER_VERSION_EVENT, &info);
                        previous_version = Some(info);
                    }
                }
            }
            _ = log_tick.tick() => {
                let Some(cfg) = config.clone() else { continue };
                publish_log_updates(&app, &cfg, &mut previous_logs).await;
            }
        }
    }
}

fn lifecycle_phase(
    previous_running: Option<bool>,
    running: bool,
    manual_intent: Option<&str>,
) -> &'static str {
    match (previous_running, running) {
        (Some(true), false) if manual_intent != Some("stopping") => "crashed",
        (Some(true), false) => "stopped",
        (_, true) => "running",
        _ => "stopped",
    }
}

async fn load_config(app: &AppHandle) -> Option<ServerConfig> {
    let dir = commands::get_config_dir(app).ok()?;
    let path = dir.join("server-config.toml");
    crate::config::loader::ConfigLoader::load_or_default(&path).await.ok()
}

async fn publish_log_updates(
    app: &AppHandle,
    config: &ServerConfig,
    previous: &mut HashMap<(String, String), Vec<String>>,
) {
    let server_dir = config.paths.server_dir.clone();
    let maps = config.effective_maps();
    for map in maps {
        let tail = match crate::backup::read_server_log(
            server_dir.clone(),
            Some(map.clone()),
            200,
        ).await {
            Ok(lines) => lines,
            Err(_) => continue,
        };
        let key = (server_dir.clone(), map.clone());
        let old_tail = previous.get(&key).cloned().unwrap_or_default();
        let new_lines = append_only_delta(&old_tail, &tail);
        previous.insert(key, tail.clone());
        if new_lines.is_empty() {
            continue;
        }
        emit(app, LOG_APPEND_EVENT, &LogAppendEvent {
            server_dir: server_dir.clone(),
            map,
            lines: new_lines,
            tail,
            observed_at_ms: now_ms(),
        });
    }
}

fn append_only_delta(old: &[String], current: &[String]) -> Vec<String> {
    if old.is_empty() {
        return current.to_vec();
    }
    if current.len() >= old.len() && current[..old.len()] == *old {
        return current[old.len()..].to_vec();
    }
    let overlap = old.iter().enumerate().rev().find_map(|(index, line)| {
        current.iter().position(|candidate| candidate == line)
            .filter(|position| current.len() - *position <= old.len() - index)
            .map(|position| (index, position))
    });
    overlap.map_or_else(|| current.to_vec(), |(_, position)| current[position + 1..].to_vec())
}

async fn current_server_pid() -> Option<u32> {
    tokio::task::spawn_blocking(|| {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let output = std::process::Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq ArkAscendedServer.exe", "/FO", "CSV", "/NH"])
                .creation_flags(0x08000000)
                .output()
                .ok()?;
            let row = String::from_utf8_lossy(&output.stdout);
            let mut fields = row.split('"').filter(|field| !field.trim().is_empty());
            let _image = fields.next()?;
            fields.next()?.trim().parse().ok()
        }
        #[cfg(not(windows))]
        {
            let output = std::process::Command::new("pgrep")
                .args(["-f", "ArkAscendedServer"])
                .output()
                .ok()?;
            String::from_utf8_lossy(&output.stdout).lines().next()?.trim().parse().ok()
        }
    }).await.ok().flatten()
}

fn emit<T: Serialize>(app: &AppHandle, topic: &str, payload: &T) {
    let _ = app.emit(topic, payload);
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::{append_only_delta, lifecycle_phase, InternetStability};

    #[test]
    fn internet_status_ignores_isolated_offline_sample() {
        let mut stability = InternetStability::default();

        assert_eq!(stability.observe(true), Some(true));
        assert_eq!(stability.observe(false), None);
        assert_eq!(stability.observe(true), None);
        assert_eq!(stability.current(), Some(true));
    }

    #[test]
    fn internet_status_requires_three_offline_samples() {
        let mut stability = InternetStability::default();

        assert_eq!(stability.observe(true), Some(true));
        assert_eq!(stability.observe(false), None);
        assert_eq!(stability.observe(false), None);
        assert_eq!(stability.observe(false), Some(false));
        assert_eq!(stability.current(), Some(false));
    }

    #[test]
    fn append_delta_deduplicates_existing_tail() {
        let old = vec!["a".into(), "b".into()];
        let current = vec!["a".into(), "b".into(), "c".into()];
        assert_eq!(append_only_delta(&old, &current), vec!["c"]);
    }

    #[test]
    fn append_delta_resets_after_log_rotation() {
        let old = vec!["a".into(), "b".into()];
        let current = vec!["fresh".into()];
        assert_eq!(append_only_delta(&old, &current), vec!["fresh"]);
    }

    #[test]
    fn lifecycle_distinguishes_manual_stop_from_crash() {
        assert_eq!(lifecycle_phase(Some(true), false, Some("stopping")), "stopped");
        assert_eq!(lifecycle_phase(Some(true), false, None), "crashed");
        assert_eq!(lifecycle_phase(Some(false), true, None), "running");
    }
}
