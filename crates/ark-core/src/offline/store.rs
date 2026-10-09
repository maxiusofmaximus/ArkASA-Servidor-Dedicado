//! The local materialised state, owned by [`redb`].
//!
//! This is the single source of truth the UI reads. It is populated from the
//! filesystem on first run (the TOML/INI the core already manages) and updated
//! whenever the core writes, so the UI never blocks on I/O to render.

use std::path::{Path, PathBuf};

use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};

use crate::error::{Error, Result};

/// Profile rows, keyed by the profile id, JSON-encoded for forward
/// compatibility: adding a field to the struct does not require a migration.
const PROFILES: TableDefinition<&str, &[u8]> = TableDefinition::new("profiles");

/// Arbitrary key/value metadata (schema version, last sync, …).
const META: TableDefinition<&str, &str> = TableDefinition::new("meta");

const META_SCHEMA_VERSION: &str = "schema_version";
const CURRENT_SCHEMA_VERSION: &str = "1";

/// A profile as stored offline.
///
/// Mirrors the fields the UI needs. Kept as a plain serialisable struct rather
/// than reusing the config type so the store's shape can evolve independently.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StoredProfile {
    pub id: String,
    pub name: String,
    pub config_path: String,
    pub created_at_unix: u64,
    pub is_active: bool,
}

/// The offline store: profile state plus a small metadata table.
pub struct OfflineStore {
    db: Database,
    path: PathBuf,
}

impl OfflineStore {
    /// Opens (or creates) the store inside `dir`, as `<dir>/offline.redb`.
    pub fn open(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)
            .map_err(|e| Error::ConfigError(format!("cannot create {}: {e}", dir.display())))?;
        let path = dir.join("offline.redb");
        let db = Database::create(&path)
            .map_err(|e| Error::ConfigError(format!("cannot open {}: {e}", path.display())))?;

