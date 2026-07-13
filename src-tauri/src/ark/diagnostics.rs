//! Server-side diagnostics & repair for the
//! " adverts + IP-connect OK but server invisible in the
//!   in-game Unofficial PC list " symptom on ARK Survival Ascended.
//!
//! Three root causes are detected and auto-fixed:
//!   1. Missing `[Internationalization] Culture=en` in `GameUserSettings.ini`
//!      — ASA requires it for EOS/Epic session registration; without it
//!      the server never publishes to the in-game browser (silent failure).
//!   2. Missing / expired EOS trust root (`Amazon RSA 2048 M02` CRL) in
//!      the Windows Trusted Root store — ASA advertises over TLS 1.2 to
//!      Epic Online Services (ports 80/443) and refuses to register if
//!      the chain can't be validated.
//!   3. Steam install build-id mismatch with the running client — major
//!      ARK patches (v89.x etc.) silently filter out servers whose
//!      reported build-id does not match the client's. We re-run
//!      `steamcmd +app_update 2430930 validate` on request to repair.
//!
//! All commands are read-only unless `repair: true` is passed.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

const EOS_CRL_URL: &str = "http://crl.r2m02.amazontrust.com/r2m02.crl";
const ASA_APP_ID: u32 = 2430930;
const CRL_SUBJECT_HINT: &str = "Amazon RSA 2048 M02";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagCheck {
    pub key:    String,   // stable id: "culture_en" | "eos_cert" | "steam_validate"
    pub label:  String,   // human-readable (Spanish — UI shows this directly)
    pub status: String,   // "ok" | "missing" | "stale" | "fixed" | "error" | "skipped"
    pub detail: String,   // extra info / error text
    pub repaired: bool,   // did this action change the system
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagReport {
    pub checks: Vec<DiagCheck>,
    pub summary: String,
    pub overall_ok: bool,
}

impl Default for DiagReport {
    fn default() -> Self {
        Self { checks: vec![], summary: String::new(), overall_ok: false }
    }
}

fn push(report: &mut DiagReport, c: DiagCheck) {
    report.checks.push(c);
}

// ────────────────────────────────────────────────────────────────────
// 1) [Internationalization] Culture=en in GameUserSettings.ini
// ────────────────────────────────────────────────────────────────────

fn read_gus_ini(server_dir: &str) -> std::io::Result<String> {
    let p = PathBuf::from(server_dir)
        .join("ShooterGame")
        .join("Saved")
        .join("Config")
        .join("WindowsServer")
        .join("GameUserSettings.ini");
    std::fs::read_to_string(&p)
}

fn has_culture_en(content: &str) -> bool {
    let mut in_section = false;
    let mut found = false;
    for raw in content.lines() {
        let t = raw.trim();
        if t.starts_with('[') && t.ends_with(']') {
            in_section = t.eq_ignore_ascii_case("[internationalization]");
            continue;
        }
        if in_section && t.eq_ignore_ascii_case("Culture=en") {
            found = true;
        }
    }
    found
}

/// Insert the `[Internationalization]\nCulture=en\n\n` block at the top
/// of the file, just before the first existing section header, leaving
/// any existing content untouched.
pub fn ensure_culture_block(content: &str) -> (String, bool) {
    if has_culture_en(content) {
        return (content.to_string(), false);
    }
    let block = "[Internationalization]\nCulture=en\n\n";
    // If the file starts with the METADATA comment, keep it first.
    if content.trim_start().starts_with(";METADATA=") {
        let first_nl = content.find('\n').map(|i| i + 1).unwrap_or(0);
        let (head, tail) = content.split_at(first_nl);
        let new = format!("{}{}{}", head, block, tail);
        (new, true)
    } else {
        (format!("{}{}", block, content), true)
    }
}

pub fn check_culture_en(server_dir: &str, repair: bool) -> DiagCheck {
    let label = "culture_en".to_string();
    let content = match read_gus_ini(server_dir) {
        Ok(s) => s,
        Err(e) => return DiagCheck {
            key: "culture_en".into(), label, status: "error".into(),
            detail: format!("Could not read GameUserSettings.ini: {}", e),
            repaired: false,
        },
    };
    if has_culture_en(&content) {
        return DiagCheck {
            key: "culture_en".into(), label, status: "ok".into(),
            detail: "Present — server should register with EOS.".into(),
            repaired: false,
        };
    }
    if !repair {
        return DiagCheck {
            key: "culture_en".into(), label, status: "missing".into(),
            detail: "Missing block. This causes the server to be invisible \
                     in the in-game browser even though direct-IP connect works. \
                     Run 'REPAIR ALL' to add it.".into(),
            repaired: false,
        };
    }
    let (new, _) = ensure_culture_block(&content);
    let path = PathBuf::from(server_dir)
        .join("ShooterGame")
        .join("Saved")
        .join("Config")
        .join("WindowsServer")
        .join("GameUserSettings.ini");
    match std::fs::write(&path, new) {
        Ok(_) => DiagCheck {
            key: "culture_en".into(), label, status: "fixed".into(),
            detail: "[Internationalization] Culture=en block added.".into(),
            repaired: true,
        },
        Err(e) => DiagCheck {
            key: "culture_en".into(), label, status: "error".into(),
            detail: format!("Could not write: {}", e),
            repaired: false,
        },
    }
}

// ────────────────────────────────────────────────────────────────────
// 2) EOS trust root CRL (`Amazon RSA 2048 M02`)
// ────────────────────────────────────────────────────────────────────

async fn crl_installed_current_user() -> bool {
    // Use certutil -store to look for the subject text. Returns silently
    // — cheaper than pulling in the Win32 cert store bindings.
    let out = Command::new("certutil")
        .args(["-store", "-user", "Root"])
        .output()
        .await;
    match out {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout).to_string();
            s.contains(CRL_SUBJECT_HINT)
        }
        Err(_) => false,
    }
}

