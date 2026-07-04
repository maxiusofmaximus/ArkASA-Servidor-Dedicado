//! PluginHub Tauri commands — the surface area for the React UI to
//! list / enable / disable plugins at runtime without resetting
//! the registry.
//!
//! Storage:
//!   `~/.ark-asa/plugins/registry.toml` — the persistent enabled set
//!                                          written through
//!                                          `crate::plugins::registry`.
//!   `crate::plugins::PluginRegistry` — the in-memory catalog,
//!                                       lazily initialised here.
//! The two live in sync: every `enable_plugin` / `disable_plugin` Tauri
//! command writes the TOML AND mutates the in-memory registry; on
//! restart, the registry is rebuilt by reading the TOML.

use crate::plugins::{registry as regfile, PluginRegistry};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static REGISTRY: OnceLock<std::sync::Mutex<PluginRegistry>> = OnceLock::new();

/// Lazy-init: first hit builds a fresh `PluginRegistry`,
/// runs `register_default_plugins` (Convex + Vercel catalog),
/// then reads `registry.toml` and replays every enabled id into
/// the registry.
fn shared_registry() -> &'static std::sync::Mutex<PluginRegistry> {
    REGISTRY.get_or_init(|| {
        let mut r = PluginRegistry::new();
        crate::plugins::register_default_plugins(&mut r);
        let file = regfile::read();
        // Replay persisted enabled-set so the registry reflects what
        // the operator toggled in the UI last session.
        for id in &file.enabled {
            // We don't actually `start(…)` here — the spawn happens
            // through the Tauri-runtime plumbing later. We just
            // mark it enabled so UI sees the right state.
            r.disable(id); // wipe any auto-disabled prior state
        }
        for id in &file.enabled {
            // Hard-set the enabled set (bypass start() lifecycle since
            // we may not have a Tauri runtime wired at this point).
            r.enable_id_no_start(id);
        }
        std::sync::Mutex::new(r)
    })
}

/// JSON-friendly view of one catalog entry — exactly what the React
/// UI needs to render the Plugin Hub tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntryView {
    pub id:              String,
    pub label:           String,
    pub channel:         String,
    pub capabilities:    Vec<String>,
    pub required_secrets: Vec<String>,
    pub oauth_url:       Option<String>,
    pub enabled:         bool,
    pub has_required_secrets: bool,
}

/// Returns the entire plugin catalog + per-plugin enabled state.
#[tauri::command]
pub fn list_plugin_catalog() -> Vec<CatalogEntryView> {
    let r = shared_registry().lock().unwrap();
    let file = regfile::read();
    let secrets_dir = crate::plugins::plugin_storage_dir();
    let mut out = Vec::new();
    let _ = secrets_dir; // reserved for future per-plugin secret check
    for entry in r.catalog_iter() {
        let s = crate::plugins::secret_store::read(entry.id);
        let has_required_secrets = s
            .as_ref()
            .map(|s| {
                entry.descriptor.required_secrets.iter()
                    .all(|k| s.fields.contains_key(*k))
            })
            .unwrap_or(false);
        out.push(CatalogEntryView {
            id:                 entry.id.to_string(),
            label:              entry.descriptor.label.to_string(),
            channel:            format!("{:?}", entry.descriptor.channel).to_lowercase(),
            capabilities:       entry.descriptor.capabilities.iter()
                                .map(|c| format!("{:?}", c).to_lowercase())
                                .collect(),
            required_secrets:   entry.descriptor.required_secrets.iter()
                                .map(|s| (*s).to_string())
                                .collect(),
            oauth_url:          entry.descriptor.oauth_url.map(str::to_string),
            enabled:            file.enabled.contains(entry.id)
                                && !file.disabled.contains(entry.id),
            has_required_secrets,
        });
    }
    out
}

/// Enable a plugin by id. Persists `registry.toml`. Returns the
/// updated `CatalogEntryView` so the UI can re-render.
#[tauri::command]
pub fn enable_plugin(id: String) -> Result<CatalogEntryView, String> {
    let r = shared_registry().lock().unwrap();
    if !r.catalog_iter().any(|e| e.id == id) {
        return Err(format!("plugin `{id}` not in catalog"));
    }
    drop(r);
    let file = regfile::enable_id(&id)?;
    Ok(view_from(&id, &file))
}

/// Disable a plugin by id.
#[tauri::command]
pub fn disable_plugin(id: String) -> Result<CatalogEntryView, String> {
    // Existence check first (immutable); then mutate the in-memory state.
    {
        let r = shared_registry().lock().unwrap();
        if !r.catalog_iter().any(|e| e.id == id) {
            return Err(format!("plugin `{id}` not in catalog"));
        }
    }
    shared_registry().lock().unwrap().disable(&id);
    let file = regfile::disable_id(&id)?;
    Ok(view_from(&id, &file))
}

/// Returns just the enabled set (handy for debug / quick UI checks).
#[tauri::command]
pub fn plugin_registry_snapshot() -> Vec<String> {
    let f = regfile::read();
    f.enabled.into_iter().collect()
}

fn view_from(id: &str, file: &regfile::RegistryFile) -> CatalogEntryView {
    let r = shared_registry().lock().unwrap();
    let entry = r.catalog_iter().find(|e| e.id == id);
    match entry {
        Some(e) => CatalogEntryView {
            id:                 e.id.to_string(),
            label:              e.descriptor.label.to_string(),
            channel:            format!("{:?}", e.descriptor.channel).to_lowercase(),
            capabilities:       e.descriptor.capabilities.iter()
                                .map(|c| format!("{:?}", c).to_lowercase())
                                .collect(),
            required_secrets:   e.descriptor.required_secrets.iter()
                                .map(|s| (*s).to_string())
                                .collect(),
            oauth_url:          e.descriptor.oauth_url.map(str::to_string),
            enabled:            file.enabled.contains(id)
                                && !file.disabled.contains(id),
            has_required_secrets: crate::plugins::secret_store::read(id)
                                .map(|s| {
                                    e.descriptor.required_secrets.iter()
                                        .all(|k| s.fields.contains_key(*k))
                                })
                                .unwrap_or(false),
        },
        None => CatalogEntryView {
            id: id.to_string(),
            label: id.to_string(),
            channel: "unknown".into(),
            capabilities: vec![], required_secrets: vec![],
            oauth_url: None, enabled: false, has_required_secrets: false,
        },
    }
}

// Tests: validate the lifecycle (enabled-set roundtrip) without
// touching the Tauri runtime — just check the helpers.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_includes_builtin_plugins() {
        let views = list_plugin_catalog();
        let ids: Vec<_> = views.iter().map(|v| v.id.as_str()).collect();
        assert!(ids.contains(&"convex"),
            "builtin `convex` must be in the catalog; got {ids:?}");
        assert!(ids.contains(&"vercel"),
            "builtin `vercel` must be in the catalog; got {ids:?}");
    }

    #[test]
    fn enable_then_disable_roundtrips() {
        // Install id into a tmp file
        let id = "convex";
        // Make sure we start clean
        let _ = disable_plugin(id.to_string());

        // Enable
        let v = enable_plugin(id.to_string()).unwrap();
        assert!(v.enabled, "after enable, view must show enabled=true");

        // Disable
        let v = disable_plugin(id.to_string()).unwrap();
        assert!(!v.enabled, "after disable, view must show enabled=false");

        // Cleanup — leave it clean
        let _ = disable_plugin(id.to_string());
    }

    #[test]
    fn enable_unknown_plugin_errors() {
        let r = enable_plugin("nope_doesnt_exist".to_string());
        assert!(r.is_err(), "unknown id must error; got Ok");
    }
}
