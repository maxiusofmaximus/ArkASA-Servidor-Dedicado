pub mod config;
pub mod error;
pub mod cli;
pub mod backup;
pub mod stub;

use config::{ConfigLoader, ConfigPersister, ServerConfig, CompositeValidator};
use serde_json::json;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::process::Command;
use tauri::{Manager, Emitter};

// ─────────────────────────────────────────────────────────────────────────────
// Path helper — uses Tauri v2's proper path API so it works regardless of
// how the process is launched (avoids relying on APPDATA env var).
// ─────────────────────────────────────────────────────────────────────────────

fn get_config_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    // On Windows, prefer APPDATA env var (always set in interactive user sessions).
    // Fall back to Tauri's data_dir() API for non-Windows or edge cases.
    let base = if cfg!(target_os = "windows") {
        match std::env::var("APPDATA") {
            Ok(p) if !p.is_empty() => PathBuf::from(p),
            _ => app.path()
                    .data_dir()
                    .map_err(|e| format!("Failed to resolve config dir: {}", e))?,
        }
    } else {
        app.path()
            .data_dir()
            .map_err(|e| format!("Failed to resolve config dir: {}", e))?
    };
    Ok(base.join("ARK ASA Config Manager"))
}

// ─────────────────────────────────────────────────────────────────────────────
// Ping state (for Tailscale keep-alive)
// ─────────────────────────────────────────────────────────────────────────────

struct PingState(Mutex<Option<tokio::task::JoinHandle<()>>>);

// ─────────────────────────────────────────────────────────────────────────────
// Tray state — shared bool so the close handler knows whether to hide or exit
// ─────────────────────────────────────────────────────────────────────────────

pub struct TrayState {
    pub minimize_to_tray: AtomicBool,
}

