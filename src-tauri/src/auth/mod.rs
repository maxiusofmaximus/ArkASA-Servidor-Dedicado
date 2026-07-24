//! JWT-style auth for the v2.1 loopback HTTP API.
//!
//! The HS256 signing secret and the latest bearer token are persisted in the
//! OS-native keyring (Windows Credential Manager / macOS Keychain / Linux
//! libsecret) via `secret_store_v2`. The legacy plaintext files
//! `~/.ark-asa/admin.jwt` and `~/.ark-asa/admin.token` were removed in GA-1
//! (formerly P8); the loader still tolerates them for one release as a
//! migration fallback — on first boot after upgrading we lift whatever is
//! still on disk into the keyring and delete the plaintext files so the
//! leaves-on-disk attack surface shrinks on the very next launch.
//!
//! On a brand-new install the suite generates a 256-bit random secret,
//! derives the initial admin token from it, and writes both into the
//! keyring. The token is then displayed in `Options → Remote Admin` so the
//! operator can copy it.
//!
//! Authorization roles are intentionally NOT in the JWT: we want the JWT
//! signing secret to remain stable across role changes. The JWT only proves
//! the caller is the desktop app itself; RBAC (admin vs viewer) is enforced
//! at the channel boundary (Convex tier, Discord allowlist, Telegram user-id
//! allowlist, etc.) and mapped to the JWT role at the Convex actions in Hito 3.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::plugins::secret_store_v2::{self as secret_store, StoredSecret};

/// Legacy plaintext filenames in `<storage_dir()`; kept only for the
/// one-shot migration fallback in GA-1. After this release the loader still
/// recognises them but immediately deletes them after lifting to the
/// keyring. Plan: removed in 2.1.1 once telemetry confirms no flock of
/// operators is stuck on a broken keyring backend (headless etc.).
const SECRET_FILENAME: &str = "admin.jwt";
const TOKEN_FILENAME:  &str = "admin.token";

/// Keyring slot id for the (secret, token) pair. Single entry per concern —
/// arbitrary attached plugins share the service prefix `ark-asa-config` but
/// each concern gets its own entry. P8 / GA-1.
const KEYRING_PLUGIN_ID: &str = "auth_admin_v2";

thread_local! {
    /// Per-test thread-local override of the keyring plugin id, used only
    /// in `cfg(test)` builds so parallel `cargo test` runners don't collide
    /// on the shared `secret_store_v2` in-memory map. Production code never
    /// touches this; it always resolves to `KEYRING_PLUGIN_ID`.
    static TEST_KEYRING_OVERRIDE: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}

/// Resolve the keyring plugin id at runtime. Production code uses the
/// `KEYRING_PLUGIN_ID` constant verbatim; tests pin a per-thread slot via
/// `set_thread_test_keyring_id` so their state is fully isolated even
/// under parallel `cargo test` runners.
fn effective_keyring_id() -> std::borrow::Cow<'static, str> {
    #[cfg(test)]
    {
        if let Some(id) = TEST_KEYRING_OVERRIDE.with(|c| c.borrow().clone()) {
            return std::borrow::Cow::Owned(id);
        }
    }
    std::borrow::Cow::Borrowed(KEYRING_PLUGIN_ID)
}

/// Field names stored inside the keyring `StoredSecret.fields` map.
const FIELD_SECRET_BASE64: &str = "secret_b64"; // base64url(HS256 secret bytes)
const FIELD_TOKEN:        &str = "token";       // the latest bearer token

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // "tauri-app"
    pub exp: i64,       // unix epoch seconds
    pub iat: i64,
    pub role: Role,
    /// Human-readable label, eg. "desktop" — purely for audit.
    pub label: String,
}

#[derive(Clone)]
pub struct AuthState {
    secret: Vec<u8>,
    token:  String,
    claims: Claims,
}

impl AuthState {
    /// Path to `~/.ark-asa/`. Caller is expected to have created it.
    pub fn storage_dir() -> PathBuf {
        let mut p = dirs_home();
        p.push(".ark-asa");
        p
    }

    /// Load (or first-time-create) the auth state. Persists the secret + token
    /// in the OS keyring via `secret_store_v2` (GA-1, formerly P8). The
    /// legacy plaintext files `admin.jwt` + `admin.token` are tolerated on a
    /// single pass: when found they are lifted into the keyring and the
    /// files are deleted so the next launch reads only the keyring.
    pub async fn load_or_init() -> Result<Self, String> {
        // Ensure the legacy dir still exists so the migration preamble can
        // probe both filenames; on a clean install this is a no-op.
        let dir = Self::storage_dir();
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| format!("mkdir {dir:?}: {e}"))?;

