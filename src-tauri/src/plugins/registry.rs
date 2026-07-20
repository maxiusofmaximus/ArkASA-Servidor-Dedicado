//! Plugin registry state — a single TOML file that records which
//! plugin ids the operator has enabled through the Plugin Hub UI.
//!
//! Lives next to the existing `secret_store_v2` (OS keyring-per-plugin).
//! The PluginRegistry at startup reads this file, walks the catalog
//! from `plugins/mod.rs::register_default_plugins`, and calls
//! `enable(id)` for each id present here. The UI writes here when
//! the operator toggles a plugin on/off without recompiling.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegistryFile {
    /// Enabled plugin ids. Persisted across restarts so operator
    /// choices survive.
    pub enabled: BTreeSet<String>,
    /// Disabled explicit overrides — ids the operator explicitly
    /// turned off (vs. defaults). Useful for plugins we want to
    /// *not* auto-start in v2.1.
    pub disabled: BTreeSet<String>,
}

fn registry_path() -> PathBuf {
    let mut p = super::plugin_storage_dir();
    p.push("registry.toml");
    p
}

pub fn read() -> RegistryFile {
    let p = registry_path();
    let raw = match std::fs::read_to_string(&p) {
        Ok(s) => s,
        Err(_) => return RegistryFile::default(),
    };
    match toml::from_str(&raw) {
        Ok(f) => f,
        Err(e) => {
            log::warn!("registry.toml is corrupt: {e}; defaulting to empty");
            RegistryFile::default()
        }
    }
}

/// Atomic write — write to tmp, rename to registry_path.
pub fn write(file: &RegistryFile) -> Result<(), String> {
    let p = registry_path();
    let dir = p.parent().ok_or("registry path missing parent")?;
    std::fs::create_dir_all(dir).map_err(|e| format!("create {dir:?}: {e}"))?;
    let tmp = dir.join(".registry.tmp");
    let serialised = toml::to_string_pretty(file).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, serialised).map_err(|e| format!("write tmp: {e}"))?;
    std::fs::rename(&tmp, &p).map_err(|e| format!("rename: {e}"))?;
    Ok(())
}

/// Convenience: enable plugin `id`, persist the file, return the
/// new RegistryFile.
pub fn enable_id(id: &str) -> Result<RegistryFile, String> {
    let mut f = read();
    f.disabled.remove(id);
    f.enabled.insert(id.to_string());
    write(&f)?;
    Ok(f)
}

/// Disable plugin `id`, persist, return the new file.
pub fn disable_id(id: &str) -> Result<RegistryFile, String> {
    let mut f = read();
    f.enabled.remove(id);
    f.disabled.insert(id.to_string());
    write(&f)?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enable_and_disable_roundtrip() {
        let _storage_guard = crate::plugins::lock_plugin_storage_for_test();
        // Use a per-test temp dir to avoid polluting global state.
        let id = "test_plugin_roundtrip";
        // Ensure clean baseline
        let mut before = read();
        before.enabled.remove(id);
        before.disabled.remove(id);
        let _ = write(&before);

        // Toggle on/off.
        let f = enable_id(id).unwrap();
        assert!(f.enabled.contains(id), "enable_id should add to enabled set");
        let f = disable_id(id).unwrap();
        assert!(f.disabled.contains(id), "disable_id should add to disabled set");
        assert!(!f.enabled.contains(id), "disable_id should remove from enabled");

        // Clean up
        before.enabled.remove(id);
        before.disabled.remove(id);
        let _ = write(&before);
    }

    #[test]
    fn read_corrupt_returns_default() {
        let _storage_guard = crate::plugins::lock_plugin_storage_for_test();
        // Force a corrupt file and check we don't panic.
        let p = registry_path();
        let original = std::fs::read(&p).ok();
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let garbage: Vec<u8> = vec![0xFFu8, 0xFE, 0x00, 0xFF, 0xAB];
        std::fs::write(&p, &garbage).unwrap();
        let f = read();
        assert!(f.enabled.is_empty());
        assert!(f.disabled.is_empty());

        // Tests share the operator's storage location in desktop builds;
        // restore the exact pre-test bytes instead of deleting user state.
        match original {
            Some(bytes) => std::fs::write(&p, bytes).unwrap(),
            None => { let _ = std::fs::remove_file(&p); }
        }
    }
}