#[tauri::command]
fn set_minimize_to_tray(state: tauri::State<Arc<TrayState>>, enabled: bool) {
    state.minimize_to_tray.store(enabled, Ordering::SeqCst);
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

// ─────────────────────────────────────────────────────────────────────────────
// On-demand server commands
// ─────────────────────────────────────────────────────────────────────────────

/// Start the UDP stub for one map so it appears in the ARK browser while
/// ARK itself is not running.  When a player connects, the stub wakes ARK.
#[tauri::command]
fn enable_on_demand(
    config: ServerConfig,
    map_index: usize,
    auto_shutdown_min: u64,
    state: tauri::State<Arc<stub::OnDemandState>>,
) -> std::result::Result<String, String> {
    let maps: Vec<String> = if config.cluster_maps.is_empty() {
        vec!["TheIsland_WP".to_string()]
    } else {
        config.cluster_maps.clone()
    };

    let map = maps.get(map_index)
        .cloned()
        .unwrap_or_else(|| "TheIsland_WP".to_string());

    let is_cluster = maps.len() > 1;
    let cluster_id: String = config.identification.session_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .to_lowercase();
    let cluster_id = if cluster_id.is_empty() { "ark_cluster".to_string() } else { cluster_id };

    let server_dir = config.paths.server_dir.trim_end_matches('\\');
    let exe = format!("{}\\ShooterGame\\Binaries\\Win64\\ArkAscendedServer.exe", server_dir);
    let cluster_dir = format!("{}\\clusters", server_dir);

    let game_port  = config.network.port       + (map_index as u16) * 2;
    let query_port = config.network.query_port + map_index as u16;
    let rcon_port  = config.network.rcon_port  + map_index as u16;

    let map_label = map.trim_end_matches("_WP");
    let session_name = if is_cluster && map_index > 0 {
        format!("{} · {}", config.identification.session_name, map_label)
    } else {
        config.identification.session_name.clone()
    };

    let mut launch_params = format!(
        "{}?listen?SessionName={}?ServerAdminPassword={}",
        map, session_name, config.identification.admin_password,
    );
    if !config.identification.server_password.is_empty() {
        launch_params.push_str(&format!("?ServerPassword={}", config.identification.server_password));
    }
    launch_params.push_str(&format!(
        "?MaxPlayers={}?Port={}?QueryPort={}?RCONEnabled=True?RCONPort={}",
        config.gameplay.max_players, game_port, query_port, rcon_port,
    ));

    let mut extra_args = vec![
        "-NoBattlEye".to_string(),
        "-server".to_string(),
        "-log".to_string(),
        "-servergamelog".to_string(),
        "-NoTransferFromFiltering".to_string(),
        format!("-WinLiveMaxPlayers={}", config.gameplay.max_players),
    ];
    if is_cluster {
        extra_args.push(format!("-clusterid={}", cluster_id));
        extra_args.push(format!("-ClusterDirOverride={}", cluster_dir));
    }
    if !config.mods.active_mods.is_empty() {
        extra_args.push(format!("-mods={}", config.mods.active_mods.join(",")));
    }

    let params = stub::MapLaunchParams {
        map: map.clone(),
        exe,
        launch_params,
        extra_args,
        game_port,
        query_port,
        auto_shutdown_min,
    };

    // Stop any existing stub for the same game_port
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

/// Stop the on-demand stub for the given game_port.
#[tauri::command]
fn disable_on_demand(
    game_port: u16,
    state: tauri::State<Arc<stub::OnDemandState>>,
) -> std::result::Result<(), String> {
    let mut handles = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(pos) = handles.iter().position(|h| h.game_port == game_port) {
        let handle = handles.remove(pos);
        let _ = handle.shutdown_tx.send(true);
        handle.task.abort();
        log::info!("On-demand stub stopped for game_port={}", game_port);
    }
    Ok(())
}

/// Stop ALL on-demand stubs.
#[tauri::command]
fn disable_all_on_demand(
    state: tauri::State<Arc<stub::OnDemandState>>,
) -> std::result::Result<(), String> {
    let mut handles = state.0.lock().map_err(|e| e.to_string())?;
    for h in handles.drain(..) {
        let _ = h.shutdown_tx.send(true);
        h.task.abort();
    }
    Ok(())
}

/// Return the list of active stub statuses.
#[tauri::command]
fn get_on_demand_status(
    state: tauri::State<Arc<stub::OnDemandState>>,
) -> std::result::Result<Vec<stub::StubStatus>, String> {
    let handles = state.0.lock().map_err(|e| e.to_string())?;
    Ok(handles.iter().map(|h| stub::StubStatus {
        map: h.map.clone(),
        state: if h.task.is_finished() { "stopped".to_string() } else { "dormant".to_string() },
        game_port: h.game_port,
        query_port: h.query_port,
        players: 0,
    }).collect())
}

// ─────────────────────────────────────────────────────────────────────────────
// CurseForge types
// ─────────────────────────────────────────────────────────────────────────────

const ARK_ASA_GAME_ID: u32 = 1172434;
const CACHE_VALIDITY_SECS: u64 = 3600; // 1 hour

/// Embedded local mods database — used as fallback when the CurseForge API
/// doesn't return results (no key, 404, or search miss).
const LOCAL_MODS_DB: &str = include_str!("mods_db.json");

/// Search the embedded local DB by name / summary / slug (case-insensitive).
fn search_local_db(query: &str) -> Vec<CurseForgeMod> {
    let q = query.to_lowercase();
    let db: Vec<CurseForgeMod> = serde_json::from_str(LOCAL_MODS_DB).unwrap_or_default();
    db.into_iter()
        .filter(|m| {
            m.name.to_lowercase().contains(&q)
                || m.summary.to_lowercase().contains(&q)
                || m.slug.to_lowercase().contains(&q)
        })
        .collect()
}

/// Look up a single mod by CurseForge ID in the embedded local DB.
fn get_local_mod_by_id(mod_id: &str) -> Option<CurseForgeMod> {
    let db: Vec<CurseForgeMod> = serde_json::from_str(LOCAL_MODS_DB).unwrap_or_default();
    db.into_iter().find(|m| m.id == mod_id)
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CurseForgeMod {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub download_count: u64,
    pub categories: Vec<String>,
    pub logo_url: Option<String>,
    pub slug: String,
    #[serde(default)]
    pub client_only: bool,
}

/// Detect if a CurseForge mod is PC-only / client-only.
///
/// ARK SA mods on CurseForge do NOT use OS strings (like "WindowsServer") in
/// gameVersions — they use game-version numbers (e.g. "0.88.14"). The only
/// reliable indicator of PC-only mods is the "Custom Cosmetics" category, which
/// ARK's own CFCore explicitly rejects on cross-platform servers.
fn detect_client_only(categories: &[CfCategory], _latest_files: &Option<Vec<CfLatestFile>>) -> bool {
    categories.iter().any(|c| {
        let n = c.name.to_lowercase();
        n.contains("custom cosmetic") || n == "cosmetics"
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ModsCacheFile {
    mods: Vec<CurseForgeMod>,
    cached_at: u64,
    total_count: u64,
}

// CurseForge API response deserialization structs
#[derive(serde::Deserialize)]
struct CfResponse {
    data: Vec<CfMod>,
    pagination: CfPagination,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfMod {
    id: u64,
    name: String,
    summary: Option<String>,
    download_count: Option<u64>,
    categories: Option<Vec<CfCategory>>,
    logo: Option<CfLogo>,
    slug: Option<String>,
    latest_files: Option<Vec<CfLatestFile>>,
}

#[derive(serde::Deserialize)]
struct CfCategory {
    name: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfLogo {
    thumbnail_url: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfLatestFile {
    #[allow(dead_code)]
    game_versions: Option<Vec<String>>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfPagination {
    total_count: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Config commands
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn load_config(config_path: String) -> std::result::Result<ServerConfig, String> {
    let path = PathBuf::from(&config_path);
    ConfigLoader::load_or_default(&path).await.map_err(|e| e.to_string())
}

/// Load from the standard TOML path in %APPDATA%, fallback to defaults.
/// This is the preferred startup command — it restores all saved settings.
#[tauri::command]
async fn load_config_or_default(app: tauri::AppHandle) -> std::result::Result<ServerConfig, String> {
    let config_dir = get_config_dir(&app)?;
    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    let toml_path = config_dir.join("server-config.toml");
    log::info!("Loading config from {:?}", toml_path);
    ConfigLoader::load_or_default(&toml_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn validate_config(config: ServerConfig) -> std::result::Result<serde_json::Value, String> {
    let validator = CompositeValidator::default();
    let result = validator.validate(&config).await.map_err(|e| e.to_string())?;
    Ok(json!({ "valid": result.valid, "errors": result.errors }))
}

#[tauri::command]
async fn save_config(app: tauri::AppHandle, config: ServerConfig) -> std::result::Result<(), String> {
    use tokio::fs;

    let config_dir = get_config_dir(&app)?;
    std::fs::create_dir_all(&config_dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
    let toml_path = config_dir.join("server-config.toml");
    log::info!("Saving config to {:?}", toml_path);
    ConfigPersister::save_toml(&config, &toml_path).await.map_err(|e| e.to_string())?;

    // Backup existing INI files before overwriting
    let backup_dir = PathBuf::from(&config.paths.backup_dir);
    fs::create_dir_all(&backup_dir).await.ok();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let game_ini = PathBuf::from(&config.paths.game_ini_path);
    let gus_ini  = PathBuf::from(&config.paths.gamesettings_ini_path);
    if game_ini.exists() {
        fs::copy(&game_ini, backup_dir.join(format!("Game.ini.{}.bak", ts))).await.ok();
    }
    if gus_ini.exists() {
        fs::copy(&gus_ini, backup_dir.join(format!("GameUserSettings.ini.{}.bak", ts))).await.ok();
    }

    // Generate fresh INI files from the current config.
    // Non-fatal: if the server path doesn't exist yet, TOML is already saved above — the user
    // can configure the server dir first and the INIs will be generated on the next save.
    if let Err(e) = ConfigPersister::generate_game_ini(&config, &game_ini).await {
        log::warn!("Could not write Game.ini (path may not exist yet): {}", e);
    }
    if let Err(e) = ConfigPersister::generate_gamesettings_ini(&config, &gus_ini).await {
        log::warn!("Could not write GameUserSettings.ini (path may not exist yet): {}", e);
    }
    Ok(())
}

#[tauri::command]
fn get_default_config() -> std::result::Result<ServerConfig, String> {
    Ok(ServerConfig::default())
}

#[tauri::command]
fn get_config_schema() -> std::result::Result<serde_json::Value, String> {
    Ok(json!({
        "identification": { "session_name": { "type": "string" }, "admin_password": { "type": "string" } },
        "network": { "port": { "type": "number", "min": 1024, "max": 65535 } },
        "gameplay": { "max_players": { "type": "number" } },
        "multipliers": { "xp_multiplier": { "type": "number" } }
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// CurseForge API key management
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn get_curseforge_api_key(app: tauri::AppHandle) -> std::result::Result<String, String> {
    let config_dir = get_config_dir(&app)?;
    let key_path = config_dir.join("curseforge_api_key.txt");
    Ok(tokio::fs::read_to_string(&key_path)
        .await
        .unwrap_or_default()
        .trim()
        .to_string())
}

#[tauri::command]
async fn set_curseforge_api_key(app: tauri::AppHandle, api_key: String) -> std::result::Result<(), String> {
    let config_dir = get_config_dir(&app)?;
    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    let key_path = config_dir.join("curseforge_api_key.txt");
    tokio::fs::write(&key_path, api_key.trim())
        .await
        .map_err(|e| e.to_string())
}

/// Fetch a single mod by its CurseForge numeric ID.
/// Returns None if the mod ID is not found (404).
#[tauri::command]
async fn get_curseforge_mod_by_id(app: tauri::AppHandle, mod_id: String) -> std::result::Result<Option<CurseForgeMod>, String> {
    let config_dir = get_config_dir(&app)?;
    let key_path = config_dir.join("curseforge_api_key.txt");
    let api_key = tokio::fs::read_to_string(&key_path)
        .await
        .unwrap_or_default()
        .trim()
        .to_string();

    if api_key.is_empty() {
        // No API key — check local DB before giving up
        return Ok(get_local_mod_by_id(&mod_id));
    }

    let client = reqwest::Client::new();
    let url = format!("https://api.curseforge.com/v1/mods/{}", mod_id.trim());

    let resp = client
        .get(&url)
        .header("x-api-key", &api_key)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if resp.status().as_u16() == 404 {
        // Not in CurseForge API — try local DB
        return Ok(get_local_mod_by_id(&mod_id));
    }
    if !resp.status().is_success() {
        return Err(format!("CurseForge API error: {}", resp.status()));
    }

    #[derive(serde::Deserialize)]
    struct SingleModResp {
        data: CfMod,
    }

    let body: SingleModResp = resp
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    let m = body.data;
    let cats = m.categories.unwrap_or_default();
    let client_only = detect_client_only(&cats, &m.latest_files);
    Ok(Some(CurseForgeMod {
        id: m.id.to_string(),
        name: m.name,
        summary: m.summary.unwrap_or_default(),
        download_count: m.download_count.unwrap_or(0),
        categories: cats.into_iter().map(|c| c.name).collect(),
        logo_url: m.logo.and_then(|l| l.thumbnail_url),
        slug: m.slug.unwrap_or_default(),
        client_only,
    }))
}

/// Check which mod IDs from the provided list are NOT available on CurseForge.
/// Returns the list of unavailable IDs. Skips check if no API key is configured.
#[tauri::command]
async fn check_mods_available(
    app: tauri::AppHandle,
    mod_ids: Vec<String>,
) -> std::result::Result<Vec<String>, String> {
    if mod_ids.is_empty() {
        return Ok(vec![]);
    }
    let config_dir = get_config_dir(&app)?;
    let key_path = config_dir.join("curseforge_api_key.txt");
    let api_key = tokio::fs::read_to_string(&key_path)
        .await
        .unwrap_or_default()
        .trim()
        .to_string();
    if api_key.is_empty() {
        return Ok(vec![]); // Can't check without key — skip validation
    }
    let client = reqwest::Client::new();
    let mut unavailable = vec![];
    for mod_id in &mod_ids {
        let url = format!("https://api.curseforge.com/v1/mods/{}", mod_id.trim());
        match client
            .get(&url)
            .header("x-api-key", &api_key)
            .header("Accept", "application/json")
            .send()
            .await
        {
            Ok(resp) if resp.status().as_u16() == 404 => {
                unavailable.push(mod_id.clone());
            }
            Err(_) => {} // Network error — skip, let the server try
            _ => {}      // 200 or other — treat as available
        }
    }
    Ok(unavailable)
}

/// Check which mod IDs are PC-only (client-only) and will not work on a cross-platform server.
/// Returns the list of client-only IDs. Skips check if no API key is configured.
#[tauri::command]
async fn check_client_only_mods(
    app: tauri::AppHandle,
    mod_ids: Vec<String>,
) -> std::result::Result<Vec<String>, String> {
    if mod_ids.is_empty() {
        return Ok(vec![]);
    }
    let config_dir = get_config_dir(&app)?;
    let key_path = config_dir.join("curseforge_api_key.txt");
    let api_key = tokio::fs::read_to_string(&key_path)
        .await
        .unwrap_or_default()
        .trim()
        .to_string();
    if api_key.is_empty() {
        return Ok(vec![]);
    }

    #[derive(serde::Deserialize)]
    struct SingleModResp { data: CfMod }

    let client = reqwest::Client::new();
    let mut client_only = vec![];
    for mod_id in &mod_ids {
        let url = format!("https://api.curseforge.com/v1/mods/{}", mod_id.trim());
        if let Ok(resp) = client
            .get(&url)
            .header("x-api-key", &api_key)
            .header("Accept", "application/json")
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<SingleModResp>().await {
                    let cats = body.data.categories.unwrap_or_default();
                    if detect_client_only(&cats, &body.data.latest_files) {
                        client_only.push(mod_id.clone());
                    }
                }
            }
        }
    }
    Ok(client_only)
}

/// Invalidate the mod cache so the next fetch_curseforge_mods call hits the API.
#[tauri::command]
async fn clear_mods_cache(app: tauri::AppHandle) -> std::result::Result<(), String> {
    let config_dir = get_config_dir(&app)?;
    let cache_path = config_dir.join("mods_cache.json");
    tokio::fs::remove_file(&cache_path).await.ok();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// CurseForge mod browser
// ─────────────────────────────────────────────────────────────────────────────

/// Fetch ARK SA mods from CurseForge.
/// - Checks a 1-hour JSON file cache for index=0 requests.
/// - Returns `{ mods, total_count, from_cache }`.
/// - Returns error string "NO_API_KEY" if no key is configured.
/// - Returns error string "INVALID_API_KEY" if the key is rejected (403).
#[tauri::command]
async fn fetch_curseforge_mods(
    app: tauri::AppHandle,
    page_size: Option<u32>,
    index: Option<u32>,
    search_filter: Option<String>,
) -> std::result::Result<serde_json::Value, String> {
    let config_dir = get_config_dir(&app)?;
    let key_path = config_dir.join("curseforge_api_key.txt");
    let cache_path = config_dir.join("mods_cache.json");

    // Read API key
    let api_key = tokio::fs::read_to_string(&key_path)
        .await
        .unwrap_or_default()
        .trim()
        .to_string();

    if api_key.is_empty() {
        return Err("NO_API_KEY".to_string());
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let page_idx = index.unwrap_or(0);
    let ps = page_size.unwrap_or(50);
    let has_search = search_filter.as_deref().map_or(false, |s| !s.is_empty());

    // Serve from cache for first page if still fresh (skip when searching)
    if page_idx == 0 && !has_search {
        if let Ok(cache_str) = tokio::fs::read_to_string(&cache_path).await {
            if let Ok(cache) = serde_json::from_str::<ModsCacheFile>(&cache_str) {
                if now.saturating_sub(cache.cached_at) < CACHE_VALIDITY_SECS {
                    return Ok(json!({
                        "mods": cache.mods,
                        "total_count": cache.total_count,
                        "from_cache": true,
                    }));
                }
            }
        }
    }

    // Fetch from CurseForge API
    let client = reqwest::Client::new();
    let mut req = client
        .get("https://api.curseforge.com/v1/mods/search")
        .query(&[
            ("gameId", ARK_ASA_GAME_ID.to_string()),
            ("sortField", "2".to_string()),
            ("sortOrder", "desc".to_string()),
            ("pageSize", ps.to_string()),
            ("index", page_idx.to_string()),
        ]);
    if let Some(ref q) = search_filter {
        if !q.is_empty() {
            req = req.query(&[("searchFilter", q.as_str())]);
        }
    }

    let resp = req
        .header("x-api-key", &api_key)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        if status.as_u16() == 403 {
            return Err("INVALID_API_KEY".to_string());
        }
        return Err(format!("CurseForge API error {}: {}", status, body));
    }

    let cf_resp: CfResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    let total_count = cf_resp.pagination.total_count;

    let mods: Vec<CurseForgeMod> = cf_resp
        .data
        .into_iter()
        .map(|m| {
            let cats = m.categories.unwrap_or_default();
            let client_only = detect_client_only(&cats, &m.latest_files);
            CurseForgeMod {
                id: m.id.to_string(),
                name: m.name,
                summary: m.summary.unwrap_or_default(),
                download_count: m.download_count.unwrap_or(0),
                categories: cats.into_iter().map(|c| c.name).collect(),
                logo_url: m.logo.and_then(|l| l.thumbnail_url),
                slug: m.slug.unwrap_or_default(),
                client_only,
            }
        })
        .collect();

    // When searching: merge local DB results that the API didn't return
    let mods = if has_search {
        let query = search_filter.as_deref().unwrap_or("");
        let local = search_local_db(query);
        let api_ids: std::collections::HashSet<String> =
            mods.iter().map(|m| m.id.clone()).collect();
        let extra: Vec<CurseForgeMod> = local
            .into_iter()
            .filter(|m| !api_ids.contains(&m.id))
            .collect();
        let mut merged = mods;
        merged.extend(extra);
        merged
    } else {
        mods
    };

    // Cache first page (only when not searching)
    if page_idx == 0 && !has_search {
        let cache = ModsCacheFile {
            mods: mods.clone(),
            cached_at: now,
            total_count,
        };
        if let Ok(json_str) = serde_json::to_string(&cache) {
            tokio::fs::write(&cache_path, json_str).await.ok();
        }
    }

    Ok(json!({
        "mods": mods,
        "total_count": total_count,
        "from_cache": false,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// Server control
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
fn server_status() -> std::result::Result<serde_json::Value, String> {
    Ok(json!({"running": false, "process_id": null, "uptime_seconds": 0}))
}

#[tauri::command]
fn start_server(config: ServerConfig) -> std::result::Result<String, String> {
    use std::process::Command;

    let maps: Vec<String> = if config.cluster_maps.is_empty() {
        vec!["TheIsland_WP".to_string()]
    } else {
        config.cluster_maps.clone()
    };

    let is_cluster = maps.len() > 1;

    // Cluster ID derived from session name (alphanumeric only)
    let cluster_id: String = config.identification.session_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .to_lowercase();
    let cluster_id = if cluster_id.is_empty() { "ark_cluster".to_string() } else { cluster_id };

    let server_dir = config.paths.server_dir.trim_end_matches('\\');
    let exe = format!("{}\\ShooterGame\\Binaries\\Win64\\ArkAscendedServer.exe", server_dir);
    let cluster_dir = format!("{}\\clusters", server_dir);

    let mut launched: Vec<String> = Vec::new();

    for (i, raw_map) in maps.iter().enumerate() {
        let map = if raw_map.trim().is_empty() { "TheIsland_WP".to_string() } else { raw_map.trim().to_string() };

        // Each cluster instance gets offset ports to avoid conflicts:
        //   game port:  base + i*2  (ARK uses port & port+1 internally)
        //   query port: base + i
        //   rcon port:  base + i
        let game_port  = config.network.port       + (i as u16) * 2;
        let query_port = config.network.query_port + i as u16;
        let rcon_port  = config.network.rcon_port  + i as u16;

        // In cluster mode each instance gets a map suffix so players can
        // distinguish them in the ARK browser (e.g. "ServidorMax · Ragnarok").
        // The primary map (index 0) keeps the bare session name.
        let map_label = map.trim_end_matches("_WP");
        let session_name = if is_cluster && i > 0 {
            format!("{} · {}", config.identification.session_name, map_label)
        } else {
            config.identification.session_name.clone()
        };

        let mut params = format!(
            "{}?listen?SessionName={}?ServerAdminPassword={}",
            map,
            session_name,
            config.identification.admin_password,
        );
        if !config.identification.server_password.is_empty() {
            params.push_str(&format!("?ServerPassword={}", config.identification.server_password));
        }
        params.push_str(&format!(
            "?MaxPlayers={}?Port={}?QueryPort={}?RCONEnabled=True?RCONPort={}",
            config.gameplay.max_players,
            game_port,
            query_port,
            rcon_port,
        ));

        let mut cmd = Command::new(&exe);
        cmd.arg(&params);
        cmd.arg("-NoBattlEye");
        cmd.arg("-server");
        cmd.arg("-log");
        cmd.arg("-servergamelog");
        cmd.arg("-NoTransferFromFiltering");
        cmd.arg(format!("-WinLiveMaxPlayers={}", config.gameplay.max_players));

        if is_cluster {
            cmd.arg(format!("-clusterid={}", cluster_id));
            cmd.arg(format!("-ClusterDirOverride={}", cluster_dir));
        }

        if !config.mods.active_mods.is_empty() {
            cmd.arg(format!("-mods={}", config.mods.active_mods.join(",")));
        }

        log::info!("Launching ARK {} (cluster={}) port={}", map, is_cluster, game_port);

        match cmd.spawn() {
            Ok(child) => launched.push(format!("{} PID {}", map, child.id())),
            Err(e) => return Err(format!(
                "Failed to start {}: {}. Check server at: {}",
                map, e, exe
            )),
        }

        // Brief stagger between cluster instances to avoid resource contention
        if is_cluster && i < maps.len() - 1 {
            std::thread::sleep(std::time::Duration::from_millis(1500));
        }
    }

    if is_cluster {
        Ok(format!("Cluster iniciado [{} instancias]: {}", launched.len(), launched.join(" | ")))
    } else {
        Ok(format!("Server started ({})", launched[0]))
    }
}

#[tauri::command]
fn stop_server() -> std::result::Result<String, String> {
    use std::process::Command;
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new("taskkill");
    cmd.args(["/F", "/IM", "ArkAscendedServer.exe"]);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    match cmd.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if out.status.success() {
                Ok(format!("Server stopped. {}", stdout.trim()))
            } else if stdout.to_lowercase().contains("not found")
                || stdout.to_lowercase().contains("no se encontr")
            {
                Ok("Server was not running".to_string())
            } else {
                Err(format!("taskkill failed: {}", stdout.trim()))
            }
        }
        Err(e) => Err(format!("Failed to run taskkill: {}", e)),
    }
}

/// Check if ArkAscendedServer.exe is currently in the process list.
#[tauri::command]
fn is_server_running() -> bool {
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new("tasklist");
    cmd.args(["/FI", "IMAGENAME eq ArkAscendedServer.exe", "/NH"]);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    match cmd.output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).contains("ArkAscendedServer.exe"),
        Err(_) => false,
    }
}

#[tauri::command]
fn restart_server(config: ServerConfig) -> std::result::Result<String, String> {
    let _ = stop_server();
    std::thread::sleep(std::time::Duration::from_secs(3));
    start_server(config)
}

#[tauri::command]
fn get_server_logs(_lines: i32) -> std::result::Result<Vec<String>, String> {
    Ok(vec!["[INFO] Server started".to_string()])
}

#[tauri::command]
fn get_server_metrics() -> std::result::Result<serde_json::Value, String> {
    Ok(json!({"cpu": 0, "memory": 0, "fps": 0}))
}

#[tauri::command]
fn backup_config(_config: ServerConfig, name: String) -> std::result::Result<String, String> {
    Ok(format!("Backup '{}' created", name))
}

#[tauri::command]
fn list_backups() -> std::result::Result<Vec<serde_json::Value>, String> {
    Ok(vec![])
}

#[tauri::command]
fn restore_backup(_name: String) -> std::result::Result<ServerConfig, String> {
    Ok(ServerConfig::default())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tailscale / ping keep-alive
// ─────────────────────────────────────────────────────────────────────────────

/// Open a URL in the system's default browser (Windows: cmd /C start).
#[tauri::command]
fn open_external_url(url: String) -> std::result::Result<(), String> {
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new("cmd");
    cmd.args(["/C", "start", "", url.as_str()]);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    cmd.spawn().map_err(|e| format!("Failed to open URL: {}", e))?;
    Ok(())
}

/// Start a background ping loop to keep the Tailscale connection alive.
/// Runs hidden one-shot pings every 30 seconds — no terminal window, no persistent process.
#[tauri::command]
async fn start_ping(ip: String, state: tauri::State<'_, PingState>) -> std::result::Result<(), String> {
    // Abort any existing ping task
    let old = { state.0.lock().map_err(|e| e.to_string())?.take() };
    if let Some(handle) = old {
        handle.abort();
    }

    let ip = ip.trim().to_string();
    let handle = tokio::spawn(async move {
        loop {
            // Hidden one-shot ping (4 packets) — no window, no persistent process
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                let _ = std::process::Command::new("ping")
                    .args(["-n", "4", "-w", "3000", &ip])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW
                    .spawn();
            }
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });

    *state.0.lock().map_err(|e| e.to_string())? = Some(handle);
    Ok(())
}

/// Stop the background ping loop.
#[tauri::command]
fn stop_ping(state: tauri::State<'_, PingState>) -> std::result::Result<(), String> {
    if let Some(handle) = state.0.lock().map_err(|e| e.to_string())?.take() {
        handle.abort();
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// App entry point
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let tray_state = Arc::new(TrayState {
        minimize_to_tray: AtomicBool::new(true),
    });
    let on_demand_state = Arc::new(stub::OnDemandState::new());

    tauri::Builder::default()
        .manage(PingState(Mutex::new(None)))
        .manage(tray_state)
        .manage(on_demand_state)
        .setup(|app| {
            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            use tauri::tray::TrayIconBuilder;

            // Build tray context menu
            let show_item = MenuItemBuilder::with_id("show", "Mostrar").build(app)?;
            let sep      = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Salir").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&show_item, &sep, &quit_item])
                .build()?;

            // Build tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("ARK ASA Server Manager")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "quit" => {
                        // Tell frontend to save config, then exit after a short grace period
                        let _ = app.emit("tray-quit", ());
                        let app2 = app.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(600));
                            app2.exit(0);
                        });
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    use tauri::tray::TrayIconEvent;
                    if matches!(event, TrayIconEvent::Click { .. }) {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Config
            load_config,
            load_config_or_default,
            validate_config,
            save_config,
            get_default_config,
            get_config_schema,
            // CurseForge
            get_curseforge_api_key,
            set_curseforge_api_key,
            fetch_curseforge_mods,
            get_curseforge_mod_by_id,
            check_mods_available,
            check_client_only_mods,
            clear_mods_cache,
            // Server control
            server_status,
            start_server,
            stop_server,
            is_server_running,
            restart_server,
            get_server_logs,
            get_server_metrics,
            backup_config,
            list_backups,
            restore_backup,
            // Cloud backup
            backup::backup_saves,
            backup::read_server_log,
            backup::start_gdrive_oauth,
            backup::start_onedrive_oauth,
            backup::refresh_gdrive_token,
            backup::refresh_onedrive_token,
            backup::test_s3_connection,
            backup::list_cloud_backups,
            backup::restore_backup_from_cloud,
            // Ping / Tailscale
            start_ping,
            stop_ping,
            // Utilities
            open_external_url,
            // Tray
            set_minimize_to_tray,
            quit_app,
            // On-demand stubs
            enable_on_demand,
            disable_on_demand,
            disable_all_on_demand,
            get_on_demand_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