        let (secret, token) = match Self::read_keyring().await? {
            Some((s, t)) => (s, t),
            None => {
                // Keyring empty (or corrupt). Try the legacy plaintext once,
                // then fall back to generating a fresh pair.
                match Self::read_legacy_plaintext(&dir).await? {
                    Some((s, t)) => {
                        log::info!(
                            "auth: lifted legacy admin.jwt+admin.token from {dir:?} into keyring"
                        );
                        let secret = s;
                        let token = t;
                        Self::write_keyring(&secret, &token).await?;
                        // One-shot lift: delete the plaintext files we just
                        // migrated so the leaves-on-disk attack surface
                        // shrinks on the very next launch.
                        if let Err(e) = tokio::fs::remove_file(dir.join(SECRET_FILENAME)).await {
                            log::warn!("auth: could not delete legacy {SECRET_FILENAME}: {e}");
                        }
                        if let Err(e) = tokio::fs::remove_file(dir.join(TOKEN_FILENAME)).await {
                            log::warn!("auth: could not delete legacy {TOKEN_FILENAME}: {e}");
                        }
                        (secret, token)
                    }
                    None => {
                        let secret = Self::fresh_secret();
                        let token = Self::derive_initial_token(&secret);
                        Self::write_keyring(&secret, &token).await?;
                        (secret, token)
                    }
                }
            }
        };

