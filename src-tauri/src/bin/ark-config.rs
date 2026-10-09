//! `ark-config` — standalone operator CLI for the ARK ASA Configuration Manager.
//!
//! It reuses the exact same dispatch layer (`integrations::dispatch`) that the
//! desktop UI, the loopback HTTP API and the chat-bot adapters already share,
//! so behaviour is identical across channels and the GUI does not have to be
//! running. See `docs/CLI.md`.
//!
//! ```text
//! ark-config [--config <path>] <command> [options]
//!
//!   status                 Cluster + per-map running state
//!   start  [--map N]       Start the whole cluster, or one map instance by index
//!   stop   [--map N]       Gracefully stop the whole cluster, or one map
//!   restart                Restart the whole cluster
//!   logs   [--tail N]      Last N lines of ShooterGame.log (default 20)
//!   ip                     Configured connection addresses
//!   config                 Print the effective TOML configuration
//!   maps                   List map index / id from the effective cluster
//!
//! Global:
//!   --config <path>, -c    Config file (else $ARK_ASA_CONFIG_PATH, else
//!                          %APPDATA%\ARK ASA Config Manager\server-config.toml)
//!   -h, --help             Show this help
//!   -V, --version          Show the version
//! ```

use ark_asa_config::config::schema::ServerConfig;
use ark_asa_config::config::{
    evaluate_rules, CompositeValidator, ConfigLoader, ConfigPersister, CustomValidatorStore,
    PresetStore, ProfileStore,
};
use ark_asa_config::integrations::command_router::{CommandKind, RouterOutcome};
use ark_asa_config::integrations::dispatch;
use std::path::PathBuf;
use std::process::ExitCode;

struct Args {
    config: Option<PathBuf>,
    server: Option<String>,
    command: Option<String>,
    map: Option<u32>,
    tail: Option<u32>,
    positional: Vec<String>,
}

fn parse_args(raw: Vec<String>) -> Result<Args, String> {
    let mut a = Args {
        config: None,
        server: None,
        command: None,
        map: None,
        tail: None,
        positional: Vec::new(),
    };
    let mut it = raw.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-c" | "--config" => {
                a.config = Some(PathBuf::from(
                    it.next().ok_or_else(|| "--config needs a path".to_string())?,
                ));
            }
            "-s" | "--server" => {
                a.server = Some(it.next().ok_or_else(|| "--server needs an id".to_string())?);
            }
            "--map" => {
                let v = it.next().ok_or_else(|| "--map needs an index".to_string())?;
                a.map = Some(v.parse::<u32>().map_err(|_| format!("bad --map value: {v}"))?);
            }
            "--tail" | "-n" => {
                let v = it.next().ok_or_else(|| "--tail needs a number".to_string())?;
                a.tail = Some(v.parse::<u32>().map_err(|_| format!("bad --tail value: {v}"))?);
            }
            other => a.positional.push(other.to_string()),
        }
    }
    a.command = a.positional.first().cloned();
    Ok(a)
}

/// Resolve the config file exactly like the desktop app does
/// (`commands::get_config_dir` → `%APPDATA%\ARK ASA Config Manager`).
fn default_config_path() -> PathBuf {
    if let Ok(p) = std::env::var("ARK_ASA_CONFIG_PATH") {
        if !p.trim().is_empty() {
            return PathBuf::from(p);
        }
    }
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            if !appdata.trim().is_empty() {
                return PathBuf::from(appdata)
                    .join("ARK ASA Config Manager")
                    .join("server-config.toml");
            }
        }
    }
    PathBuf::from("server-config.toml")
}

