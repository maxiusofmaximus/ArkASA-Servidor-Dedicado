//! Shareable configuration presets.
//!
//! A preset is just a `ServerConfig` written to `<config_dir>/presets/<name>.toml`.
//! Operators can save the current config as a preset, share the file, and load
//! it into another profile/machine — the low-marginal-cost "publish a PvP/PvE
//! template" loop from the research backlog.

use crate::config::loader::ConfigLoader;
use crate::config::schema::ServerConfig;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PresetInfo {
    pub name: String,
    pub path: String,
    pub session_name: String,
}

pub struct PresetStore {
    dir: PathBuf,
}

impl PresetStore {
    pub fn new(config_dir: impl Into<PathBuf>) -> Self {
        Self { dir: config_dir.into().join("presets") }
    }

    pub fn dir(&self) -> PathBuf {
        self.dir.clone()
    }

    /// Reject traversal and normalise to a filesystem-safe stem.
    fn sanitize(name: &str) -> Result<String, String> {
        let n = name.trim();
        if n.is_empty() {
            return Err("preset name cannot be empty".to_string());
        }
        if n.contains("..") || n.contains('/') || n.contains('\\') {
            return Err("invalid preset name".to_string());
        }
        Ok(n.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                    c
                } else {
                    '_'
                }
            })
            .collect())
    }

    fn path(&self, name: &str) -> Result<PathBuf, String> {
        Ok(self.dir.join(format!("{}.toml", Self::sanitize(name)?)))
    }

    pub fn list(&self) -> Vec<PresetInfo> {
        let mut out = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&self.dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|x| x.to_str()) != Some("toml") {
                    continue;
                }
                let name = p.file_stem().and_then(|x| x.to_str()).unwrap_or("").to_string();
                let session_name = std::fs::read_to_string(&p)
                    .ok()
                    .and_then(|raw| toml::from_str::<ServerConfig>(&raw).ok())
                    .map(|c| c.identification.session_name)
                    .unwrap_or_default();
                out.push(PresetInfo {
                    name,
                    path: p.to_string_lossy().to_string(),
                    session_name,
                });
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    pub fn save(&self, name: &str, config: &ServerConfig) -> Result<PresetInfo, String> {
        let safe = Self::sanitize(name)?;
        let path = self.dir.join(format!("{safe}.toml"));
        std::fs::create_dir_all(&self.dir).map_err(|e| format!("create {:?}: {e}", self.dir))?;
        let raw = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
        std::fs::write(&path, raw).map_err(|e| format!("write {:?}: {e}", path))?;
        Ok(PresetInfo {
            name: safe,
            path: path.to_string_lossy().to_string(),
            session_name: config.identification.session_name.clone(),
        })
    }

    pub fn load(&self, name: &str) -> Result<ServerConfig, String> {
        let path = self.path(name)?;
        let raw = std::fs::read_to_string(&path).map_err(|e| format!("read `{name}`: {e}"))?;
        ConfigLoader::load_from_str(&raw).map_err(|e| e.to_string())
    }

    pub fn delete(&self, name: &str) -> Result<(), String> {
        let path = self.path(name)?;
        std::fs::remove_file(&path).map_err(|e| format!("delete `{name}`: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (PresetStore, PathBuf) {
        let dir = std::env::temp_dir().join(format!("ark-preset-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&dir).unwrap();
        (PresetStore::new(&dir), dir)
    }

    #[test]
    fn roundtrip_and_delete() {
        let (s, dir) = store();
        let mut cfg = ServerConfig::default();
        cfg.identification.session_name = "PvP Hardcore".into();
        cfg.gameplay.max_players = 42;

        let info = s.save("pvp hardcore", &cfg).unwrap();
        assert_eq!(info.name, "pvp hardcore");
        assert_eq!(s.list().len(), 1);
        assert_eq!(s.list()[0].session_name, "PvP Hardcore");

        let loaded = s.load("pvp hardcore").unwrap();
        assert_eq!(loaded.gameplay.max_players, 42);

        s.delete("pvp hardcore").unwrap();
        assert!(s.list().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_traversal() {
        let (s, dir) = store();
        assert!(s.save("../evil", &ServerConfig::default()).is_err());
        assert!(s.save("", &ServerConfig::default()).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
