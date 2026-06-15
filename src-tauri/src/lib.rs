pub mod config;
pub mod error;
pub mod cli;
pub mod backup;
pub mod stub;

// Re-export ark sub-modules used in commands
mod ark;

use config::{ConfigLoader, ConfigPersister, ServerConfig, CompositeValidator};
use ark::{build_launch_args, RconClient};
use serde_json::json;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::process::Command;
use tauri::{Manager, Emitter};
use once_cell::sync::Lazy;

// ─────────────────────────────────────────────────────────────────────────────
// Local mods DB — parsed once at startup, not on every search/lookup.
// ─────────────────────────────────────────────────────────────────────────────

const LOCAL_MODS_JSON: &str = include_str!("mods_db.json");

static LOCAL_MODS_DB: Lazy<Vec<CurseForgeMod>> = Lazy::new(|| {
    serde_json::from_str(LOCAL_MODS_JSON).unwrap_or_default()
});

fn search_local_db(query: &str) -> Vec<CurseForgeMod> {
    let q = query.to_lowercase();
    LOCAL_MODS_DB.iter()
        .filter(|m| {
            m.name.to_lowercase().contains(&q)
                || m.summary.to_lowercase().contains(&q)
                || m.slug.to_lowercase().contains(&q)
        })
        .cloned()
        .collect()
}

fn get_local_mod_by_id(mod_id: &str) -> Option<CurseForgeMod> {
    LOCAL_MODS_DB.iter().find(|m| m.id == mod_id).cloned()
}

// ─────────────────────────────────────────────────────────────────────────────
// Path helpers
// ─────────────────────────────────────────────────────────────────────────────

fn get_config_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
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

