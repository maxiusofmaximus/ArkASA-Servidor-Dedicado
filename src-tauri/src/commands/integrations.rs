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
use crate::commands::network::DetectedIps;
use crate::commands::utilities::{detect_public_ip, detect_tailscale_ip, detect_local_ip};
use crate::commands::receipts::shared_ledger;

#[tauri::command]
pub async fn admin_token(
    _auth: tauri::State<'_, ()>,
) -> Result<String, String> {
    let holder = auth_initial_holder_static();
    let guard  = holder.lock().await;
    match guard.as_ref() {
        Some(a) => Ok(a.active_token().to_string()),
        None   => Err("admin auth not yet initialised — restart the app and retry".into()),
    }
}

#[tauri::command]
pub async fn rotate_admin_token() -> Result<String, String> {
    let holder = auth_initial_holder_static();
    let mut guard = holder.lock().await;
    let Some(a) = guard.as_mut() else {
        return Err("admin auth not yet initialised".into());
    };
    let mut a_clone = (**a).clone();
    let token = a_clone.rotate().await?.to_string();
    *guard = Some(Arc::new(a_clone));
    Ok(token)
}

#[tauri::command]
pub async fn set_admin_feature_flag(_key: String, _value: bool) -> Result<(), String> {
    // Hito 4 wires this up to a TOML-backed registry so Convex enable/
    // disable round-trips to disk rather than eating argv. For now this
    // is intentionally a no-op so the frontend can call it without
    // crashing.
    Ok(())
}

/// Manually trigger a one-shot migration of any remaining v1 plaintext
/// `secret_store.toml` files into the OS keyring. Idempotent: returns 0 if
/// everything has already been lifted. Also runs automatically on app
/// startup inside the `setup` closure - this command exposes it to the
/// frontend so an operator can re-trigger it from a diagnostics UI.
#[tauri::command]
pub async fn migrate_secrets() -> Result<usize, String> {
    crate::plugins::secret_store_v2::migrate_secrets()
}

/// Static handle to the AuthState, populated by `run()` at startup.
/// Hito 4 will replace this with a `tauri::manage`-d value once we
/// settle on the best pattern.
use std::sync::OnceLock;
static AUTH_HOLDER: OnceLock<Arc<tokio::sync::Mutex<Option<Arc<auth::AuthState>>>>> =
    OnceLock::new();
pub fn auth_initial_holder_static() -> Arc<tokio::sync::Mutex<Option<Arc<auth::AuthState>>>> {
    AUTH_HOLDER.get_or_init(|| Arc::new(tokio::sync::Mutex::new(None))).clone()
}

