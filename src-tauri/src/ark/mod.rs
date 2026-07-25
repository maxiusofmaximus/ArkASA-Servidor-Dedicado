pub mod diagnostics;
pub mod installer;
pub mod labels;
pub mod launcher;
pub mod logs;
pub mod rcon;

// Live consumers in lib.rs and integrations import these two re-exports:
//   - `build_launch_args` for spawning the ASA server with the right flags
//   - `RconClient` for sending RCON commands to a running server
// `map_label` / `map_key_stem` (from `labels`) substitute underscore + trim
// `_WP`, see P25 in OPEN_WORK.md.
// ark/process.rs, ark/server.rs and ark/metrics.rs were deleted in P3.2
// (IMPLEMENTATION_PLAN.md §7.2.2) — they were last touched on 2026-06-25
// and had zero live callers outside each other.
pub use launcher::build_launch_args;
pub use rcon::RconClient;
pub use labels::{map_label, map_key_stem};

/// Apply the canonical Windows `CREATE_NO_WINDOW` (0x08000000) flag to a
/// tokio `Command` so SteamCMD-style console binaries don't pop a black
/// CMD box per spawn. No-op on non-Windows.
///
/// Previously inlined in `installer.rs::run_steamcmd` and duplicated
/// at the version-probe site. Centralising per P6.
#[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
pub fn set_no_window_flag(cmd: &mut tokio::process::Command) {
    #[cfg(target_os = "windows")]
    #[allow(unused_imports)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
}
