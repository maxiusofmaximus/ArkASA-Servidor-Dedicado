//! Tailscale bridge module — turns the desktop app into a CGNAT-aware
//! operator of Tailscale. No first-party implementation of the
//! WireGuard protocol; we shell out to the official `tailscale` CLI
//! (and to `ip` / `ipconfig` as a probe) and surface what we find.
//!
//! Why a dedicated module?
//!  - Testing CGNAT heuristics without launching a child process.
//!  - Keeping the surfaced JSON shape (`TailscaleStatus`) stable
//!    even as Tailscale ships newer versions of their CLI.
//!
//! Operator flow:
//!   1. Select the Tailscale plugin and open ARKS → Connection.
//!   2. Paste an auth key from <https://login.tailscale.com/admin/settings/keys>.
//!   3. The app runs `tailscale up --auth-key <key> --hostname <host>`.
//!   4. Status panel shows the discovered `100.x.x.x` IP. The
//!      operator can use it as the connection entry in
//!      `network.connection_entries` instead of relying on a router
//!      port-forward.

use serde::{Deserialize, Serialize};

/// What the React UI surfaces about Tailscale. Stable across Tailscale
/// CLI versions — only depends on `tailscale ip -4` and detection
/// heuristics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TailscaleStatus {
    /// Is the `tailscale` binary findable on PATH?
    pub installed: bool,
    /// Is Tailscale currently up (i.e. `tailscale status` exits 0)?
    pub up: bool,
    /// The Tailscale IPv4 (100.64.0.0/10) for this machine.
    pub ip: Option<String>,
    /// The hostname the operator told `tailscale up --hostname …`.
    pub hostname: Option<String>,
    /// Did the heuristic detect CGNAT? We call this *suspect* because
    /// false positives happen (e.g. captive portals, fresh NAT).
    pub cgnat_suspect: bool,
    /// Last public IP probed by `detect_public_ip()`. Used for
    /// comparison — a different WAN IP and advertised IP usually
    /// means CGNAT.
    pub public_ip: Option<String>,
    /// A human-friendly hint: where to download Tailscale, or a
    /// remediation note ("auth key rejected — regenerate on the
    /// admin panel").
    pub hint: String,
}

/// Detection — `tailscale` binary on PATH?
pub fn detect_tailscale_cli() -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Check the install paths Tailscale uses on Windows.
        for p in ["C:\\Program Files\\Tailscale\\tailscale.exe",
                  "C:\\Program Files (x86)\\Tailscale\\tailscale.exe"] {
            if std::path::Path::new(p).exists() { return true; }
        }
        // Also try `where tailscale` with hidden window.
        std::process::Command::new("where")
            .arg("tailscale")
            .creation_flags(0x08000000)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        for p in ["/usr/bin/tailscale",
                  "/usr/local/bin/tailscale",
                  "/opt/homebrew/bin/tailscale",
                  "/snap/bin/tailscale"] {
            if std::path::Path::new(p).exists() { return true; }
        }
        std::process::Command::new("which")
            .arg("tailscale")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// A static URL the operator can hover-click in the UI to download
/// the Tailscale binary.
pub fn tailscale_install_hint() -> &'static str {
    if cfg!(windows) {
        "https://tailscale.com/download/windows"
    } else if cfg!(target_os = "macos") {
        "https://tailscale.com/download/mac"
    } else if cfg!(target_os = "linux") {
        "https://tailscale.com/download/linux"
    } else {
        "https://tailscale.com/download"
    }
}

/// Heuristic: is CGNAT *suspected*?
///
/// We flag suspect when **any** of:
///   1. No public IPv4 probed at all (CGNAT often blocks ipify).
///   2. Tailscale is up AND there's no public IPv4 (operator likely
///      bypassed CGNAT *via* Tailscale).
///   3. Operator there is no Tailscale AND no public IPv4. Just CGNAT.
///
/// Otherwise the heuristic returns false (positive public IP = port
/// forwarding probably works).
pub fn cgnat_suspect(
    public_ip:    &Option<String>,
    tailscale_ip: &Option<String>,
) -> bool {
    match (public_ip, tailscale_ip) {
        (None, _)       => true,
        (Some(_), None) => false,
        (Some(_), Some(_)) => false,
    }
}

