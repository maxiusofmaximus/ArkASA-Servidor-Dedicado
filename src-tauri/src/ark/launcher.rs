//! ARK launch argument builder — single source of truth for CLI parameters.
//!
//! Both `start_server` (Tauri command) and `stub::run_stub` (on-demand lifecycle)
//! use `build_launch_args` so port offsets, cluster flags, mod lists, and IP
//! advertising stay consistent without duplication.

use crate::config::schema::ServerConfig;

/// The two parts of an ARK server invocation.
pub struct LaunchArgs {
    /// First positional argument: `TheIsland_WP?listen?SessionName=...`
    pub url_params: String,
    /// Additional CLI flags: `-NoBattlEye`, `-mods=...`, etc.
    pub flags: Vec<String>,
}

/// Build `LaunchArgs` for one ARK instance in a (potentially) clustered setup.
///
/// `instance_idx` is the 0-based position in `config.effective_maps()`.
/// For a single-server setup, always pass `0`.
///
/// **Port assignment policy** is governed by `network.fixed_port_assignment_per_map`:
///   * `true`  (default)  → stable hash of `map` so TheIsland always lands on
///                          the same triplet regardless of boot order.
///   * `false` (legacy)   → use the `instance_idx` stride, i.e. first-launched
///                          map gets port 0×stride (matches pre-fix behaviour).
pub fn build_launch_args(config: &ServerConfig, map: &str, instance_idx: usize) -> LaunchArgs {
    let maps = config.effective_maps();
    let is_cluster = maps.len() > 1;

    // ── Port assignment ───────────────────────────────────────────────────────
    let (game_port, query_port, rcon_port) = if config.network.fixed_port_assignment_per_map {
        config.network.ports_for_map_id(map)
    } else {
        config.network.ports_for_index(instance_idx)
    };

    // In cluster mode: secondary maps get a human-readable suffix so players
    // can distinguish them in the ARK server browser.
    let map_label = map.trim_end_matches("_WP");
    let session_name = if is_cluster && instance_idx > 0 {
        format!("{} · {}", config.identification.session_name, map_label)
    } else {
        config.identification.session_name.clone()
    };

    // ── URL-style parameter string (first CLI arg) ────────────────────────────
    let mut url = format!(
        "{}?listen?SessionName={}?ServerAdminPassword={}",
        map, session_name, config.identification.admin_password,
    );
    if !config.identification.server_password.is_empty() {
        url.push_str(&format!("?ServerPassword={}", config.identification.server_password));
    }
    url.push_str(&format!(
        "?MaxPlayers={}?Port={}?QueryPort={}?RCONEnabled=True?RCONPort={}",
        config.gameplay.max_players, game_port, query_port, rcon_port,
    ));

    // ── Additional CLI flags ──────────────────────────────────────────────────
    // Note: `-NoBattlEye` is conditionally added based on `network.no_battleye`.
    // When the flag is `true` (operator has BattleEye off), we **omit** it so
    // ARK launches with anti-cheat disabled. When `false` (default), we keep
    // the existing behaviour (BattleEye off — matches v2.0.0 release).
    let mut flags = vec![
        "-server".to_string(),
        "-log".to_string(),
        "-servergamelog".to_string(),
        "-NoTransferFromFiltering".to_string(),
        format!("-WinLiveMaxPlayers={}", config.gameplay.max_players),
    ];
    if config.network.no_battleye {
        flags.push("-NoBattlEye".to_string());
    }

    // Advertise the configured IP so cluster-travel redirects and direct
    // connections use the correct reachable address.
    let effective_ip = config.network.effective_ip();
    if !effective_ip.is_empty() {
        flags.push(format!("-ip={}", effective_ip));
    }

    if is_cluster {
        flags.push(format!("-clusterid={}", config.cluster_id()));
        flags.push(format!("-ClusterDirOverride={}", config.paths.cluster_dir()));
    }

    if !config.mods.active_mods.is_empty() {
        flags.push(format!("-mods={}", config.mods.active_mods.join(",")));
    }

    LaunchArgs { url_params: url, flags }
}
