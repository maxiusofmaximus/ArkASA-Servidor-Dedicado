//! Multi-server profiles.
//!
//! Historically the app managed exactly one server: a single
//! `<config_dir>/server-config.toml`. This module introduces a profile
//! registry so an operator can keep several independent servers side by side —
//! each with its own config (and therefore its own saves/output paths) and its
//! own `mods`, cluster, ports and passwords.
//!
//! Layout (backwards compatible):
//!
//! ```text
//! <config_dir>/
//!   profiles.json                     # index: active id + profile list
//!   server-config.toml                # LEGACY single-server config
//!   servers/<id>/server-config.toml   # new profiles
//! ```
//!
//! On first use, if `profiles.json` is absent but the legacy
//! `server-config.toml` exists, it is adopted as the `default` profile whose
//! `config_path` points at the legacy file — nothing is moved, so an existing
//! installation keeps working untouched.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// One managed server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerProfile {
    /// Stable, filesystem-safe identifier (also the `servers/<id>/` folder).
    pub id: String,
    /// Human label shown in the UI / CLI.
    pub name: String,
    /// Absolute path to this profile's `server-config.toml`.
    pub config_path: String,
    /// Creation time (Unix seconds).
    pub created_at_unix: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ProfilesIndex {
    #[serde(default)]
    active: Option<String>,
    #[serde(default)]
    profiles: Vec<ServerProfile>,
}

/// Read/write access to the profile registry. Cheap to construct; every method
/// re-reads the index from disk so concurrent callers (GUI, CLI, bots) stay in
/// sync.
pub struct ProfileStore {
    config_dir: PathBuf,
}

impl ProfileStore {
    pub fn new(config_dir: impl Into<PathBuf>) -> Self {
        Self { config_dir: config_dir.into() }
    }

    fn index_path(&self) -> PathBuf {
        self.config_dir.join("profiles.json")
    }

    fn legacy_path(&self) -> PathBuf {
        self.config_dir.join("server-config.toml")
    }

    fn servers_dir(&self) -> PathBuf {
        self.config_dir.join("servers")
    }

    fn profile_dir(&self, id: &str) -> PathBuf {
        self.servers_dir().join(id)
    }

    fn now_unix() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Load the index, creating (and persisting) a migrated default on first
    /// run. Never fails to return an index: a corrupt/unreadable
    /// `profiles.json` is treated as absent and rebuilt.
    fn load_index(&self) -> ProfilesIndex {
        match std::fs::read_to_string(self.index_path()) {
            Ok(raw) => match serde_json::from_str::<ProfilesIndex>(&raw) {
                Ok(idx) if !idx.profiles.is_empty() || idx.active.is_some() => idx,
                _ => self.migrate_or_default(),
            },
            Err(_) => self.migrate_or_default(),
        }
    }

    fn migrate_or_default(&self) -> ProfilesIndex {
        let mut idx = ProfilesIndex::default();

        let id = "default".to_string();
        // Adopt the legacy single-server path if it exists; otherwise point the
        // default profile at a fresh profile dir. Either way we never move the
        // legacy file.
        let config_path = if self.legacy_path().exists() {
            self.legacy_path()
        } else {
            self.profile_dir(&id).join("server-config.toml")
        };
        let _ = std::fs::create_dir_all(self.profile_dir(&id));

        idx.profiles.push(ServerProfile {
            id: id.clone(),
            name: "Servidor principal".to_string(),
            config_path: config_path.to_string_lossy().to_string(),
            created_at_unix: Self::now_unix(),
        });
        idx.active = Some(id);
        let _ = self.save_index(&idx);
        idx
    }

