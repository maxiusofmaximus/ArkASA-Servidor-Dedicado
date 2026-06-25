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
use std::sync::{Arc, Mutex, LazyLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::process::Command;
use tauri::{Manager, Emitter};

// ─────────────────────────────────────────────────────────────────────────────
// Local mods DB — parsed once at startup, not on every search/lookup.
// ─────────────────────────────────────────────────────────────────────────────

const LOCAL_MODS_JSON: &str = include_str!("mods_db.json");

static LOCAL_MODS_DB: LazyLock<Vec<CurseForgeMod>> = LazyLock::new(|| {
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

pub(crate) static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

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
    tokio::fs::create_dir_all(&config_dir).await.map_err(|e| e.to_string())?;
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
    fs::create_dir_all(&config_dir).await
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
    tokio::fs::create_dir_all(&config_dir).await.map_err(|e| e.to_string())?;
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

    let checks: Vec<_> = mod_ids.iter().map(|mod_id| {
        let url = format!("https://api.curseforge.com/v1/mods/{}", mod_id.trim());
        let key = api_key.clone();
        let id = mod_id.clone();
        async move {
            let resp = HTTP_CLIENT
                .get(&url)
                .header("x-api-key", &key)
                .header("Accept", "application/json")
                .send()
                .await;
            if let Ok(r) = resp {
                if r.status().as_u16() == 404 { Some(id) } else { None }
            } else { None }
        }
    }).collect();
    let unavailable: Vec<String> = futures::future::join_all(checks).await.into_iter().flatten().collect();
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

    let checks: Vec<_> = mod_ids.iter().map(|mod_id| {
        let url = format!("https://api.curseforge.com/v1/mods/{}", mod_id.trim());
        let key = api_key.clone();
        let id = mod_id.clone();
        async move {
            let resp = HTTP_CLIENT
                .get(&url)
                .header("x-api-key", &key)
                .header("Accept", "application/json")
                .send()
                .await;
            if let Ok(r) = resp {
                if r.status().is_success() {
                    if let Ok(body) = r.json::<SingleModResp>().await {
                        let cats = body.data.categories.unwrap_or_default();
                        if detect_client_only(&cats) { return Some(id); }
                    }
                }
            }
            None
        }
    }).collect();
    let client_only: Vec<String> = futures::future::join_all(checks).await.into_iter().flatten().collect();
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
async fn start_server(config: ServerConfig, cluster_delay_sec: Option<u64>) -> std::result::Result<String, String> {
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

        #[cfg(not(windows))]
        {
            use std::os::unix::process::CommandExt;
            unsafe { cmd.pre_exec(|| { libc::setsid(); Ok(()) }); }
        }

        log::info!("Launching ARK {} (cluster={}) port={}", map, is_cluster, game_port);

        match cmd.spawn() {
            Ok(child) => launched.push(format!("{} PID {}", map, child.id())),
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
        #[cfg(not(windows))]
        {
            let _ = std::process::Command::new("pkill")
                .args(["-f", "ArkAscendedServer"])
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
async fn is_server_running() -> bool {
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

#[derive(serde::Serialize)]
struct MapInstanceStatus {
    map_index: usize,
    map_id: String,
    map_label: String,
    running: bool,
}

fn map_display_label(map_id: &str) -> String {
    map_id.trim_end_matches("_WP").replace('_', " ")
}

fn is_tcp_port_open(port: u16) -> bool {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok()
}

#[cfg(windows)]
fn kill_process_on_port(port: u16) -> std::result::Result<(), String> {
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
fn kill_process_on_port(port: u16) -> std::result::Result<(), String> {
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
async fn get_cluster_instance_status(config: ServerConfig) -> std::result::Result<Vec<MapInstanceStatus>, String> {
    let maps = config.effective_maps();
    let futures: Vec<_> = maps
        .iter()
        .enumerate()
        .map(|(i, map_id)| {
            let (_, _, rcon_port) = config.network.ports_for_index(i);
            let map_id_owned = map_id.clone();
            let map_label = map_display_label(map_id);
            tokio::task::spawn_blocking(move || MapInstanceStatus {
                map_index: i,
                map_id: map_id_owned,
                map_label,
                running: is_tcp_port_open(rcon_port),
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
fn start_server_instance(
    config: ServerConfig,
    map_index: usize,
) -> std::result::Result<String, String> {
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
    let (game_port, _, rcon_port) = config.network.ports_for_index(map_index);

    if is_tcp_port_open(rcon_port) {
        return Err(format!("{} is already running (RCON port {} open)", map_display_label(&map), rcon_port));
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
async fn stop_server_instance(config: ServerConfig, map_index: usize) -> std::result::Result<String, String> {
    let maps = config.effective_maps();
    if map_index >= maps.len() {
        return Err(format!("Invalid map index {} (cluster has {} maps)", map_index, maps.len()));
    }

    let map_id = maps[map_index].clone();
    let label = map_display_label(&map_id);
    let password = &config.identification.admin_password;
    let (_, _, rcon_port) = config.network.ports_for_index(map_index);
    let client = RconClient::new(rcon_port, password.as_str());

    match client.graceful_shutdown().await {
        Ok(()) => {
            log::info!("RCON graceful shutdown OK for {} (port {})", label, rcon_port);
            Ok(format!("{} detenido correctamente (saveworld + doexit)", label))
        }
        Err(e) => {
            log::warn!("RCON shutdown failed for {} (port {}): {}", label, rcon_port, e);
            if let Err(kill_err) = kill_process_on_port(rcon_port) {
                Err(format!("RCON failed ({}) and could not kill process: {}", e, kill_err))
            } else {
                Ok(format!("{} detenido (forzado por puerto {})", label, rcon_port))
            }
        }
    }
}

// ── Raw config file I/O ─────────────────────────────────────────────────────

#[tauri::command]
async fn read_text_file(path: String) -> std::result::Result<String, String> {
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(format!("Failed to read {}: {}", path, e)),
    }
}

#[tauri::command]
async fn write_text_file(path: String, content: String) -> std::result::Result<(), String> {
    let p = PathBuf::from(&path);
    if let Some(parent) = p.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    tokio::fs::write(&p, content).await.map_err(|e| format!("Failed to write {}: {}", path, e))?;
    Ok(())
}

#[tauri::command]
fn merge_config_from_ini(config: ServerConfig, ini_content: String) -> std::result::Result<ServerConfig, String> {
    Ok(ConfigLoader::merge_ini_content(&config, &ini_content))
}

#[tauri::command]
async fn restart_server(config: ServerConfig, cluster_delay_sec: Option<u64>) -> std::result::Result<String, String> {
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
// IP auto-detection
// ─────────────────────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct DetectedIps {
    public_ip:    Option<String>,
    tailscale_ip: Option<String>,
    local_ip:     Option<String>,
}

#[tauri::command]
async fn detect_ips() -> DetectedIps {
    let (public_ip, tailscale_ip, local_ip) = tokio::join!(
        detect_public_ip(),
        detect_tailscale_ip(),
        async { detect_local_ip() },
    );
    DetectedIps { public_ip, tailscale_ip, local_ip }
}

#[tauri::command]
fn parse_config_from_toml(toml_str: String) -> Result<config::ServerConfig, String> {
    toml::from_str::<config::ServerConfig>(&toml_str)
        .map_err(|e| format!("Failed to parse TOML: {}", e))
}

/// Serialize a ServerConfig back to TOML string (used after zip import).
#[tauri::command]
fn config_to_toml(config: config::ServerConfig) -> Result<String, String> {
    toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))
}

/// Extract config.toml from a backup .zip and parse it into a ServerConfig.
#[tauri::command]
fn parse_config_from_zip(zip_data: Vec<u8>) -> Result<config::ServerConfig, String> {
    use std::io::Cursor;
    let cursor = Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to open zip: {}", e))?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;
        let name = file.name().to_lowercase();
        if name == "config.toml" || name.ends_with("/config.toml") {
            let mut contents = String::new();
            use std::io::Read;
            file.read_to_string(&mut contents)
                .map_err(|e| format!("Failed to read config.toml from zip: {}", e))?;
            return toml::from_str::<config::ServerConfig>(&contents)
                .map_err(|e| format!("Failed to parse config.toml: {}", e));
        }
    }
    Err("No config.toml found inside the zip file".to_string())
}

async fn detect_public_ip() -> Option<String> {
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

async fn detect_tailscale_ip() -> Option<String> {
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

fn is_tailscale_range(ip: &str) -> bool {
    // Tailscale uses 100.64.0.0/10 → second octet 64-127
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 { return false; }
    let a: u8 = parts[0].parse().unwrap_or(0);
    let b: u8 = parts[1].parse().unwrap_or(0);
    a == 100 && b >= 64 && b <= 127
}

fn detect_local_ip() -> Option<String> {
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
fn open_external_url(url: String) -> std::result::Result<(), String> {
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

            let show_item = MenuItemBuilder::with_id("show", "Show").build(app)?;
            let sep       = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
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
            get_cluster_instance_status,
            start_server_instance,
            stop_server_instance,
            read_text_file,
            write_text_file,
            merge_config_from_ini,
            restart_server,
            get_server_logs,
            get_server_metrics,
            backup_config,
            list_backups,
            restore_backup,
            // Cloud backup
            backup::backup_saves,
            backup::read_backup_metadata,
            backup::read_server_log,
            backup::start_gdrive_oauth,
            backup::start_onedrive_oauth,
            backup::refresh_gdrive_token,
            backup::refresh_onedrive_token,
            backup::test_s3_connection,
            backup::list_cloud_backups,
            backup::restore_backup_from_cloud,
            // IP detection
            detect_ips,
            parse_config_from_toml,
            config_to_toml,
            parse_config_from_zip,
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
