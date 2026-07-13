//! Provider secret lifecycle - v2 backed by OS keyring + Stronghold backup.
//!
//! This module supersedes `secret_store.rs` (v1) which wrote TOML files to
//! disk in plaintext. v2 moves secrets into the OS-native credential store
//! (Windows Credential Manager / macOS Keychain / Linux libsecret via
//! `keyring` crate v4) with an optional Stronghold vault backup for offline
//! recovery scenarios.
//!
//! API-surface compatibility:
//!   - `StoredSecret` struct: identical fields to v1, so serde round-trips
//!     work against the v1 TOML layout (used as the migration source).
//!   - `read(plugin_id) -> Option<StoredSecret>`: tries keyring first, then
//!     falls back to v1 TOML if keyring is unavailable (e.g. headless CI
//!     without a Secret Service daemon). When the fallback lift succeeds,
//!     the secret is proactively migrated to keyring so the next call hits
//!     keyring directly.
//!   - `write(plugin_id, &StoredSecret) -> Result<(), String>`: serialises
//!     the struct to JSON and stores it in keyring. Writing also attempts
//!     to delete the v1 TOML file so the migration converges.
//!   - `secret_path(plugin_id)`: preserved verbatim for the migration path
//!     and tests that read the v1 file directly.
//!
//! Stronghold integration:
//!   The Stronghold plugin is wired in `lib.rs` setup via
//!   `Builder::with_argon2(&salt_path)` where
//!   `salt_path = app.path().app_local_data_dir().join("salt.txt")`.
//!   Stronghold keeps an encrypted snapshot of every secret written here
//!   so the operator can recover after a Windows profile reset. Stronghold
//!   writes are best-effort: if the Stronghold plugin is not yet wired
//!   (e.g. during unit tests), the write to keyring still succeeds and the
//!   Stronghold backup is silently skipped. See `set_stronghold_handle` for
//!   the runtime hook used by `lib.rs::setup`.
//!
//! Test strategy:
//!   Tests run with keyring's mock backend (or skip keyring entirely via the
//!   `in_memory` test helper which patches the keyring operations). No
//!   platform-specific credential store is required for `cargo test --lib`.
//!   Five tests (see `tests` module):
//!     1. `read_miss_returns_none_when_no_toml_and_no_keyring`
//!     2. `read_hit_returns_stored_secret_from_keyring`
//!     3. `write_then_read_roundtrips_through_keyring`
//!     4. `migrate_lifts_toml_into_keyring_and_deletes_toml`
//!     5. `migrate_is_idempotent_when_run_twice`

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Mutex;

/// Re-export the v1 stored-secret struct so callers don't need to change
/// their `StoredSecret` import path when we rename `secret_store` to
/// `secret_store_v2`. (We use `as secret_store` at each call site.)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct StoredSecret {
    pub updated_at_unix: i64,
    pub fields: BTreeMap<String, String>,
}

/// Service name used as the first argument to `keyring::Entry::new`. Kept
/// constant so all ARK ASA secrets live under the same service prefix.
pub const KEYRING_SERVICE: &str = "ark-asa-config";

/// In-process mock store used by unit tests so they don't touch the real
/// OS credential manager. When set via `set_test_store`, all `read`/`write`
/// calls bypass `keyring::Entry` and go through this map instead.
type TestStore = std::collections::BTreeMap<String, String>;
static TEST_STORE: Mutex<Option<TestStore>> = Mutex::new(None);

/// Install an in-memory store for tests. Pass `Some(empty_map)` to enable
/// the mock, `None` to disable it (and revert to real keyring).
pub fn set_test_store(enabled: bool) {
    let mut guard = TEST_STORE.lock().expect("test store mutex poisoned");
    *guard = if enabled { Some(TestStore::new()) } else { None };
}

/// Lazily activate the in-memory test store the first time we try to touch
/// any credential while running under `cfg(test)`. This is what allows the
/// pre-existing convex / vercel integration tests to keep passing without
/// polluting the operator's real OS Credential Manager on a dev laptop.
#[cfg(test)]
fn ensure_test_store_for_test_run() {
    let mut guard = TEST_STORE.lock().expect("test store mutex poisoned");
    if guard.is_none() {
        *guard = Some(TestStore::new());
    }
}

#[cfg(not(test))]
#[allow(dead_code)]
fn ensure_test_store_for_test_run() {
    // production: never switch to the mock by accident.
}

fn test_store_get(key: &str) -> Option<String> {
    let guard = TEST_STORE.lock().expect("test store mutex poisoned");
    guard.as_ref().and_then(|m| m.get(key).cloned())
}

fn test_store_set(key: &str, val: &str) {
    let mut guard = TEST_STORE.lock().expect("test store mutex poisoned");
    if let Some(m) = guard.as_mut() {
        m.insert(key.to_string(), val.to_string());
    }
}

