//! Prometheus / OpenMetrics exposition for the ARK ASA dedicated server.
//!
//! Rendered on demand by the loopback HTTP API at `GET /api/v1/metrics`
//! (bearer-authenticated, like every other endpoint) so an operator can scrape
//! it with Prometheus and graph it in the bundled Grafana dashboard
//! (`docs/grafana/ark-asa-dashboard.json`).
//!
//! Design notes:
//!   * Reuses `commands::server::metrics_for_config` — single source of truth.
//!   * Honest absence: a metric whose value is not available (e.g. FPS before
//!     the log has seen a `Stat FPS` line, memory on a failed live-parse) is
//!     simply **not emitted**, matching the P28 rule that the UI/API never
//!     paints a phantom zero.
//!   * Values are gauges (no counters yet) — cheap and safe to scrape every 15s.

/// Escape a Prometheus label value per the exposition format.
fn esc(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
}

/// Render the full exposition text (ASCII, `text/plain; version=0.0.4`).
pub async fn render() -> String {
    let mut out = String::new();
    out.push_str("# HELP arkasa_info Static information about the ARK ASA Config Manager.\n");
    out.push_str("# TYPE arkasa_info gauge\n");
    out.push_str(&format!(
        "arkasa_info{{version=\"{}\"}} 1\n",
        esc(env!("CARGO_PKG_VERSION"))
    ));

    // No active profile resolvable → info-only, still a valid exposition.
    let Some(m) = crate::commands::server::metrics_from_env().await else {
        return out;
    };

    let session = m.get("session_name").and_then(|v| v.as_str()).unwrap_or("");
    let lbl = if session.is_empty() {
        String::new()
    } else {
        format!("{{session=\"{}\"}}", esc(session))
    };

    out.push_str(
        "# HELP arkasa_server_running 1 if the ARK server process is running.\n\
         # TYPE arkasa_server_running gauge\n",
    );
    let running = m.get("running").and_then(|v| v.as_bool()).unwrap_or(false);
    out.push_str(&format!(
        "arkasa_server_running{} {}\n",
        lbl,
        if running { 1 } else { 0 }
    ));

    // Emit each gauge only when the value is a real number.
    macro_rules! gauge {
        ($name:literal, $help:literal, $key:literal) => {{
            out.push_str(concat!(
                "# HELP ", $name, " ", $help, "\n",
                "# TYPE ", $name, " gauge\n"
            ));
            if let Some(n) = m.get($key).and_then(|v| v.as_f64()) {
                out.push_str(&format!("{}{} {}\n", $name, lbl, n));
            }
        }};
    }

    gauge!("arkasa_process_memory_mb", "Resident memory of the ARK server process, in MB.", "memory_mb");
    gauge!("arkasa_server_fps", "Server FPS as reported in ShooterGame.log.", "fps");
    gauge!("arkasa_udp_conns", "Public UDP connections held by the ARK process.", "udp_conns");
    gauge!("arkasa_maps_total", "Configured cluster maps.", "maps_total");
    gauge!("arkasa_mods_active", "Active mods.", "mods_count");
    gauge!("arkasa_max_players", "Configured max players.", "max_players");
    gauge!("arkasa_game_port", "Configured game port.", "game_port");
    gauge!("arkasa_query_port", "Configured query port.", "query_port");
    gauge!("arkasa_rcon_port", "Configured RCON port.", "rcon_port");

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_label_values() {
        assert_eq!(esc("a\"b\\c\nd"), "a\\\"b\\\\c d");
    }

    #[tokio::test]
    async fn renders_valid_header_without_active_profile() {
        // In the test env ARK_ASA_CONFIG_PATH is normally unset → info-only,
        // which must still be valid exposition text.
        let body = render().await;
        assert!(body.contains("# TYPE arkasa_info gauge"));
        assert!(body.contains("arkasa_info{version="));
    }

    #[tokio::test]
    async fn renders_gauges_for_active_profile() {
        let dir = std::env::temp_dir().join(format!(
            "ark-metrics-test-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let cfg_path = dir.join("server-config.toml");
        let mut c = crate::config::schema::ServerConfig::default();
        c.identification.session_name = "MetricsTest".into();
        c.cluster_maps = vec!["TheIsland_WP".into(), "Ragnarok_WP".into()];
        std::fs::write(&cfg_path, toml::to_string(&c).unwrap()).unwrap();

        std::env::set_var("ARK_ASA_CONFIG_PATH", &cfg_path);
        let body = render().await;
        std::env::remove_var("ARK_ASA_CONFIG_PATH");
        let _ = std::fs::remove_dir_all(&dir);

        assert!(
            body.contains("arkasa_server_running{session=\"MetricsTest\"}"),
            "running gauge missing:\n{body}"
        );
        assert!(
            body.contains("arkasa_maps_total{session=\"MetricsTest\"} 2"),
            "maps gauge missing/wrong:\n{body}"
        );
    }
}