        if secret.len() < 32 {
            return Err("stored secret is too short".into());
        }

        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: "tauri-app".into(),
            iat: now,
            exp: (Utc::now() + Duration::days(30)).timestamp(),
            role: Role::Admin,
            label: "desktop".into(),
        };

        Ok(Self { secret, token, claims })
    }

    /// Re-issue the bearer token, rotating the secret. Persists the new state
    /// to the keyring (and, for one release, also writes the legacy
    /// plaintext files so a downgrade round-trip still works).
    pub async fn rotate(&mut self) -> Result<&str, String> {
        let new_secret = Self::fresh_secret();
        self.secret = new_secret.clone();

        let token = Self::derive_rotated_token(&new_secret);
        self.token = token.clone();

        Self::write_keyring(&self.secret, &self.token).await?;
        // Belt-and-braces: cover the (unexpected) downgrade path.
        let dir = Self::storage_dir();
        tokio::fs::create_dir_all(&dir).await.ok();
        let _ = tokio::fs::write(dir.join(SECRET_FILENAME), &self.secret).await;
        let _ = tokio::fs::write(dir.join(TOKEN_FILENAME), &self.token).await;

        Ok(&self.token)
    }

    pub fn active_token(&self) -> &str { &self.token }

    /// Build a signed JWT for outbound HTTP calls (eg. Convex actions
    /// calling back into the loopback API).
    pub fn sign_jwt(&self, role: Role) -> Result<String, String> {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: "tauri-app".into(),
            iat: now,
            exp: (Utc::now() + Duration::days(30)).timestamp(),
            role,
            label: "desktop".into(),
        };
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(&self.secret),
        ).map_err(|e| format!("jwt encode: {e}"))
    }

    /// Validate an inbound `Bearer <token>` header value. The header value
    /// may be either the raw bearer token string OR a JWT issued by `sign_jwt`.
    /// Returns just the role for callers that don't need identity claims.
    pub fn validate(&self, header_value: &str) -> Result<Role, String> {
        self.validate_with_claims(header_value).map(|c| c.role)
    }

    /// Validate and return the full identity claims. Use this when the
    /// caller wants to bind an inbound request to a `RemoteCommandContext`
    /// identity (so the receipts ledger can record actor info, not just
    /// "http-api"). Raw bearer tokens return a synthetic `Claims { sub:
    /// "tauri-app", label: "operator", role: Admin }` so the downstream
    /// can rely on the field being non-empty.
    pub fn validate_with_claims(&self, header_value: &str) -> Result<Claims, String> {
        let stripped = header_value.strip_prefix("Bearer ").unwrap_or(header_value);
        let trimmed = stripped.trim();

        // Fast path: raw bearer token (the canonical form operators copy).
        // We DO have stored self-claims for this case (the bootstrap
        // Claims), so we hand those back to the caller.
        if trimmed == self.token {
            return Ok(self.claims.clone());
        }

        // Slow path: signed JWT (used by Convex backend, sub-bots, etc.).
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 30; // seconds

        match decode::<Claims>(trimmed, &DecodingKey::from_secret(&self.secret), &validation) {
            Ok(data) => Ok(data.claims),
            Err(e)   => Err(format!("auth rejected: {e}")),
        }
    }

    // ─── keyring + legacy-lift helpers (GA-1 / P8) ──────────────────────

    /// Read the (secret, token) pair out of the keyring. Returns `Ok(None)`
    /// when no keyring entry exists. The `secret_store_v2::read` call is
    /// synchronous (it talks to the OS Credential Manager), so it is
    /// off-loaded to `spawn_blocking` to avoid stalling the tokio runtime.
    /// The keyring slot id is captured into the closure so the test-only
    /// thread-local override still applies inside the blocking worker.
    async fn read_keyring() -> Result<Option<(Vec<u8>, String)>, String> {
        let slot = effective_keyring_id().into_owned();
        tokio::task::spawn_blocking(move || -> Result<Option<(Vec<u8>, String)>, String> {
            let Some(stored) = secret_store::read(&slot) else {
                return Ok(None);
            };
            let token = match stored.fields.get(FIELD_TOKEN) {
                Some(t) if !t.is_empty() => t.clone(),
                _ => return Ok(None),
            };
            let b64 = match stored.fields.get(FIELD_SECRET_BASE64) {
                Some(s) => s.clone(),
                None => return Ok(None),
            };
            let bytes = URL_SAFE_NO_PAD
                .decode(b64.as_bytes())
                .map_err(|e| format!("admin secret b64 decode: {e}"))?;
            Ok(Some((bytes, token)))
        })
        .await
        .map_err(|e| format!("keyring join: {e}"))?
    }

    /// One-shot legacy lift: decode `admin.jwt` (raw bytes) + `admin.token`
    /// (string) so the keyring write can replace the plaintext pair. The
    /// delete happens in the caller, after the keyring write succeeds, so a
    /// crash mid-migration cannot lose the only copy of the secret.
    async fn read_legacy_plaintext(dir: &PathBuf) -> Result<Option<(Vec<u8>, String)>, String> {
        let dir = dir.clone();
        tokio::task::spawn_blocking(move || -> Result<Option<(Vec<u8>, String)>, String> {
            let secret_path = dir.join(SECRET_FILENAME);
            let token_path  = dir.join(TOKEN_FILENAME);
            if !secret_path.exists() || !token_path.exists() {
                return Ok(None);
            }
            let secret = std::fs::read(&secret_path)
                .map_err(|e| format!("read legacy secret: {e}"))?;
            let token  = std::fs::read_to_string(&token_path)
                .map_err(|e| format!("read legacy token: {e}"))?;
            let token  = token.trim().to_string();
            if token.is_empty() {
                return Ok(None);
            }
            Ok(Some((secret, token)))
        })
        .await
        .map_err(|e| format!("legacy lift join: {e}"))?
    }

    /// Write the (secret, token) pair into the keyring slot, wrapping the
    /// synchronous `keyring` call in `spawn_blocking`. The slot id is
    /// captured into the worker so test-only thread-local overrides apply.
    async fn write_keyring(secret: &[u8], token: &str) -> Result<(), String> {
        let b64 = URL_SAFE_NO_PAD.encode(secret);
        let token = token.to_string();
        let slot = effective_keyring_id().into_owned();
        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let mut fields = std::collections::BTreeMap::new();
            fields.insert(FIELD_SECRET_BASE64.to_string(), b64);
            fields.insert(FIELD_TOKEN.to_string(), token);
            let stored = StoredSecret {
                updated_at_unix: Utc::now().timestamp(),
                fields,
            };
            secret_store::write(&slot, &stored)
        })
        .await
        .map_err(|e| format!("keyring write join: {e}"))?
    }

    /// Random 32-byte HS256 secret builder.
    fn fresh_secret() -> Vec<u8> {
        let mut s = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut s);
        s
    }

    /// Derive the *initial* bearer token from the freshly-minted secret so
    /// we can re-issue it on demand without needing a second secret source.
    /// 32 bytes → base64url(24 bytes) keeps the on-the-wire token compact.
    fn derive_initial_token(secret: &[u8]) -> String {
        Self::derive_token(secret, b"v2.1-initial-token")
    }

    /// Derive a rotated token after `rotate()` rolling a new secret.
    fn derive_rotated_token(secret: &[u8]) -> String {
        Self::derive_token(secret, b"v2.1-rotated-token")
    }

    fn derive_token(secret: &[u8], domain: &[u8]) -> String {
        let mut buf = [0u8; 24];
        {
            use sha2::Digest;
            let mut hasher = sha2::Sha256::new();
            hasher.update(secret);
            hasher.update(domain);
            let digest = hasher.finalize();
            buf.copy_from_slice(&digest[..24]);
        }
        URL_SAFE_NO_PAD.encode(buf)
    }
}