pub fn machine_host_id() -> Result<String, String> {
    use std::process::Command;
    #[cfg(target_os = "windows")]
    let out = Command::new("hostname").output().map_err(|e| e.to_string())?;
    #[cfg(not(target_os = "windows"))]
    let out = Command::new("hostname").output().map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[tauri::command]
pub async fn detect_ips() -> DetectedIps {
    let (public_ip, tailscale_ip, local_ip) = tokio::join!(
        detect_public_ip(),
        detect_tailscale_ip(),
        async { detect_local_ip() },
    );
    DetectedIps { public_ip, tailscale_ip, local_ip }
}

// ─── Tailscale wizard (v2.1, Network blocker #4) ──────────────────────

/// Check whether the `tailscale` CLI binary is installed. The UI
/// surfaces a hint (download URL) when `installed` is false.
#[tauri::command]
pub fn tailscale_installed() -> bool {
    integrations::tailscale::detect_tailscale_cli()
}

/// Returns the Tailscale download URL for the operator's platform.
#[tauri::command]
pub fn tailscale_download_url() -> String {
    integrations::tailscale::tailscale_install_hint().to_string()
}

/// Combined status panel: detects public IP + tailscale IP +
/// decides if CGNAT is *suspected* based on the heuristic. Used
/// by the React UI to decide whether to suggest the Tailscale
/// wizard.
#[tauri::command]
pub async fn tailscale_status_combined() -> integrations::tailscale::TailscaleStatus {
    let (public_ip, ts_ip) = tokio::join!(
        detect_public_ip(),
        detect_tailscale_ip(),
    );
    let installed = integrations::tailscale::detect_tailscale_cli();
    let ts_has_ip = ts_ip.is_some();
    let cgnat = integrations::tailscale::cgnat_suspect(&public_ip, &ts_ip);
    integrations::tailscale::TailscaleStatus {
        installed,
        up:           installed && ts_has_ip,
        ip:           ts_ip,
        hostname:     None,
        cgnat_suspect: cgnat,
        public_ip,
        hint: if installed {
            if cgnat {
                "CGNAT detected (no public IP). Consider setting up Tailscale.".into()
            } else if ts_has_ip {
                "Public IP reachable. Tailscale is up but you may not need it.".into()
            } else {
                "Public IP reachable. Tailscale installed but not up — pastes Auth Key to enable.".into()
            }
        } else {
            format!(
                "Tailscale not installed. Download from {} and rerun Setup.",
                integrations::tailscale::tailscale_install_hint()
            )
        },
    }
}

/// Run `tailscale up --auth-key <key> --hostname <host>` against
/// the official CLI. We persist nothing — secrets stay in the
/// `secret_store_v2` (OS keyring) if the operator wants to refresh later.
#[tauri::command]
pub async fn tailscale_setup(
    auth_key:          String,
    hostname:          String,
    publicly_dns_label: Option<String>,
) -> Result<integrations::tailscale::TailscaleStatus, String> {
    integrations::tailscale::tailscale_up(
        &auth_key,
        &hostname,
        publicly_dns_label.as_deref(),
    ).await
}

#[tauri::command]
pub fn parse_config_from_toml(toml_str: String) -> Result<config::ServerConfig, String> {
    let mut config: config::ServerConfig = toml::from_str(&toml_str)
        .map_err(|e| format!("Failed to parse TOML: {}", e))?;
    config.network.migrate_legacy_connections();
    Ok(config)
}

/// Serialize a ServerConfig back to TOML string (used after zip import).
#[tauri::command]
pub fn config_to_toml(config: config::ServerConfig) -> Result<String, String> {
    toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))
}

/// v2.2 — Hosting adapter: render the cloud-init / startup script for the
/// chosen provider so the operator can copy-paste it into their VPS
/// marketplace.
#[tauri::command]
pub fn render_hosting_script(
    target: integrations::hosting::HostTarget,
    bundle_url: String,
) -> Result<String, String> {
    Ok(integrations::hosting::provision_script(&target, &bundle_url))
}

/// v2.2 — Hosting adapter: list every provider supported by the application.
#[tauri::command]
pub fn list_hosting_providers() -> Vec<HostingProviderView> {
    integrations::hosting::HostProvider::all().iter().map(|p| {
        HostingProviderView {
            key: format!("{:?}", p).to_lowercase(),
            label: p.label().to_string(),
        }
    }).collect()
}

#[derive(serde::Serialize)]
pub struct HostingProviderView {
    pub key: String,
    pub label: String,
}

/// v2.2 — Database adapter: list supported backends.
#[tauri::command]
pub fn list_database_backends() -> Vec<integrations::database::DbBackendView> {
    integrations::database::all_backends()
}

