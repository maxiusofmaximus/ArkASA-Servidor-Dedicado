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

use crate::commands::get_config_dir;

const LOCAL_MODS_JSON: &str = include_str!("../mods_db.json");

static LOCAL_MODS_DB: LazyLock<Vec<CurseForgeMod>> = LazyLock::new(|| {
    serde_json::from_str(LOCAL_MODS_JSON).unwrap_or_default()
});

pub fn search_local_db(query: &str) -> Vec<CurseForgeMod> {
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

pub fn get_local_mod_by_id(mod_id: &str) -> Option<CurseForgeMod> {
    LOCAL_MODS_DB.iter().find(|m| m.id == mod_id).cloned()
}

pub async fn read_api_key(config_dir: &PathBuf) -> String {
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

const ARK_ASA_GAME_ID:      u32 = 1_172_434;
const CACHE_VALIDITY_SECS:  u64 = 3_600; // 1 hour

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
    /// Field part of the CurseForge API envelope; unused today but kept so a
    /// future feature (e.g. listing available update files) can read it via
    /// the existing serde::Deserialize pass without a follow-up audit.
    /// P3.2 audit (IMPLEMENTATION_PLAN.md §7.2.1): kept with rationale.
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
    /// Field part of the CurseForge API envelope; unused today but kept for
    /// forward-compat with a "show compatible game versions" UI.
    /// P3.2 audit (IMPLEMENTATION_PLAN.md §7.2.1): kept with rationale.
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

pub async fn get_curseforge_api_key(app: tauri::AppHandle) -> std::result::Result<String, String> {
    Ok(read_api_key(&get_config_dir(&app)?).await)
}

#[tauri::command]
pub async fn set_curseforge_api_key(app: tauri::AppHandle, api_key: String) -> std::result::Result<(), String> {
    let config_dir = get_config_dir(&app)?;
    tokio::fs::create_dir_all(&config_dir).await.map_err(|e| e.to_string())?;
    tokio::fs::write(config_dir.join("curseforge_api_key.txt"), api_key.trim())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_curseforge_mod_by_id(
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
pub async fn check_mods_available(
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
pub async fn check_client_only_mods(
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
pub async fn clear_mods_cache(app: tauri::AppHandle) -> std::result::Result<(), String> {
    tokio::fs::remove_file(get_config_dir(&app)?.join("mods_cache.json"))
        .await
        .ok();
    Ok(())
}

#[tauri::command]
pub async fn fetch_curseforge_mods(
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