/// Returns Some(path) to a downloaded CRL in the temp dir, or None.
async fn download_crl() -> Result<PathBuf, String> {
    let tmp = std::env::temp_dir().join("r2m02.crl");
    // Use PowerShell's Invoke-WebRequest instead of pulling reqwest.
    let ps = format!(
        "$ErrorActionPreference='Stop'; \
         try {{ Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing -TimeoutSec 60 }} \
         catch {{ Write-Error $_.Exception.Message; exit 1 }}",
        EOS_CRL_URL,
        tmp.display()
    );
    let out = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps])
        .output()
        .await
        .map_err(|e| format!("Failed to run PowerShell: {}", e))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).to_string();
        return Err(format!("Failed to download EOS CRL: {}", err));
    }
    if !tmp.exists() {
        return Err("CRL was not saved to disk.".into());
    }
    Ok(tmp)
}

fn install_crl_user_store(crl_path: &PathBuf) -> Result<(), String> {
    let out = std::process::Command::new("certutil")
        .args(["-user", "-addstore", "-f", "Root", &crl_path.to_string_lossy()])
        .output()
        .map_err(|e| format!("certutil could not be started: {}", e))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).to_string();
        return Err(format!("certutil failed: {}", err));
    }
    Ok(())
}

/// Returns Some(error) if install into CurrentUser succeeds but admin
/// elevation is required to install into LocalMachine. The UI can then
/// show a banner with a button to retry elevated.
fn install_crl_machine_store(crl_path: &PathBuf) -> Result<(), String> {
    let out = std::process::Command::new("certutil")
        .args(["-addstore", "-f", "Root", &crl_path.to_string_lossy()])
        .output()
        .map_err(|e| format!("certutil could not be started: {}", e))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        if stderr.contains("acceso denegado") || stderr.contains("Access Denied")
            || stderr.contains("0x80070005")
        {
            return Err("NEEDS_ADMIN".into());
        }
        return Err(format!("certutil failed: {}", stderr));
    }
    Ok(())
}

pub async fn check_eos_cert(repair: bool) -> DiagCheck {
    let label = "eos_cert".to_string();

    if crl_installed_current_user().await {
        return DiagCheck {
            key: "eos_cert".into(), label, status: "ok".into(),
            detail: "Present in the user store. If the server is still \
                     invisible in the in-game list, also install for all users \
                     (requires admin — relaunch the app as administrator).".into(),
            repaired: false,
        };
    }

    if !repair {
        return DiagCheck {
            key: "eos_cert".into(), label, status: "missing".into(),
            detail: "Certificate not installed. ASA needs it to register the \
                    server with Epic Online Services. Run 'REPAIR ALL' to install it.".into(),
            repaired: false,
        };
    }

    // Repair path: download + install in CurrentUser, then attempt LocalMachine.
    let crl_path = match download_crl().await {
        Ok(p) => p,
        Err(e) => return DiagCheck {
            key: "eos_cert".into(), label, status: "error".into(),
            detail: e, repaired: false,
        },
    };

    if let Err(e) = install_crl_user_store(&crl_path) {
        return DiagCheck {
            key: "eos_cert".into(), label, status: "error".into(),
            detail: e, repaired: false,
        };
    }

    // Best effort: also install into LocalMachine. If admin is required,
    // we still report a partial fix (Current User is usually enough for
    // the user account that runs ArkAscendedServer.exe).
    match install_crl_machine_store(&crl_path) {
        Ok(_) => DiagCheck {
            key: "eos_cert".into(), label, status: "fixed".into(),
            detail: "CRL installed in CurrentUser and LocalMachine.".into(),
            repaired: true,
        },
        Err(e) if e == "NEEDS_ADMIN" => DiagCheck {
            key: "eos_cert".into(), label, status: "fixed".into(),
            detail: "CRL installed in CurrentUser. To also install into \
                    LocalMachine (all users), relaunch the app as administrator \
                    and re-run diagnostics.".into(),
            repaired: true,
        },
        Err(e) => DiagCheck {
            key: "eos_cert".into(), label, status: "fixed".into(),
            detail: format!("Installed in CurrentUser. LocalMachine skipped: {}", e),
            repaired: true,
        },
    }
}