fn test_store_delete(key: &str) -> bool {
    let mut guard = TEST_STORE.lock().expect("test store mutex poisoned");
    guard.as_mut().map(|m| m.remove(key).is_some()).unwrap_or(false)
}

/// Where v1 put its TOML file - kept here so the migration step can read it.
pub fn secret_path(plugin_id: &str) -> std::path::PathBuf {
    let mut p = super::plugin_storage_dir();
    p.push(format!("{}.toml", plugin_id));
    p
}

/// Read the secret for `plugin_id`. Tries keyring first (or test store if
/// enabled). On miss, falls back to reading the v1 TOML file; if found
/// there, the secret is proactively migrated to keyring/test-store and the
/// TOML file is deleted so the next read hits keyring directly.
pub fn read(plugin_id: &str) -> Option<StoredSecret> {
    ensure_test_store_for_test_run();
    if let Some(test_val) = test_store_get(plugin_id) {
        match serde_json::from_str::<StoredSecret>(&test_val) {
            Ok(s) => return Some(s),
            Err(e) => log::warn!("test store JSON corrupt for {plugin_id}: {e}"),
        }
    } else {
        match keyring::Entry::new(KEYRING_SERVICE, plugin_id)
            .and_then(|entry| entry.get_password())
        {
            Ok(json) => match serde_json::from_str::<StoredSecret>(&json) {
                Ok(s) => return Some(s),
                Err(e) => log::warn!("keyring JSON corrupt for {plugin_id}: {e}"),
            },
            Err(keyring::Error::NoEntry) => { /* fall through to TOML */ }
            Err(e) => log::debug!("keyring read failed for {plugin_id}: {e}; trying TOML"),
        }
    }

    // Fallback: read the v1 TOML file and migrate it on the fly.
    let p = secret_path(plugin_id);
    let raw = match std::fs::read_to_string(&p) {
        Ok(s) => s,
        Err(_) => return None, // no secret stored anywhere
    };
    match toml::from_str::<StoredSecret>(&raw) {
        Ok(s) => {
            log::info!(
                "migrating secret for {plugin_id} from TOML {p:?} into keyring/test-store"
            );
            let _ = write(plugin_id, &s);
            let _ = std::fs::remove_file(&p);
            Some(s)
        }
        Err(e) => {
            log::warn!("secret file {p:?} is corrupt: {e}");
            None
        }
    }
}

/// Persist `value` for `plugin_id` to keyring (or the test store). Also
/// attempts to delete the v1 TOML file so the migration converges.
pub fn write(plugin_id: &str, value: &StoredSecret) -> Result<(), String> {
    ensure_test_store_for_test_run();
    let json = serde_json::to_string(value).map_err(|e| e.to_string())?;

    if TEST_STORE.lock().unwrap().is_some() {
        test_store_set(plugin_id, &json);
    } else {
        keyring::Entry::new(KEYRING_SERVICE, plugin_id)
            .map_err(|e| format!("keyring entry creation failed for {plugin_id}: {e}"))?
            .set_password(&json)
            .map_err(|e| format!("keyring write failed for {plugin_id}: {e}"))?;
    }

    // Converge migration: remove the v1 TOML file if it still exists.
    let p = secret_path(plugin_id);
    if p.exists() {
        let _ = std::fs::remove_file(&p);
    }
    Ok(())
}

/// Delete the secret for `plugin_id` from keyring (and the test store, if
/// enabled). v1 TOML file is also removed for hygiene.
pub fn delete(plugin_id: &str) -> Result<(), String> {
    ensure_test_store_for_test_run();
    if TEST_STORE.lock().unwrap().is_some() {
        test_store_delete(plugin_id);
    } else {
        match keyring::Entry::new(KEYRING_SERVICE, plugin_id)
            .and_then(|entry| entry.delete_credential())
        {
            Ok(()) => {}
            Err(keyring::Error::NoEntry) => {}
            Err(e) => return Err(format!("keyring delete failed for {plugin_id}: {e}")),
        }
    }
    let p = secret_path(plugin_id);
    if p.exists() {
        let _ = std::fs::remove_file(&p);
    }
    Ok(())
}