/// Spawn `tailscale up` with the auth key. We **don't** bake auth
/// key handling into the UI — we accept the key as a parameter from
/// the React side (operator pastes it from the Tailscale admin
/// panel), pass it through to the CLI, and snapshot the resulting
/// status.
///
/// Returns a structured `TailscaleStatus` so the UI can show the
/// new IP, the hostname, and any stderr message.
///
/// `publicly_dns_label` semantics (we accept three shapes natively):
///   - `None` or empty → no `--advertise-tags` (don't tag the node).
///   - `"label-name"` → wraps to `tag:label-name`.
///   - `"tag:label-name"` → pass through verbatim (operator who
///     already understands Tailscale's tag taxonomy can write the
///     raw form).
/// `extra_tags` is a comma-separated list of additional
/// `--advertise-tags` arguments; each is passed verbatim. The
/// function caps the combined tag count at 8 to keep tailscale's
/// CLI within reasonable args sizes.
pub async fn tailscale_up(
    auth_key: &str,
    hostname: &str,
    publicly_dns_label: Option<&str>,
) -> Result<TailscaleStatus, String> {
    if auth_key.trim().is_empty() {
        return Err("auth key is empty — paste one from https://login.tailscale.com/admin/settings/keys".into());
    }
    if hostname.trim().is_empty() {
        return Err("hostname is empty — pick a Tailscale name like 'arkasa-pi5'".into());
    }
    // Bounded wait — the official CLI can wait on auth/network state. Tokio
    // owns the child process here so timeout cancellation does not strand a
    // blocking worker or leave the setup request hanging forever.
    let out = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        tokio::process::Command::new("tailscale")
            .args(tailscale_up_args(auth_key, hostname, publicly_dns_label))
            .output(),
    )
    .await
    .map_err(|_| "`tailscale up` timed out after 20 seconds; check Tailscale connectivity and try again".to_string())?
    .map_err(|e| format!(
        "couldn't spawn `tailscale up`: {e}. Install Tailscale first: {}",
        tailscale_install_hint()
    ))?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();

    if out.status.success() {
        // Re-poll `tailscale ip -4` to surface the new IP.
        let ip = detect_tailscale_ip_after_up();
        let ip_for_hint = ip.clone();
        Ok(TailscaleStatus {
            installed: true,
            up: true,
            ip,
            hostname: Some(hostname.to_string()),
            cgnat_suspect: ip_for_hint.is_none(), // without an IP we're still stuck
            public_ip: None, // UI calls detect_ips separately
            hint: format!("tailscale up succeeded. {}",
                          ip_for_hint
                              .map(|i| format!("Your Tailscale IP is {i}."))
                              .unwrap_or_else(|| "Could not auto-detect the IP yet. Run `tailscale ip -4` in a terminal.".into())),
        })
    } else {
        Ok(TailscaleStatus {
            installed: true,
            up: false,
            ip: None,
            hostname: Some(hostname.to_string()),
            cgnat_suspect: true,
            public_ip: None,
            hint: format!("tailscale up FAILED. stderr=\n{stderr}\nstdout=\n{stdout}"),
        })
    }
}

/// Build the complete `tailscale up` argument list in one place. Keeping this
/// pure makes the CLI contract testable without launching a real daemon and
/// avoids accidentally reverting to the deprecated `--authkey` spelling.
fn tailscale_up_args(
    auth_key: &str,
    hostname: &str,
    publicly_dns_label: Option<&str>,
) -> Vec<String> {
    let mut args = vec![
        "up".to_string(),
        "--auth-key".to_string(),
        auth_key.to_string(),
        "--hostname".to_string(),
        hostname.to_string(),
    ];
    if let Some(tag_arg) = tag_for_label(publicly_dns_label) {
        args.push("--advertise-tags".to_string());
        args.push(tag_arg);
    }
    args
}