/// Read the stored CurseForge API key, returning an empty string if absent.
async fn read_api_key(config_dir: &PathBuf) -> String {
    tokio::fs::read_to_string(config_dir.join("curseforge_api_key.txt"))
        .await
        .unwrap_or_default()
        .trim()
        .to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared reqwest client — reuse across all CurseForge calls.
// ─────────────────────────────────────────────────────────────────────────────

static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

const ARK_ASA_GAME_ID:      u32 = 1_172_434;
const CACHE_VALIDITY_SECS:  u64 = 3_600; // 1 hour

// ─────────────────────────────────────────────────────────────────────────────
// Ping state (Tailscale keep-alive)
// ─────────────────────────────────────────────────────────────────────────────

struct PingState(Mutex<Option<tokio::task::JoinHandle<()>>>);

// ─────────────────────────────────────────────────────────────────────────────
// Tray state
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

#[tauri::command]
async fn enable_on_demand(
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

    let (game_port, query_port, rcon_port) = config.network.ports_for_index(map_index);
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
async fn disable_on_demand(
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
async fn disable_all_on_demand(
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
async fn get_on_demand_status(
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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CurseForgeMod {
    pub id:             String,
    pub name:           String,
    pub summary:        String,
    pub download_count: u64,
    pub categories:     Vec<String>,
    pub logo_url:       Option<String>,
    pub slug:           String,
    #[serde(default)]
    pub client_only:    bool,
}

/// Detect PC-only / client-only mods by category.
fn detect_client_only(categories: &[CfCategory]) -> bool {
    categories.iter().any(|c| {
        let n = c.name.to_lowercase();
        n.contains("custom cosmetic") || n == "cosmetics"
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ModsCacheFile {
    mods:        Vec<CurseForgeMod>,
    cached_at:   u64,
    total_count: u64,
}

// CurseForge API response types
#[derive(serde::Deserialize)]
struct CfResponse {
    data:       Vec<CfMod>,
    pagination: CfPagination,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfMod {
    id:             u64,
    name:           String,
    summary:        Option<String>,
    download_count: Option<u64>,
    categories:     Option<Vec<CfCategory>>,
    logo:           Option<CfLogo>,
    slug:           Option<String>,
    #[allow(dead_code)]
    latest_files:   Option<Vec<CfLatestFile>>,
}

#[derive(serde::Deserialize)]
struct CfCategory { name: String }

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfLogo { thumbnail_url: Option<String> }

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfLatestFile {
    #[allow(dead_code)]
    game_versions: Option<Vec<String>>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfPagination { total_count: u64 }

fn cf_mod_to_domain(m: CfMod) -> CurseForgeMod {
    let cats       = m.categories.unwrap_or_default();
    let client_only = detect_client_only(&cats);
    CurseForgeMod {
        id:             m.id.to_string(),
        name:           m.name,
        summary:        m.summary.unwrap_or_default(),
        download_count: m.download_count.unwrap_or(0),
        categories:     cats.into_iter().map(|c| c.name).collect(),
        logo_url:       m.logo.and_then(|l| l.thumbnail_url),
        slug:           m.slug.unwrap_or_default(),
        client_only,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Config commands
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn load_config(config_path: String) -> std::result::Result<ServerConfig, String> {
    ConfigLoader::load_or_default(&PathBuf::from(config_path))
        .await
        .map_err(|e| e.to_string())
}

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
    let result = CompositeValidator::default()
        .validate(&config)
        .await
        .map_err(|e| e.to_string())?;
    Ok(json!({ "valid": result.valid, "errors": result.errors }))
}

#[tauri::command]
async fn save_config(app: tauri::AppHandle, config: ServerConfig) -> std::result::Result<(), String> {
    use tokio::fs;

    let config_dir = get_config_dir(&app)?;
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;

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
        "network":        { "port": { "type": "number", "min": 1024, "max": 65535 } },
        "gameplay":       { "max_players": { "type": "number" } },
        "multipliers":    { "xp_multiplier": { "type": "number" } }
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// CurseForge API key management
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn get_curseforge_api_key(app: tauri::AppHandle) -> std::result::Result<String, String> {
    Ok(read_api_key(&get_config_dir(&app)?).await)
}

#[tauri::command]
async fn set_curseforge_api_key(app: tauri::AppHandle, api_key: String) -> std::result::Result<(), String> {
    let config_dir = get_config_dir(&app)?;
    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    tokio::fs::write(config_dir.join("curseforge_api_key.txt"), api_key.trim())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_curseforge_mod_by_id(
    app: tauri::AppHandle,
    mod_id: String,
) -> std::result::Result<Option<CurseForgeMod>, String> {
    let api_key = read_api_key(&get_config_dir(&app)?).await;
    if api_key.is_empty() {
        return Ok(get_local_mod_by_id(&mod_id));
    }

    let url = format!("https://api.curseforge.com/v1/mods/{}", mod_id.trim());
    let resp = HTTP_CLIENT
        .get(&url)
        .header("x-api-key", &api_key)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if resp.status().as_u16() == 404 {
        return Ok(get_local_mod_by_id(&mod_id));
    }
    if !resp.status().is_success() {
        return Err(format!("CurseForge API error: {}", resp.status()));
    }

    #[derive(serde::Deserialize)]
    struct SingleModResp { data: CfMod }

    let body: SingleModResp = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    Ok(Some(cf_mod_to_domain(body.data)))
}

#[tauri::command]
async fn check_mods_available(
    app: tauri::AppHandle,
    mod_ids: Vec<String>,
) -> std::result::Result<Vec<String>, String> {
    if mod_ids.is_empty() { return Ok(vec![]); }
    let api_key = read_api_key(&get_config_dir(&app)?).await;
    if api_key.is_empty() { return Ok(vec![]); }

    let mut unavailable = vec![];
    for mod_id in &mod_ids {
        let url = format!("https://api.curseforge.com/v1/mods/{}", mod_id.trim());
        if let Ok(resp) = HTTP_CLIENT
            .get(&url)
            .header("x-api-key", &api_key)
            .header("Accept", "application/json")
            .send()
            .await
        {
            if resp.status().as_u16() == 404 {
                unavailable.push(mod_id.clone());
            }
        }
    }
    Ok(unavailable)
}

#[tauri::command]
async fn check_client_only_mods(
    app: tauri::AppHandle,
    mod_ids: Vec<String>,
) -> std::result::Result<Vec<String>, String> {
    if mod_ids.is_empty() { return Ok(vec![]); }
    let api_key = read_api_key(&get_config_dir(&app)?).await;
    if api_key.is_empty() { return Ok(vec![]); }

    #[derive(serde::Deserialize)]
    struct SingleModResp { data: CfMod }

    let mut client_only = vec![];
    for mod_id in &mod_ids {
        let url = format!("https://api.curseforge.com/v1/mods/{}", mod_id.trim());
        if let Ok(resp) = HTTP_CLIENT
            .get(&url)
            .header("x-api-key", &api_key)
            .header("Accept", "application/json")
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<SingleModResp>().await {
                    let cats = body.data.categories.unwrap_or_default();
                    if detect_client_only(&cats) {
                        client_only.push(mod_id.clone());
                    }
                }
            }
        }
    }
    Ok(client_only)
}

#[tauri::command]
async fn clear_mods_cache(app: tauri::AppHandle) -> std::result::Result<(), String> {
    tokio::fs::remove_file(get_config_dir(&app)?.join("mods_cache.json"))
        .await
        .ok();
    Ok(())
}

#[tauri::command]
async fn fetch_curseforge_mods(
    app: tauri::AppHandle,
    page_size:     Option<u32>,
    index:         Option<u32>,
    search_filter: Option<String>,
) -> std::result::Result<serde_json::Value, String> {
    let config_dir  = get_config_dir(&app)?;
    let cache_path  = config_dir.join("mods_cache.json");
    let api_key     = read_api_key(&config_dir).await;

    if api_key.is_empty() {
        return Err("NO_API_KEY".to_string());
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let page_idx   = index.unwrap_or(0);
    let ps         = page_size.unwrap_or(50);
    let has_search = search_filter.as_deref().map_or(false, |s| !s.is_empty());

    // Serve first page from cache when not searching
    if page_idx == 0 && !has_search {
        if let Ok(s) = tokio::fs::read_to_string(&cache_path).await {
            if let Ok(cache) = serde_json::from_str::<ModsCacheFile>(&s) {
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

    let mut req = HTTP_CLIENT
        .get("https://api.curseforge.com/v1/mods/search")
        .query(&[
            ("gameId",    ARK_ASA_GAME_ID.to_string()),
            ("sortField", "2".to_string()),
            ("sortOrder", "desc".to_string()),
            ("pageSize",  ps.to_string()),
            ("index",     page_idx.to_string()),
        ]);
    if let Some(ref q) = search_filter {
        if !q.is_empty() { req = req.query(&[("searchFilter", q.as_str())]); }
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
        return if status.as_u16() == 403 {
            Err("INVALID_API_KEY".to_string())
        } else {
            Err(format!("CurseForge API error {}: {}", status, body))
        };
    }

    let cf_resp: CfResponse = resp.json().await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    let total_count = cf_resp.pagination.total_count;
    let mut mods: Vec<CurseForgeMod> = cf_resp.data.into_iter().map(cf_mod_to_domain).collect();

    // Merge local DB results the API missed when searching.
    // Deduplicate by both ID *and* name (case-insensitive) so a stale local
    // entry with a wrong ID doesn't appear alongside the API's correct entry
    // for the same mod (which would let the user add the wrong ID by mistake).
    if has_search {
        let query      = search_filter.as_deref().unwrap_or("");
        let local      = search_local_db(query);
        let api_ids:   std::collections::HashSet<String> = mods.iter().map(|m| m.id.clone()).collect();
        let api_names: std::collections::HashSet<String> = mods.iter().map(|m| m.name.to_lowercase()).collect();
        mods.extend(local.into_iter().filter(|m| {
            !api_ids.contains(&m.id) && !api_names.contains(&m.name.to_lowercase())
        }));
    }

    // Cache the first page (not search results — they change with the query)
    if page_idx == 0 && !has_search {
        let cache = ModsCacheFile { mods: mods.clone(), cached_at: now, total_count };
        if let Ok(s) = serde_json::to_string(&cache) {
            tokio::fs::write(&cache_path, s).await.ok();
        }
    }

    Ok(json!({ "mods": mods, "total_count": total_count, "from_cache": false }))
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
    let maps       = config.effective_maps();
    let is_cluster = maps.len() > 1;
    let exe        = config.paths.ark_exe();
    let mut launched: Vec<String> = Vec::with_capacity(maps.len());

    for (i, raw_map) in maps.iter().enumerate() {
        let map  = if raw_map.trim().is_empty() { "TheIsland_WP".to_string() } else { raw_map.trim().to_string() };
        let args = build_launch_args(&config, &map, i);
        let (game_port, _, _) = config.network.ports_for_index(i);

        let mut cmd = Command::new(&exe);
        cmd.arg(&args.url_params);
        for flag in &args.flags { cmd.arg(flag); }

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0000_0008); // DETACHED_PROCESS
        }

        log::info!("Launching ARK {} (cluster={}) port={}", map, is_cluster, game_port);

        match cmd.spawn() {
            Ok(child) => launched.push(format!("{} PID {}", map, child.id())),
            Err(e) => return Err(format!("Failed to start {}: {}. Exe: {}", map, e, exe)),
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

/// Graceful shutdown via RCON (saveworld → doexit).
/// Falls back to taskkill /F for instances that don't respond.
#[tauri::command]
async fn stop_server(config: ServerConfig) -> std::result::Result<String, String> {
    let maps     = config.effective_maps();
    let password = &config.identification.admin_password;

    let mut graceful: Vec<String> = vec![];
    let mut failed:   Vec<String> = vec![];

    for (i, map) in maps.iter().enumerate() {
        let (_, _, rcon_port) = config.network.ports_for_index(i);
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

    if !failed.is_empty() {
        log::warn!("Falling back to taskkill for: {:?}", failed);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let _ = std::process::Command::new("taskkill")
                .args(["/F", "/IM", "ArkAscendedServer.exe"])
                .creation_flags(0x08000000)
                .output();
        }
    }

    Ok(match (graceful.is_empty(), failed.is_empty()) {
        (true, _)   => "Servidor detenido (taskkill)".to_string(),
        (_, true)   => format!("Servidor detenido correctamente (saveworld + doexit): {}", graceful.join(", ")),
        _           => format!("RCON OK: {} | taskkill: {}", graceful.join(", "), failed.join(", ")),
    })
}

#[tauri::command]
fn is_server_running() -> bool {
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new("tasklist");
    cmd.args(["/FI", "IMAGENAME eq ArkAscendedServer.exe", "/NH"]);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    matches!(cmd.output(), Ok(out) if String::from_utf8_lossy(&out.stdout).contains("ArkAscendedServer.exe"))
}

#[tauri::command]
fn restart_server(config: ServerConfig) -> std::result::Result<String, String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "ArkAscendedServer.exe"])
            .creation_flags(0x08000000)
            .output();
    }
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

#[tauri::command]
fn open_external_url(url: String) -> std::result::Result<(), String> {
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new("cmd");
    cmd.args(["/C", "start", "", url.as_str()]);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    cmd.spawn().map_err(|e| format!("Failed to open URL: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn start_ping(ip: String, state: tauri::State<'_, PingState>) -> std::result::Result<(), String> {
    // Abort any running ping task first
    if let Some(h) = state.0.lock().map_err(|e| e.to_string())?.take() {
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
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });

    *state.0.lock().map_err(|e| e.to_string())? = Some(handle);
    Ok(())
}

#[tauri::command]
fn stop_ping(state: tauri::State<'_, PingState>) -> std::result::Result<(), String> {
    if let Some(h) = state.0.lock().map_err(|e| e.to_string())?.take() {
        h.abort();
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// App entry point
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let tray_state     = Arc::new(TrayState { minimize_to_tray: AtomicBool::new(true) });
    let on_demand_state = Arc::new(stub::OnDemandState::new());

    tauri::Builder::default()
        .manage(PingState(Mutex::new(None)))
        .manage(tray_state)
        .manage(on_demand_state)
        .setup(|app| {
            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            use tauri::tray::TrayIconBuilder;

            let show_item = MenuItemBuilder::with_id("show", "Mostrar").build(app)?;
            let sep       = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Salir").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show_item, &sep, &quit_item]).build()?;

            let tray = TrayIconBuilder::new()
                .icon(tauri::include_image!("icons/icon.png"))
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

            // In Tauri v2, TrayIcon::drop() removes the icon from the OS tray.
            // We must keep the handle alive for the entire app lifetime.
            // std::mem::forget prevents the destructor from running.
            std::mem::forget(tray);

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