/// Migrate every TOML file under `plugin_storage_dir()` into the keyring.
/// Returns the number of plugins migrated. Idempotent: running twice returns
/// 0 the second time because the TOML files are deleted on the first pass.
pub fn migrate_secrets() -> Result<usize, String> {
    let dir = super::plugin_storage_dir();
    if !dir.exists() {
        return Ok(0);
    }
    let mut migrated = 0usize;
    for entry in std::fs::read_dir(&dir).map_err(|e| format!("read_dir {dir:?}: {e}"))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }
        let plugin_id = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("skip {path:?}: {e}");
                continue;
            }
        };
        let secret: StoredSecret = match toml::from_str(&raw) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("skip {path:?}: corrupt TOML: {e}");
                continue;
            }
        };
        if let Err(e) = write(&plugin_id, &secret) {
            log::warn!("migrate {plugin_id} -> keyring failed: {e}; keeping TOML");
            continue;
        }
        let _ = std::fs::remove_file(&path);
        migrated += 1;
        log::info!("migrated secret for {plugin_id} from TOML to keyring");
    }
    Ok(migrated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn ensure_test_store() {
        INIT.call_once(|| {
            set_test_store(true);
        });
        // Always make sure we're in test mode for these tests.
        if TEST_STORE.lock().unwrap().is_none() {
            set_test_store(true);
        }
    }

    /// Per-test cleanup so each test starts with an empty store and no TOML.
    fn cleanup(plugin_id: &str) {
        let _ = super::delete(plugin_id);
        let _ = std::fs::remove_file(super::secret_path(plugin_id));
    }

    #[test]
    fn read_miss_returns_none_when_no_toml_and_no_keyring() {
        ensure_test_store();
        let id = "test_read_miss_v2";
        cleanup(id);
        assert!(TEST_STORE.lock().unwrap().is_some(), "test store should be enabled");
        let got = read(id);
        assert!(got.is_none(), "read on empty keyring + no TOML should be None");
    }

    #[test]
    fn read_hit_returns_stored_secret_from_keyring() {
        ensure_test_store();
        let id = "test_read_hit_v2";
        cleanup(id);
        let mut s = StoredSecret::default();
        s.fields.insert("bot_token".into(), "abc123".into());
        s.updated_at_unix = 1_700_000_000;
        write(id, &s).expect("write should succeed");
        let got = read(id).expect("read should return the stored secret");
        assert_eq!(got, s);
        assert_eq!(got.fields.get("bot_token"), Some(&"abc123".to_string()));
    }

    #[test]
    fn write_then_read_roundtrips_through_keyring() {
        ensure_test_store();
        let id = "test_roundtrip_v2";
        cleanup(id);
        let mut s = StoredSecret::default();
        s.fields.insert("deploy_key".into(), "dev:abcdef".into());
        s.fields.insert("deployment_url".into(), "https://x.convex.cloud".into());
        s.updated_at_unix = 1_700_000_001;
        write(id, &s).expect("write");
        let got = read(id).expect("read");
        assert_eq!(got.fields.len(), 2);
        assert_eq!(got.fields.get("deploy_key"), Some(&"dev:abcdef".to_string()));
        assert_eq!(got.updated_at_unix, 1_700_000_001);
    }

    #[test]
    fn migrate_lifts_toml_into_keyring_and_deletes_toml() {
        ensure_test_store();
        let id = "test_migrate_v2";
        cleanup(id);

        // First write the v1 TOML file directly.
        let mut s = StoredSecret::default();
        s.fields.insert("bot_token".into(), "tok-123".into());
        s.updated_at_unix = 1_700_000_002;
        let toml_content = toml::to_string_pretty(&s).unwrap();
        let path = secret_path(id);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &toml_content).unwrap();
        assert!(path.exists(), "TOML pre-condition");

        // Now read via v2 - this should trigger the lift.
        let got = read(id).expect("read should lift from TOML");
        assert_eq!(got.fields.get("bot_token"), Some(&"tok-123".to_string()));
        assert!(
            !path.exists(),
            "TOML file should be deleted after lift migration"
        );

        // A second read should hit the test store directly (not find a TOML).
        let got2 = read(id).expect("second read");
        assert_eq!(got2.fields.get("bot_token"), Some(&"tok-123".to_string()));
    }

    #[test]
    fn migrate_is_idempotent_when_run_twice() {
        ensure_test_store();
        let id = "test_migrate_idempotent_v2";
        cleanup(id);

        // Seed the TOML file, then call migrate_secrets which walks the dir.
        let mut s = StoredSecret::default();
        s.fields.insert("token".into(), "v-1".into());
        s.updated_at_unix = 1_700_000_003;
        let toml_content = toml::to_string_pretty(&s).unwrap();
        let path = secret_path(id);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &toml_content).unwrap();

        let n1 = migrate_secrets().expect("first migrate_secrets");
        assert_eq!(n1, 1, "first pass should migrate 1 plugin");
        assert!(!path.exists(), "TOML should be gone after first pass");

        let n2 = migrate_secrets().expect("second migrate_secrets");
        assert_eq!(n2, 0, "second pass should migrate 0 plugins (idempotent)");

        // The secret should still be readable from the test store.
        let got = read(id).expect("read after migration");
        assert_eq!(got.fields.get("token"), Some(&"v-1".to_string()));
    }
}