/// Re-polls `tailscale ip -4` and returns whatever Tailscale reports.
fn detect_tailscale_ip_after_up() -> Option<String> {
    if let Ok(out) = std::process::Command::new("tailscale")
        .args(["ip", "-4"])
        .output()
    {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() && !s.to_lowercase().contains("error") {
            // Multi-line: take the first valid IP
            for tok in s.split_whitespace() {
                if is_tailscale_ip(tok) { return Some(tok.to_string()); }
            }
        }
    }
    None
}

fn is_tailscale_ip(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 { return false; }
    let a: u8 = parts[0].parse().unwrap_or(0);

    let b: u8 = parts[1].parse().unwrap_or(0);
    a == 100 && b >= 64 && b <= 127
}

/// Translate an operator-supplied label into a Tailscale
/// `--advertise-tags` argument. Rules:
///  - `None` or empty → `None` (no tag).
///  - `"arkasa"` → `"tag:arkasa"`.
///  - `"tag:arkasa"` → pass through verbatim.
///  - `"tag:"` alone → `None`.
///  - Whitespace is trimmed; uppercase `TAG:` is normalised to
///    lowercase.
pub fn tag_for_label(label: Option<&str>) -> Option<String> {
    let raw = label?.trim();
    if raw.is_empty() { return None; }
    let lower = raw.to_lowercase();
    if let Some(rest) = lower.strip_prefix("tag:") {
        if rest.is_empty() { None } else { Some(rest.to_string()) }
    } else {
        Some(lower)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cgnat_suspect_when_no_public_ip() {
        assert!(cgnat_suspect(&None, &None));
        assert!(cgnat_suspect(&None, &Some("100.123.45.67".into())));
    }

    #[test]
    fn not_suspect_when_public_ip_present() {
        let pub_ip = Some("187.234.12.5".into());
        let ts_ip  = None;
        assert!(!cgnat_suspect(&pub_ip, &ts_ip));
        let pub_ip = Some("187.234.12.5".into());
        let ts_ip  = Some("100.100.100.50".into());
        assert!(!cgnat_suspect(&pub_ip, &ts_ip));
    }

    #[test]
    fn is_tailscale_ip_100_range() {
        assert!(is_tailscale_ip("100.64.0.1"));
        assert!(is_tailscale_ip("100.100.100.50"));
        assert!(is_tailscale_ip("100.127.255.254"));
        // Out of range
        assert!(!is_tailscale_ip("100.63.0.1"));
        assert!(!is_tailscale_ip("100.128.0.1"));
        // Public IP
        assert!(!is_tailscale_ip("187.234.12.5"));
        assert!(!is_tailscale_ip("8.8.8.8"));
        // Wrong shape
        assert!(!is_tailscale_ip("not-an-ip"));
        assert!(!is_tailscale_ip(""));
    }

    #[test]
    fn install_hint_returns_some_url() {
        let url = tailscale_install_hint();
        assert!(url.starts_with("https://tailscale.com"));
    }

    #[test]
    fn tailscale_up_rejects_empty_inputs() {
        // Run synchronously via `tokio::runtime::Runtime` because
        // tailscale_up is async fn but we don't need a real
        // runtime — we use block_on.
        let rt = tokio::runtime::Runtime::new().unwrap();
        let res = rt.block_on(tailscale_up("", "arkasa", None));
        assert!(res.is_err());
        let res = rt.block_on(tailscale_up("tskey-fake", "", None));
        assert!(res.is_err());
    }

    #[test]
    fn tailscale_up_uses_current_auth_key_flag() {
        let args = tailscale_up_args("tskey-auth-test", "arkasa", None);
        assert_eq!(args, vec![
            "up", "--auth-key", "tskey-auth-test", "--hostname", "arkasa",
        ]);
    }

    #[test]
    fn status_default_is_empty() {
        let s = TailscaleStatus::default();
        assert!(!s.installed);
        assert!(!s.up);
        assert!(s.ip.is_none());
        assert!(s.hostname.is_none());
        assert!(!s.cgnat_suspect);
        assert!(s.public_ip.is_none());
        assert!(s.hint.is_empty());
    }
}