fn dirs_home() -> PathBuf {
    if let Ok(p) = std::env::var("ARK_ASA_HOME") {
        return PathBuf::from(p);
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") { return PathBuf::from(appdata).join("ark-asa"); }
        if let Ok(userp)   = std::env::var("USERPROFILE") { return PathBuf::from(userp).join("AppData").join("Roaming").join("ark-asa"); }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = std::env::var("HOME") { return PathBuf::from(home).join(".ark-asa"); }
    }
    PathBuf::from(".ark-asa")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn install_test_keyring_id(suffix: &str) -> String {
        let id = format!("auth_admin_v2__{suffix}");
        TEST_KEYRING_OVERRIDE.with(|c| *c.borrow_mut() = Some(id.clone()));
        id
    }

    fn clear_test_keyring_id() {
        TEST_KEYRING_OVERRIDE.with(|c| *c.borrow_mut() = None);
    }

    fn wipe(id: &str) {
        let _ = secret_store::delete(id);
    }

    #[tokio::test]
    async fn round_trip_token() {
        let id = install_test_keyring_id("round_trip");
        wipe(&id);
        let home = std::env::temp_dir().join("ark-asa-test");
        let _ = std::fs::create_dir_all(&home.join(".ark-asa"));
        std::env::set_var("ARK_ASA_HOME", &home);
        let auth = AuthState::load_or_init().await.unwrap();
        let token = auth.active_token().to_string();
        let checked = auth.validate(&format!("Bearer {token}")).unwrap();
        assert_eq!(checked, Role::Admin);
        wipe(&id);
        clear_test_keyring_id();
    }

    #[tokio::test]
    async fn rejects_bad_token() {
        let id = install_test_keyring_id("rejects_bad");
        wipe(&id);
        let home = std::env::temp_dir().join("ark-asa-test2");
        let _ = std::fs::create_dir_all(&home.join(".ark-asa"));
        std::env::set_var("ARK_ASA_HOME", &home);
        let auth = AuthState::load_or_init().await.unwrap();
        assert!(auth.validate("Bearer total-garbage").is_err());
        wipe(&id);
        clear_test_keyring_id();
    }

    /// GA-1 / P8: a legacy `admin.jwt`+`admin.token` pair on disk must be
    /// lifted into the keyring on first boot and the plaintext files must
    /// be deleted afterwards.
    #[tokio::test]
    async fn legacy_plaintext_is_lifted_and_deleted() {
        let id = install_test_keyring_id("legacy_lift");
        wipe(&id);
        // `storage_dir()` returns `<ARK_ASA_HOME>/.ark-asa/`, so the legacy
        // plaintext must live one level down under the home we set.
        let home = std::env::temp_dir().join("ark-asa-test-legacy-lift");
        let dir  = home.join(".ark-asa");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::remove_file(dir.join(SECRET_FILENAME));
        let _ = std::fs::remove_file(dir.join(TOKEN_FILENAME));

        let secret = vec![0xAAu8; 32];
        let token = "LEGACY-TOKEN-do-not-rotate-me".to_string();
        std::fs::write(dir.join(SECRET_FILENAME), &secret).unwrap();
        std::fs::write(dir.join(TOKEN_FILENAME), &token).unwrap();

        std::env::set_var("ARK_ASA_HOME", &home);
        let auth = AuthState::load_or_init().await.unwrap();

        assert_eq!(auth.secret, secret);
        assert_eq!(auth.active_token(), "LEGACY-TOKEN-do-not-rotate-me");
        assert!(!dir.join(SECRET_FILENAME).exists(), "admin.jwt must be deleted");
        assert!(!dir.join(TOKEN_FILENAME).exists(), "admin.token must be deleted");

        let again = AuthState::load_or_init().await.unwrap();
        assert_eq!(again.secret, secret);
        assert_eq!(again.active_token(), "LEGACY-TOKEN-do-not-rotate-me");

        wipe(&id);
        clear_test_keyring_id();
    }
}
