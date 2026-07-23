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