fn usage() -> &'static str {
    concat!(
        "ark-config ", env!("CARGO_PKG_VERSION"), " — ARK ASA Configuration Manager CLI\n\n",
        "USAGE:\n",
        "  ark-config [--config <path>] <command> [options]\n\n",
        "COMMANDS:\n",
        "  status                 Cluster + per-map running state\n",
        "  start  [--map N]       Start the whole cluster, or one map by index\n",
        "  stop   [--map N]       Gracefully stop the whole cluster, or one map\n",
        "  restart                Restart the whole cluster\n",
        "  logs   [--tail N]      Last N lines of ShooterGame.log (default 20)\n",
        "  ip                     Configured connection addresses\n",
        "  config                 Print the effective TOML configuration\n",
        "  maps                   List map index / id from the effective cluster\n",
        "  validate               Run the built-in + custom validators on the config\n",
        "  drift                  Compare the on-disk INIs against the config\n",
        "  update [--check]       Verified ARK server update via steamcmd\n",
        "  servers                List server profiles (the active one is marked *)\n",
        "  presets                List saved config presets\n",
        "  preset-save <name>     Save the active config as a preset\n",
        "  preset-apply <name>    Write a preset into the active profile config\n",
        "  preset-delete <name>   Delete a preset\n\n",
        "GLOBAL:\n",
        "  --server <id>, -s      Target a specific server profile (else the active one)\n",
        "  --config <path>, -c    Explicit config file (overrides profiles)\n",
        "  -h, --help             Show this help\n",
        "  -V, --version          Show the version\n",
        "\n",
        "Config resolution: --config > --server/profile registry\n",
        "  (%APPDATA%\\ARK ASA Config Manager\\profiles.json) > $ARK_ASA_CONFIG_PATH > legacy\n",
    )
}

/// `%APPDATA%\ARK ASA Config Manager` (the desktop app's config dir).
fn config_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            if !appdata.trim().is_empty() {
                return Some(PathBuf::from(appdata).join("ARK ASA Config Manager"));
            }
        }
    }
    None
}

fn maps_of(config: &ServerConfig) -> Vec<String> {
    let maps = config.effective_maps();
    if maps.is_empty() {
        vec!["TheIsland_WP".to_string()]
    } else {
        maps
    }
}

fn print_status(outcomes: &RouterOutcome) {
    match outcomes {
        RouterOutcome::Status { running, maps } => {
            println!(
                "Cluster: {}",
                if *running { "RUNNING" } else { "STOPPED" }
            );
            if maps.is_empty() {
                println!("  (no maps)");
            }
            for m in maps {
                println!(
                    "  [{}] {:<14} {}",
                    m.map_index,
                    m.map_label,
                    if m.running { "RUNNING" } else { "stopped" }
                );
            }
        }
        other => print_outcome(other),
    }
}

