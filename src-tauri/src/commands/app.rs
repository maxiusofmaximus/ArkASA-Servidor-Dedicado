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


pub struct PingState(pub Mutex<Option<tokio::task::JoinHandle<()>>>);

// ─────────────────────────────────────────────────────────────────────────────
// Tray state
// ─────────────────────────────────────────────────────────────────────────────

pub struct TrayState {
    pub minimize_to_tray: AtomicBool,
}

#[tauri::command]
pub fn set_minimize_to_tray(state: tauri::State<Arc<TrayState>>, enabled: bool) {
    state.minimize_to_tray.store(enabled, Ordering::SeqCst);
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

// ─────────────────────────────────────────────────────────────────────────────
// On-demand server commands
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn enable_on_demand(
    config: ServerConfig,
    map_index: usize,
    auto_shutdown_min: u64,
    state: tauri::State<'_, Arc<stub::OnDemandState>>,
    app: tauri::AppHandle,
) -> std::result::Result<String, String> {
    let maps = config.effective_maps();
    let map  = maps.get(map_index)
        .cloned()
        .unwrap_or_else(|| "TheIsland_WP".to_string());

    let (game_port, _peer_port, query_port, rcon_port) = config.network.ports_for_index(map_index);
    let args = build_launch_args(&config, &map, map_index);

    let params = stub::MapLaunchParams {
        map:            map.clone(),
        exe:            config.paths.ark_exe(),
        launch_params:  args.url_params,
        extra_args:     args.flags,
        game_port,
        query_port,
        rcon_port,
        admin_password: config.identification.admin_password.clone(),
        auto_shutdown_min,
        app:            Some(app),
    };

    // Replace any existing stub on the same port
    {
        let mut handles = state.0.lock().map_err(|e| e.to_string())?;
        if let Some(pos) = handles.iter().position(|h| h.game_port == game_port) {
            let old = handles.remove(pos);
            let _ = old.shutdown_tx.send(true);
            old.task.abort();
        }
    }

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let task = tokio::spawn(stub::run_stub(params, shutdown_rx));

    state.0.lock().map_err(|e| e.to_string())?.push(stub::OnDemandHandle {
        map: map.clone(),
        game_port,
        query_port,
        shutdown_tx,
        task,
    });

    Ok(format!("Stub started for {} (game={} query={})", map, game_port, query_port))
}

#[tauri::command]
pub async fn disable_on_demand(
    game_port: u16,
    state: tauri::State<'_, Arc<stub::OnDemandState>>,
) -> std::result::Result<(), String> {
    let mut handles = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(pos) = handles.iter().position(|h| h.game_port == game_port) {
        let handle = handles.remove(pos);
        // Signal the task — it will RCON saveworld+doexit before exiting
        let _ = handle.shutdown_tx.send(true);
        log::info!("On-demand stub graceful shutdown signaled for game_port={}", game_port);
    }
    Ok(())
}

#[tauri::command]
pub async fn disable_all_on_demand(
    state: tauri::State<'_, Arc<stub::OnDemandState>>,
) -> std::result::Result<(), String> {
    let mut handles = state.0.lock().map_err(|e| e.to_string())?;
    for h in handles.drain(..) {
        // Send shutdown signal — the stub task handles RCON saveworld+doexit
        // gracefully before killing the process. Do NOT .abort() here because
        // that would cut the task before it can run the RCON shutdown sequence.
        let _ = h.shutdown_tx.send(true);
        // We intentionally don't await or abort — the task will exit on its own
        // after completing the graceful shutdown (a few seconds at most).
    }
    Ok(())
}

#[tauri::command]
pub async fn get_on_demand_status(
    state: tauri::State<'_, Arc<stub::OnDemandState>>,
) -> std::result::Result<Vec<stub::StubStatus>, String> {
    let handles = state.0.lock().map_err(|e| e.to_string())?;
    Ok(handles.iter().map(|h| stub::StubStatus {
        map:        h.map.clone(),
        state:      if h.task.is_finished() { "stopped".to_string() } else { "dormant".to_string() },
        game_port:  h.game_port,
        query_port: h.query_port,
        players:    0,
    }).collect())
}

// ─────────────────────────────────────────────────────────────────────────────
// CurseForge types
// ─────────────────────────────────────────────────────────────────────────────