/// v2.2 — Database adapter: validate a connection string for the chosen
/// backend without performing a network call (best-effort syntactic check).
#[tauri::command]
pub fn validate_database_config(cfg: integrations::database::DatabaseConfig) -> Result<(), String> {
    if cfg.url.is_empty() { return Err("Missing database URL".into()); }
    match cfg.backend {
        integrations::database::DbBackend::Sqlite | integrations::database::DbBackend::SqliteAlt => Ok(()),
        integrations::database::DbBackend::Convex => {
            if !cfg.url.starts_with("http") { return Err("Convex URL must start with http(s)".into()); }
            Ok(())
        }
        integrations::database::DbBackend::Supabase | integrations::database::DbBackend::Insforge => {
            if !cfg.url.starts_with("http") { return Err("URL must start with http(s)".into()); }
            if cfg.api_key.is_empty() { return Err("Anon key required".into()); }
            Ok(())
        }
        integrations::database::DbBackend::Postgres => {
            if !cfg.url.starts_with("http") && !cfg.url.starts_with("postgres") {
                return Err("Postgres URL must be http(s) or postgres://".into());
            }
            Ok(())
        }
        integrations::database::DbBackend::Mongodb => {
            if !cfg.url.starts_with("https") {
                return Err("MongoDB Atlas Data API requires https://...".into());
            }
            if cfg.api_key.is_empty() { return Err("Atlas API key required".into()); }
            Ok(())
        }
    }
}

/// v2.2 — Hosting adapter: track a deployment record inside the audit log.
#[tauri::command]
pub async fn record_hosting_deployment(target: integrations::hosting::HostTarget) -> Result<(), String> {
    if let Some(ledger) = shared_ledger().read().as_ref().cloned() {
        ledger.append(
            serde_json::json!({
                "provider": target.provider.label(),
                "region":   target.region,
                "ssh_host": target.ssh_host,
                "disk_gb":  target.disk_gb,
            }),
            receipts::Stage::Hosting,
        )?;
    }
    Ok(())
}

/// v2.3 — Hosting adapter: render a single-file bash runner the operator
/// copies into their workstation. Wraps the right provider CLI
/// (`hcloud`/`doctl`/`aws`/`az`/`gcloud`/`oci`/`vagrant`).
#[tauri::command]
pub fn render_provider_run_script(
    target: integrations::hosting::HostTarget,
    bundle_url: String,
) -> Result<String, String> {
    integrations::hosting::render_provider_run_script(&target, &bundle_url)
}

/// Render a complete local-provision plan for the operator's own
/// hardware (Raspberry Pi 5 / Intel NUC / WSL2 / macOS). The frontend
/// calls this from the **Run locally** disclosure inside `HostingTab`.
/// Returns a 4-tuple: bundled script, inline one-liner, stages
/// (per-step verbatim copy + expectation), platform notes. See
/// `src-tauri/src/integrations/local_provision.rs` for the patch
/// details (apt → brew on macOS, systemd → `screen` on macOS, etc.).
#[tauri::command]
pub fn render_local_provision_plan(
    class:        String,
    ssh_user:     String,
    ssh_host:     String,
    bundle_url:   String,
    disk_gb:      u32,
) -> Result<integrations::local_provision::LocalProvisionPlan, String> {
    use integrations::local_provision::{LocalProvisionPlan, LocalTargetClass};
    let class = match class.as_str() {
        "debian-pi5"     => LocalTargetClass::DebianPi5,
        "debian-x86"     => LocalTargetClass::DebianX86,
        "ubuntu-x86"     => LocalTargetClass::UbuntuX86,
        "wsl2-debian"    => LocalTargetClass::Wsl2Debian,
        "wsl2-ubuntu"    => LocalTargetClass::Wsl2Ubuntu,
        "macos-arm"      => LocalTargetClass::MacosArm,
        "macos-intel"    => LocalTargetClass::MacosIntel,
        other            => return Err(format!("unknown local target class `{other}` — pick one of {:?}", LocalTargetClass::all())),
    };
    Ok::<LocalProvisionPlan, String>(
        integrations::local_provision::build_local_plan(
            class, &ssh_user, &ssh_host, &bundle_url, disk_gb
        )
    )
}

// ─── Receipts ledger (v2.2) ──────────────────────────────────────────────────
//
// Lazily-initialised JSONL append-only ledger under `${AppData}/receipts/`.
// Shared across all Tauri commands via the OnceLock cell in `shared_ledger()`.
//
// `receipts_today_path`     → operator-facing path to today's JSONL file.
// `receipts_tail(n)`         → last N receipts for UI/teaching support.
// `receipts_probe(host_id)`  → id ensures the ledger is bound to this host.