fn print_outcome(o: &RouterOutcome) {
    match o {
        RouterOutcome::Started { pid, map } => println!("OK  {map} started (pid {pid})"),
        RouterOutcome::Stopped { map } => println!("OK  {map} stopped"),
        RouterOutcome::Restarted { map } => println!("OK  {map} restarted"),
        RouterOutcome::Status { .. } => print_status(o),
        RouterOutcome::Logs { lines } => {
            for l in lines {
                println!("{l}");
            }
        }
        RouterOutcome::Ip { primary, entries } => {
            if let Some(p) = primary {
                println!("Primary: {p}");
            }
            for e in entries {
                println!(
                    "  {:<18} {}{}",
                    e.address,
                    e.id,
                    if e.primary { "  (primary)" } else { "" }
                );
            }
        }
        RouterOutcome::ConfigGet { toml } => print!("{toml}"),
        RouterOutcome::ConfigSet { applied } => println!("OK  {applied} field(s) applied"),
        RouterOutcome::Error { reason } => println!("{reason}"),
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();

    if raw.iter().any(|a| a == "-h" || a == "--help") {
        print!("{}", usage());
        return ExitCode::SUCCESS;
    }
    if raw.iter().any(|a| a == "-V" || a == "--version") {
        println!("ark-config {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    let args = match parse_args(raw) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            eprint!("{}", usage());
            return ExitCode::from(2);
        }
    };

    // Optional numeric positionals: `start 1`, `stop 0`, `logs 50`.
    let mut map_idx = args.map;
    let mut tail = args.tail;
    for p in args.positional.iter().skip(1) {
        if let Ok(n) = p.parse::<u32>() {
            if map_idx.is_none() {
                map_idx = Some(n);
            } else if tail.is_none() {
                tail = Some(n);
            }
        }
    }

    let Some(command) = args.command.as_deref() else {
        eprint!("{}", usage());
        return ExitCode::from(2);
    };

    let store = config_dir().map(|d| ProfileStore::new(d));

    // `servers` is answered locally from the profile registry.
    if command == "servers" {
        let Some(s) = &store else {
            eprintln!("error: could not resolve the config dir (APPDATA unset?)");
            return ExitCode::FAILURE;
        };
        let active = s.active_id();
        for p in s.list() {
            println!(
                "{} {:<20} {}",
                if p.id == active { "*" } else { " " },
                p.id,
                p.name
            );
        }
        return ExitCode::SUCCESS;
    }

    let cfg_path = if let Some(c) = args.config.clone() {
        c
    } else if let (Some(s), Some(id)) = (&store, args.server.as_deref()) {
        match s.resolve_config_path(Some(id)) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else if let Some(s) = &store {
        s.resolve_config_path(None)
            .unwrap_or_else(|_| default_config_path())
    } else {
        default_config_path()
    };

    // `presets` / `preset-delete` / `preset-apply` don't need the config loaded.
    if matches!(command, "presets" | "preset-delete" | "preset-apply") {
        let Some(dir) = config_dir() else {
            eprintln!("error: could not resolve the config dir (APPDATA unset?)");
            return ExitCode::FAILURE;
        };
        let store = PresetStore::new(&dir);
        match command {
            "presets" => {
                for p in store.list() {
                    println!("  {:<24} {}", p.name, p.session_name);
                }
            }
            "preset-delete" => {
                let Some(name) = args.positional.get(1) else {
                    eprintln!("error: preset-delete <name>");
                    return ExitCode::from(2);
                };
                if let Err(e) = store.delete(name) {
                    eprintln!("error: {e}");
                    return ExitCode::FAILURE;
                }
                println!("OK  preset `{name}` borrado");
            }
            _ => {
                let Some(name) = args.positional.get(1) else {
                    eprintln!("error: preset-apply <name>");
                    return ExitCode::from(2);
                };
                let preset = match store.load(name) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("error: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                if let Err(e) = ConfigPersister::save_toml(&preset, &cfg_path).await {
                    eprintln!("error: {e}");
                    return ExitCode::FAILURE;
                }
                println!("OK  preset `{name}` aplicado a {}", cfg_path.display());
            }
        }
        return ExitCode::SUCCESS;
    }

    let config = match ConfigLoader::load_or_default(&cfg_path).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: could not load config {}: {e}", cfg_path.display());
            return ExitCode::FAILURE;
        }
    };

    // `maps` is answered locally — it never touches the server lifecycle.
    if command == "maps" {
        for (i, m) in maps_of(&config).iter().enumerate() {
            println!("  [{i}] {m}");
        }
        return ExitCode::SUCCESS;
    }

    // `validate` runs the built-in validators plus the operator's declarative
    // custom validators (`<config_dir>/custom_validators.toml`).
    if command == "validate" {
        let mut errors = match CompositeValidator::default().validate(&config).await {
            Ok(r) => r.errors,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        };
        if let Some(dir) = config_dir() {
            let store = CustomValidatorStore::new(dir);
            errors.extend(evaluate_rules(&config, &store.list()));
        }
        if errors.is_empty() {
            println!("OK  configuración válida");
            return ExitCode::SUCCESS;
        }
        for e in &errors {
            eprintln!("✗ [{}] {}", e.validator, e.message);
        }
        eprintln!("{} problema(s) de validación", errors.len());
        return ExitCode::FAILURE;
    }

    // `drift`: compare the on-disk INIs against what the config says (exit 1
    // when they diverge — ARK or another tool rewrote them).
    if command == "drift" {
        let entries = ark_asa_config::config::drift::detect(&config);
        if entries.is_empty() {
            println!("OK  sin drift (los INI en disco coinciden con la config)");
            return ExitCode::SUCCESS;
        }
        for e in &entries {
            let actual = e.actual.clone().unwrap_or_else(|| "<ausente>".to_string());
            eprintln!(
                "✗ {} [{}] {} = {} (esperado: {})",
                e.file, e.section, e.key, actual, e.expected
            );
        }
        eprintln!("{} diferencia(s) de configuración", entries.len());
        return ExitCode::FAILURE;
    }

    // `preset-save <name>`: snapshot the active config into the preset library.
    if command == "preset-save" {
        let Some(name) = args.positional.get(1) else {
            eprintln!("error: preset-save <name>");
            return ExitCode::from(2);
        };
        let Some(dir) = config_dir() else {
            eprintln!("error: could not resolve the config dir (APPDATA unset?)");
            return ExitCode::FAILURE;
        };
        match PresetStore::new(&dir).save(name, &config) {
            Ok(p) => println!("OK  preset `{}` guardado en {}", p.name, p.path),
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        }
        return ExitCode::SUCCESS;
    }

    // `update [--check]`: verified ARK server update. Refuses to run while the
    // server is up and confirms the local build id actually moved afterwards
    // (a steamcmd "success" that leaves a stale build is a real, common case).
    if command == "update" {
        let check_only = args.positional.iter().any(|p| p == "--check");
        use ark_asa_config::commands::server;
        let pre = match server::check_server_version(config.clone()).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error: no se pudo comprobar la versión: {e}");
                return ExitCode::FAILURE;
            }
        };
        println!("build local : {:?}", pre.local_buildid);
        println!("build remoto: {:?}", pre.latest_buildid);
        if check_only {
            println!(
                "{}",
                if pre.needs_update { "→ actualización disponible" } else { "→ ya está al día" }
            );
            return ExitCode::SUCCESS;
        }
        if !pre.needs_update {
            println!("OK  ya está al día");
            return ExitCode::SUCCESS;
        }
        if server::is_server_running().await {
            eprintln!("error: el servidor está en marcha; páralo antes de actualizar");
            return ExitCode::FAILURE;
        }
        match server::update_server(config.clone()).await {
            Ok(msg) => println!("{msg}"),
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        }
        match server::check_server_version(config).await {
            Ok(post) if !post.needs_update => {
                println!("OK  servidor actualizado a build {:?}", post.local_buildid);
                return ExitCode::SUCCESS;
            }
            Ok(post) => {
                eprintln!(
                    "⚠ la actualización no surtió efecto (local={:?} remoto={:?}); revisa el log de steamcmd",
                    post.local_buildid, post.latest_buildid
                );
                return ExitCode::FAILURE;
            }
            Err(e) => {
                eprintln!("error al verificar tras actualizar: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    let kind = match command {
        "status" => CommandKind::Status,
        "start" => {
            if map_idx.is_some() {
                CommandKind::StartInstance
            } else {
                CommandKind::Start
            }
        }
        "stop" => {
            if map_idx.is_some() {
                CommandKind::StopInstance
            } else {
                CommandKind::Stop
            }
        }
        "restart" => CommandKind::Restart,
        "logs" => CommandKind::Logs,
        "ip" => CommandKind::Ip,
        "config" => CommandKind::ConfigGet,
        other => {
            eprintln!("error: unknown command `{other}`");
            eprint!("{}", usage());
            return ExitCode::from(2);
        }
    };

    let outcome = match dispatch(&config, kind, map_idx, tail).await {
        Ok(o) => o,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    if command == "status" {
        print_status(&outcome);
    } else {
        print_outcome(&outcome);
    }
    ExitCode::SUCCESS
}