        let store = Self { db, path };
        store.ensure_schema()?;
        Ok(store)
    }

    /// Absolute path of the backing file (useful for diagnostics).
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn ensure_schema(&self) -> Result<()> {
        let txn = self
            .db
            .begin_write()
            .map_err(|e| Error::ConfigError(format!("store write txn: {e}")))?;
        {
            let _ = txn
                .open_table(PROFILES)
                .map_err(|e| Error::ConfigError(format!("open profiles table: {e}")))?;
            let mut meta = txn
                .open_table(META)
                .map_err(|e| Error::ConfigError(format!("open meta table: {e}")))?;
            meta.insert(META_SCHEMA_VERSION, CURRENT_SCHEMA_VERSION)
                .map_err(|e| Error::ConfigError(format!("write schema version: {e}")))?;
        }
        txn.commit()
            .map_err(|e| Error::ConfigError(format!("store commit: {e}")))?;
        Ok(())
    }

    /// Replaces the whole profile set in one transaction.
    ///
    /// "Whole set" rather than per-row upsert: the profile list is small and
    /// always written from one source, so a full replace cannot leave the store
    /// disagreeing with the filesystem.
    pub fn replace_profiles(&self, profiles: &[StoredProfile]) -> Result<()> {
        let txn = self
            .db
            .begin_write()
            .map_err(|e| Error::ConfigError(format!("store write txn: {e}")))?;
        {
            let mut table = txn
                .open_table(PROFILES)
                .map_err(|e| Error::ConfigError(format!("open profiles table: {e}")))?;

            // Drain first: a profile that disappeared from the filesystem must
            // not survive in the store.
            let existing: Vec<String> = {
                let readable = table
                    .iter()
                    .map_err(|e| Error::ConfigError(format!("iterate profiles: {e}")))?
                    .map(|row| {
                        row.map(|(k, _)| k.value().to_owned())
                            .map_err(|e| Error::ConfigError(format!("read profile key: {e}")))
                    })
                    .collect::<Result<Vec<_>>>()?;
                readable
            };
            for key in existing {
                table
                    .remove(key.as_str())
                    .map_err(|e| Error::ConfigError(format!("remove profile {key}: {e}")))?;
            }

            for profile in profiles {
                let bytes = serde_json::to_vec(profile)
                    .map_err(|e| Error::ConfigError(format!("serialise profile {}: {e}", profile.id)))?;
                table
                    .insert(profile.id.as_str(), bytes.as_slice())
                    .map_err(|e| Error::ConfigError(format!("insert profile {}: {e}", profile.id)))?;
            }
        }
        txn.commit()
            .map_err(|e| Error::ConfigError(format!("store commit: {e}")))?;
        Ok(())
    }

    /// All stored profiles, ordered by id for a stable UI list.
    pub fn profiles(&self) -> Result<Vec<StoredProfile>> {
        let txn = self
            .db
            .begin_read()
            .map_err(|e| Error::ConfigError(format!("store read txn: {e}")))?;

        let table = match txn.open_table(PROFILES) {
            Ok(t) => t,
            // Table never written yet: legitimately empty, not an error.
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
            Err(e) => return Err(Error::ConfigError(format!("open profiles table: {e}"))),
        };

        let mut out = Vec::new();
        for row in table
            .iter()
            .map_err(|e| Error::ConfigError(format!("iterate profiles: {e}")))?
        {
            let (_, value) = row.map_err(|e| Error::ConfigError(format!("read profile row: {e}")))?;
            let profile: StoredProfile = serde_json::from_slice(value.value())
                .map_err(|e| Error::ConfigError(format!("decode stored profile: {e}")))?;
            out.push(profile);
        }
        out.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(out)
    }

    /// The active profile id, if any.
    pub fn active_profile_id(&self) -> Result<Option<String>> {
        Ok(self
            .profiles()?
            .into_iter()
            .find(|p| p.is_active)
            .map(|p| p.id))
    }

    /// Number of stored profiles (used by tests and diagnostics).
    pub fn profile_count(&self) -> Result<u64> {
        let txn = self
            .db
            .begin_read()
            .map_err(|e| Error::ConfigError(format!("store read txn: {e}")))?;
        match txn.open_table(PROFILES) {
            Ok(t) => t
                .len()
                .map_err(|e| Error::ConfigError(format!("count profiles: {e}"))),
            Err(redb::TableError::TableDoesNotExist(_)) => Ok(0),
            Err(e) => Err(Error::ConfigError(format!("open profiles table: {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(id: &str, active: bool) -> StoredProfile {
        StoredProfile {
            id: id.into(),
            name: format!("name-{id}"),
            config_path: format!("/cfg/{id}/server-config.toml"),
            created_at_unix: 1_700_000_000,
            is_active: active,
        }
    }

    #[test]
    fn opens_a_fresh_store_empty() {
        let dir = tempfile::tempdir().unwrap();
        let store = OfflineStore::open(dir.path()).unwrap();
        assert_eq!(store.profile_count().unwrap(), 0);
        assert!(store.profiles().unwrap().is_empty());
        assert!(store.active_profile_id().unwrap().is_none());
    }

    #[test]
    fn stores_and_reads_profiles_back() {
        let dir = tempfile::tempdir().unwrap();
        let store = OfflineStore::open(dir.path()).unwrap();
        store
            .replace_profiles(&[profile("b", false), profile("a", true)])
            .unwrap();

        let profiles = store.profiles().unwrap();
        assert_eq!(profiles.len(), 2);
        // Sorted by id, not insertion order.
        assert_eq!(profiles[0].id, "a");
        assert_eq!(profiles[1].id, "b");
        assert_eq!(profiles[0].name, "name-a");
        assert_eq!(store.active_profile_id().unwrap().as_deref(), Some("a"));
    }

    #[test]
    fn replace_is_not_additive_it_removes_vanished_profiles() {
        let dir = tempfile::tempdir().unwrap();
        let store = OfflineStore::open(dir.path()).unwrap();
        store
            .replace_profiles(&[profile("a", true), profile("b", false)])
            .unwrap();
        // "b" is deleted on disk; the store must not keep lying about it.
        store.replace_profiles(&[profile("a", true)]).unwrap();

        let profiles = store.profiles().unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].id, "a");
    }

    #[test]
    fn survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        {
            let store = OfflineStore::open(dir.path()).unwrap();
            store.replace_profiles(&[profile("a", true)]).unwrap();
        }
        // A second open must see the persisted data (durability).
        let reopened = OfflineStore::open(dir.path()).unwrap();
        assert_eq!(reopened.profile_count().unwrap(), 1);
        assert_eq!(reopened.active_profile_id().unwrap().as_deref(), Some("a"));
    }
}