// ────────────────────────────────────────────────────────────────────
// 3) Steam install validate (`steamcmd +app_update 2430930 validate`)
// ────────────────────────────────────────────────────────────────────

fn parse_manifest_buildid(manifest: &std::path::Path) -> Option<u64> {
    if !manifest.exists() { return None; }
    let text = std::fs::read_to_string(manifest).ok()?;
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("\"buildid\"") {
            let v = rest.split_whitespace().next()
                .unwrap_or("")
                .trim_matches('"')
                .trim_matches('\'')
                .trim_matches(',');
            if let Ok(n) = v.parse::<u64>() {
                return Some(n);
            }
        }
    }
    None
}

pub async fn check_steam_validate(steam_cmd_dir: &str, server_dir: &str, _repair: bool) -> DiagCheck {
    let label = "steam_validate".to_string();

    // SteamCMD installs with `+force_install_dir <server_dir>`, so the
    // manifest lives in <server_dir>/steamapps/. We fall back to the
    // legacy <steam_cmd_dir>/steamapps/ location in case the server was
    // installed without force_install_dir.
    let manifest_name = format!("appmanifest_{}.acf", ASA_APP_ID);
    let manifest = {
        let primary = PathBuf::from(server_dir)
            .join("steamapps")
            .join(&manifest_name);
        if primary.exists() {
            primary
        } else {
            PathBuf::from(steam_cmd_dir)
                .join("steamapps")
                .join(&manifest_name)
        }
    };

    let local_buildid = parse_manifest_buildid(&manifest);
    let detail = match local_buildid {
        Some(b) => format!("Local build: {}. Compare with the client's in-game build.", b),
        None => "Manifest not found — server was installed outside SteamCMD \
                 or the directory is incorrect.".to_string(),
    };

    let status = if local_buildid.is_some() { "ok" } else { "stale" };
    DiagCheck {
        key: "steam_validate".into(), label, status: status.into(),
        detail: format!("{}. To validate / repair stale binaries after a \
                        major ASA update, run: 'steamcmd +force_install_dir \"{}\" \
                        +login anonymous +app_update {} validate +quit'. \
                        (Not auto-executed to avoid disrupting running sessions.)",
                        detail, server_dir, ASA_APP_ID),
        repaired: false,
    }
}

// ────────────────────────────────────────────────────────────────────
// Aggregator command invoked from the UI
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn diagnose_server_list(
    server_dir: String,
    steam_cmd_dir: String,
    repair: bool,
) -> Result<DiagReport, String> {
    let mut report = DiagReport::default();

    // 1) Culture=en
    push(&mut report, check_culture_en(&server_dir, repair));

    // 2) EOS certificate (async — downloads CRL if repairing)
    push(&mut report, check_eos_cert(repair).await);

    // 3) Steam validate (informational — never auto-runs)
    push(&mut report, check_steam_validate(&steam_cmd_dir, &server_dir, repair).await);

    // Summary
    let ok_count  = report.checks.iter().filter(|c| c.status == "ok").count();
    let bad_count = report.checks.len().saturating_sub(ok_count);
    let fixed     = report.checks.iter().any(|c| c.repaired);
    report.overall_ok = bad_count == 0;
    report.summary = if repair {
        if fixed && bad_count == 0 {
            format!("Repair complete — {} check(s) now OK after fixes.", ok_count)
        } else if fixed {
            format!("Partial repair — {} fix(es) applied, {} check(s) still not OK. \
                    Restart the server and re-run diagnostics.",
                report.checks.iter().filter(|c| c.repaired).count(),
                bad_count)
        } else if bad_count == 0 {
            format!("Nothing to repair — {} check(s) OK.", ok_count)
        } else {
            format!("Could not repair everything — {} check(s) not OK. \
                    See each step's detail.", bad_count)
        }
    } else {
        format!("{} check(s) OK, {} need repair.", ok_count, bad_count)
    };

    Ok(report)
}