    fn save_index(&self, idx: &ProfilesIndex) -> Result<(), String> {
        std::fs::create_dir_all(&self.config_dir)
            .map_err(|e| format!("create {}: {e}", self.config_dir.display()))?;
        let raw = serde_json::to_string_pretty(idx).map_err(|e| e.to_string())?;
        // Atomic-ish write: write to a temp file then rename.
        let tmp = self.index_path().with_extension("json.tmp");
        std::fs::write(&tmp, raw).map_err(|e| format!("write {}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, self.index_path())
            .map_err(|e| format!("rename -> {}: {e}", self.index_path().display()))?;
        Ok(())
    }

    /// All profiles, in creation order.
    pub fn list(&self) -> Vec<ServerProfile> {
        self.load_index().profiles
    }

    /// The active profile id (falls back to the first profile).
    pub fn active_id(&self) -> String {
        let idx = self.load_index();
        if let Some(a) = idx.active {
            if idx.profiles.iter().any(|p| p.id == a) {
                return a;
            }
        }
        idx.profiles.first().map(|p| p.id.clone()).unwrap_or_else(|| "default".into())
    }

    pub fn get(&self, id: &str) -> Option<ServerProfile> {
        self.load_index().profiles.into_iter().find(|p| p.id == id)
    }

    /// Resolve `id` (or the active profile when `None`) to its config path.
    pub fn resolve_config_path(&self, id: Option<&str>) -> Result<PathBuf, String> {
        let idx = self.load_index();
        let wanted = match id {
            Some(i) => i.to_string(),
            None => self.active_id(),
        };
        idx.profiles
            .iter()
            .find(|p| p.id == wanted)
            .map(|p| PathBuf::from(&p.config_path))
            .ok_or_else(|| format!("unknown server profile `{wanted}`"))
    }

    pub fn set_active(&self, id: &str) -> Result<(), String> {
        let mut idx = self.load_index();
        if !idx.profiles.iter().any(|p| p.id == id) {
            return Err(format!("unknown server profile `{id}`"));
        }
        idx.active = Some(id.to_string());
        self.save_index(&idx)
    }

    /// Create a new, empty profile. Returns the created profile (not made
    /// active — callers decide).
    pub fn create(&self, name: &str) -> Result<ServerProfile, String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("profile name cannot be empty".to_string());
        }
        let mut idx = self.load_index();
        let id = Self::unique_id(&idx, name);
        let dir = self.profile_dir(&id);
        std::fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
        let profile = ServerProfile {
            id: id.clone(),
            name: name.to_string(),
            config_path: dir.join("server-config.toml").to_string_lossy().to_string(),
            created_at_unix: Self::now_unix(),
        };
        idx.profiles.push(profile.clone());
        if idx.active.is_none() {
            idx.active = Some(id);
        }
        self.save_index(&idx)?;
        Ok(profile)
    }

    pub fn rename(&self, id: &str, name: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("profile name cannot be empty".to_string());
        }
        let mut idx = self.load_index();
        let p = idx
            .profiles
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| format!("unknown server profile `{id}`"))?;
        p.name = name.to_string();
        self.save_index(&idx)
    }

    /// Remove a profile from the registry. Its folder under `servers/<id>/`
    /// is deleted; a legacy config file outside that folder is left on disk
    /// (never destroyed). The last remaining profile cannot be deleted.
    pub fn delete(&self, id: &str) -> Result<(), String> {
        let mut idx = self.load_index();
        if idx.profiles.len() <= 1 {
            return Err("cannot delete the only server profile".to_string());
        }
        let Some(pos) = idx.profiles.iter().position(|p| p.id == id) else {
            return Err(format!("unknown server profile `{id}`"));
        };
        let removed = idx.profiles.remove(pos);

        // Only delete files we own (inside our `servers/` tree).
        let cfg = PathBuf::from(&removed.config_path);
        if let (Ok(cfg_c), Ok(srv_c)) = (cfg.canonicalize(), self.servers_dir().canonicalize()) {
            if cfg_c.starts_with(&srv_c) {
                if let Some(parent) = cfg_c.parent() {
                    let _ = std::fs::remove_dir_all(parent);
                }
            }
        }
        let _ = std::fs::remove_dir_all(self.profile_dir(&removed.id));

        if idx.active.as_deref() == Some(id) {
            idx.active = idx.profiles.first().map(|p| p.id.clone());
        }
        self.save_index(&idx)
    }

    fn unique_id(idx: &ProfilesIndex, name: &str) -> String {
        let slug: String = name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string();
        let base = if slug.is_empty() { "server".to_string() } else { slug };
        if !idx.profiles.iter().any(|p| p.id == base) {
            return base;
        }
        loop {
            let cand = format!("{}-{}", base, &uuid::Uuid::new_v4().simple().to_string()[..6]);
            if !idx.profiles.iter().any(|p| p.id == cand) {
                return cand;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_store() -> ProfileStore {
        let dir = std::env::temp_dir().join(format!(
            "ark-profile-test-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4().simple()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        ProfileStore::new(dir)
    }

    #[test]
    fn migration_adopts_legacy_config_without_moving_it() {
        let store = tmp_store();
        let legacy = store.legacy_path();
        std::fs::write(&legacy, "x = 1\n").unwrap();

        let profiles = store.list();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].id, "default");
        assert_eq!(profiles[0].config_path, legacy.to_string_lossy());
        assert_eq!(store.active_id(), "default");
        // legacy file untouched
        assert!(legacy.exists());
    }

    #[test]
    fn create_switch_and_delete() {
        let store = tmp_store();
        let p = store.create("PvP Servidor").unwrap();
        assert_eq!(store.list().len(), 2);
        store.set_active(&p.id).unwrap();
        assert_eq!(store.active_id(), p.id);
        assert_eq!(
            store.resolve_config_path(None).unwrap(),
            PathBuf::from(&p.config_path)
        );
        store.delete(&p.id).unwrap();
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.active_id(), "default");
    }

    #[test]
    fn cannot_delete_last_profile() {
        let store = tmp_store();
        assert!(store.delete("default").is_err());
    }
}
