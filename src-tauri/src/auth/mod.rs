//! JWT-style auth for the v2.1 loopback HTTP API.
//!
//! Two artefacts are persisted in `~/.ark-asa/`:
//!   - `admin.jwt`  : HS256 signing secret used by the bearer token JWTs
//!   - `admin.token` : the latest bearer token the desktop UI shows
//!
//! On first launch the suite generates a 256-bit random secret, derives the
//! initial admin token from it, and writes both to disk. The token is then
//! displayed in `Options → Remote Admin` so the operator can copy it.
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
use tokio::fs;

const SECRET_FILENAME: &str = "admin.jwt";
const TOKEN_FILENAME:  &str = "admin.token";

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

    /// Load (or first-time-create) the auth state.
    pub async fn load_or_init() -> Result<Self, String> {
        let dir = Self::storage_dir();
        fs::create_dir_all(&dir).await.map_err(|e| format!("mkdir {dir:?}: {e}"))?;

        let secret_path = dir.join(SECRET_FILENAME);
        let token_path  = dir.join(TOKEN_FILENAME);

        let secret = if secret_path.exists() {
            let raw = fs::read(&secret_path).await.map_err(|e| format!("read secret: {e}"))?;
            if raw.len() < 32 {
                return Err("stored secret is too short".into());
            }
            raw
        } else {
            let mut s = vec![0u8; 32];
            rand::thread_rng().fill_bytes(&mut s);
            fs::write(&secret_path, &s).await.map_err(|e| format!("write secret: {e}"))?;
            s
        };

        let token = if token_path.exists() {
            fs::read_to_string(&token_path).await.map_err(|e| format!("read token: {e}"))?
        } else {
            // Initial bearer token: derived from the secret rather than from
            // a fresh source so we can re-issue it on demand later without
            // having to write a second random secret. 32 bytes → base64url.
            let mut buf = [0u8; 24];
            {
                let mut hasher = sha2::Sha256::new();
                use sha2::Digest;
                hasher.update(&secret);
                hasher.update(b"v2.1-initial-token");
                let digest = hasher.finalize();
                buf.copy_from_slice(&digest[..24]);
            }
            let encoded = URL_SAFE_NO_PAD.encode(buf);
            fs::write(&token_path, &encoded).await.map_err(|e| format!("write token: {e}"))?;
            encoded
        };

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

    /// Re-issue the bearer token, rotating the secret. Persists the new state.
    pub async fn rotate(&mut self) -> Result<&str, String> {
        let mut new_secret = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut new_secret);
        self.secret = new_secret.clone();

        let mut buf = [0u8; 24];
        {
            let mut hasher = sha2::Sha256::new();
            use sha2::Digest;
            hasher.update(&new_secret);
            hasher.update(b"v2.1-rotated-token");
            let digest = hasher.finalize();
            buf.copy_from_slice(&digest[..24]);
        }
        let token = URL_SAFE_NO_PAD.encode(buf);
        self.token = token.clone();

        let dir = Self::storage_dir();
        fs::write(dir.join(SECRET_FILENAME), &self.secret).await
            .map_err(|e| format!("write secret: {e}"))?;
        fs::write(dir.join(TOKEN_FILENAME), &token).await
            .map_err(|e| format!("write token: {e}"))?;
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

    #[tokio::test]
    async fn round_trip_token() {
        let dir = std::env::temp_dir().join("ark-asa-test");
        std::env::set_var("ARK_ASA_HOME", &dir);
        let auth = AuthState::load_or_init().await.unwrap();
        let token = auth.active_token().to_string();
        let checked = auth.validate(&format!("Bearer {token}")).unwrap();
        assert_eq!(checked, Role::Admin);
    }

    #[tokio::test]
    async fn rejects_bad_token() {
        let dir = std::env::temp_dir().join("ark-asa-test2");
        std::env::set_var("ARK_ASA_HOME", &dir);
        let auth = AuthState::load_or_init().await.unwrap();
        assert!(auth.validate("Bearer total-garbage").is_err());
    }
}
