//! Public, read-only status surface: a small JSON document and SVG badges.
//!
//! **Opt-in and sanitised on purpose.** The loopback API is bearer-protected
//! for a reason, so these endpoints are:
//!   * disabled unless the operator sets `ARK_ASA_PUBLIC_STATUS` to a truthy
//!     value (1/true/yes/on) — nothing is exposed by default;
//!   * read-only and leak-free: they expose the session name, running state,
//!     configured map/mod/player counts, FPS and the app version. **Never**
//!     paths, ports, passwords or tokens.
//!
//! Expose them to the internet only through a reverse proxy with TLS (see the
//! security notes in `docs/RESEARCH_NEW_IDEAS.md`).

use serde_json::json;

/// Whether the operator opted into the public surface.
pub fn enabled() -> bool {
    matches!(
        std::env::var("ARK_ASA_PUBLIC_STATUS")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn truthy(v: bool) -> &'static str {
    if v { "RUNNING" } else { "OFFLINE" }
}

/// Sanitised status document. Safe to publish.
pub async fn status_json() -> serde_json::Value {
    let m = crate::commands::server::metrics_from_env().await;
    match m {
        Some(m) => {
            let running = m.get("running").and_then(|v| v.as_bool()).unwrap_or(false);
            json!({
                "app": "ark-asa-config",
                "app_version": env!("CARGO_PKG_VERSION"),
                "session_name": m.get("session_name").cloned().unwrap_or(json!("")),
                "status": truthy(running),
                "running": running,
                "maps_total": m.get("maps_total"),
                "mods_active": m.get("mods_count"),
                "max_players": m.get("max_players"),
                "platform": m.get("server_platform"),
                "fps": m.get("fps"),
                "updated_ms": m.get("timestamp_ms"),
            })
        }
        None => json!({
            "app": "ark-asa-config",
            "app_version": env!("CARGO_PKG_VERSION"),
            "status": "UNKNOWN",
            "running": false,
        }),
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// A shields.io-style flat SVG badge: `<label>: RUNNING/OFFLINE`.
pub async fn badge_svg(label: &str) -> String {
    let m = crate::commands::server::metrics_from_env().await;
    let running = m
        .as_ref()
        .and_then(|v| v.get("running"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let maps = m
        .as_ref()
        .and_then(|v| v.get("maps_total"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let label = if label.trim().is_empty() { "ARK ASA" } else { label.trim() };
    let status = if running { "RUNNING" } else { "OFFLINE" };
    let color = if running { "#2ea043" } else { "#b62324" };

    // Fixed-width, monospace-ish badge (no external font needed).
    let lw = (label.chars().count() as i32 * 7 + 16).max(40);
    let rw = (status.chars().count() as i32 * 7 + 16).max(40);
    let total = lw + rw;
    let lx = lw as f32 / 2.0;
    let rx = lw as f32 + rw as f32 / 2.0;

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{total}" height="20" role="img" aria-label="{label}: {status}">
<title>{label}: {status}</title>
<linearGradient id="s" x2="0" y2="100%"><stop offset="0" stop-color="#bbb" stop-opacity=".1"/><stop offset="1" stop-opacity=".1"/></linearGradient>
<clipPath id="r"><rect width="{total}" height="20" rx="3" fill="#fff"/></clipPath>
<g clip-path="url(#r)">
<rect width="{lw}" height="20" fill="#555"/>
<rect x="{lw}" width="{rw}" height="20" fill="{color}"/>
<rect width="{total}" height="20" fill="url(#s)"/>
</g>
<g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" font-size="11">
<text x="{lx}" y="15" fill="#010101" fill-opacity=".3">{label}</text>
<text x="{lx}" y="14">{label}</text>
<text x="{rx}" y="15" fill="#010101" fill-opacity=".3">{status}</text>
<text x="{rx}" y="14">{status}</text>
</g>
<!-- maps: {maps} -->
</svg>"##,
        total = total,
        lw = lw,
        rw = rw,
        lx = lx,
        rx = rx,
        label = xml_escape(label),
        status = status,
        color = color,
        maps = maps,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_by_default() {
        // In the test env the var is unset.
        std::env::remove_var("ARK_ASA_PUBLIC_STATUS");
        assert!(!enabled());
        std::env::set_var("ARK_ASA_PUBLIC_STATUS", "true");
        assert!(enabled());
        std::env::remove_var("ARK_ASA_PUBLIC_STATUS");
    }

    #[tokio::test]
    async fn badge_shape_and_status_colour() {
        let svg = badge_svg("ARK ASA").await;
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("ARK ASA"));
        assert!(svg.contains("OFFLINE") || svg.contains("RUNNING"));
    }

    #[test]
    fn escapes_ampersands_in_labels() {
        assert_eq!(xml_escape("a&b"), "a&amp;b");
    }
}
