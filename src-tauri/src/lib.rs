pub mod config;
pub mod error;
pub mod cli;
pub mod backup;
pub mod stub;
pub mod auth;
pub mod integrations;
pub mod plugins;
pub mod receipts;
pub mod commands;
pub mod events;

mod ark;

use commands::*;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use tauri::{Manager, Emitter};

pub fn run() {
    let tray_state     = Arc::new(TrayState { minimize_to_tray: AtomicBool::new(true) });
    let on_demand_state = Arc::new(stub::OnDemandState::new());

    // v2.1 — admin auth + loopback HTTP API.
    // AuthState::load_or_init is async; bounded blocking spawn so the UI thread
    // does not stall on first run.
    let auth_initial_holder: Arc<tokio::sync::Mutex<Option<Arc<auth::AuthState>>>> =
        Arc::new(tokio::sync::Mutex::new(None));
    {
        let holder = auth_initial_holder.clone();
        tauri::async_runtime::spawn(async move {
            match auth::AuthState::load_or_init().await {
                Ok(auth) => {
                    log::info!(
                        "admin token started; copy from Options → Remote Admin. Active token length: {}",
                        auth.active_token().len(),
                    );
                    *holder.lock().await = Some(Arc::new(auth));
                }
                Err(e) => log::error!("admin auth init failed: {e}"),
            }
        });
    }
    let admin_state_holder: Arc<tokio::sync::Mutex<Option<Arc<integrations::http_api::AdminApiState>>>> =
        Arc::new(tokio::sync::Mutex::new(None));
    {
        let holder = admin_state_holder.clone();
        let auth_holder = auth_initial_holder.clone();
        tauri::async_runtime::spawn(async move {
            // Wait up to 5 seconds for the auth to materialise.
            let auth = {
                let mut auth_opt: Option<Arc<auth::AuthState>> = None;
                for _ in 0..50 {
                    if let Some(a) = auth_holder.lock().await.clone() {
                        auth_opt = Some(a);
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
                match auth_opt {
                    Some(a) => a,
                    None    => {
                        log::warn!("admin auth not ready after 5s; loopback HTTP API will start when 'admin_token' command is invoked.");
                        return;
                    }
                }
            };

            let host_id = machine_host_id().unwrap_or_else(|_| "unknown".into());

            // Build the multi-channel router FIRST — the loopback HTTP
            // server (AdminApiState) shares it via a typed Arc<dyn ...>
            // so that /hooks/whatsapp, /api/v1/internal/dispatch and
            // admin POSTs to /api/v1/{start,stop,restart} all dispatch
            // through the same path Telegram/Discord/Slack already use.
            let router_fn = move |ctx: integrations::command_router::RemoteCommandContext,
                                   cmd: integrations::command_router::RemoteCommand| {
                let cfg_path = std::env::var("ARK_ASA_CONFIG_PATH")
                    .unwrap_or_else(|_| "server-config.toml".to_string());
                let cfg_path_pb = std::path::PathBuf::from(cfg_path);
                let cfg = match tauri::async_runtime::block_on(
                    crate::config::loader::ConfigLoader::load_or_default(&cfg_path_pb),
                ) {
                    Ok(c) => c,
                    Err(e) => {
                        return Err::<integrations::command_router::RouterOutcome, String>(
                            format!("config load failed: {e}"),
                        );
                    }
                };
                let _ = ctx;
                let outcome = tauri::async_runtime::block_on(
                    integrations::dispatch(&cfg, cmd.kind, cmd.map_index, cmd.tail),
                );
                match outcome {
                    Ok(o) => Ok(o),
                    Err(e) => Ok(integrations::command_router::RouterOutcome::Error {
                        reason: format!("{e}"),
                    }),
                }
            };
            let router_arc: std::sync::Arc<
                dyn Fn(
                    integrations::command_router::RemoteCommandContext,
                    integrations::command_router::RemoteCommand,
                )
                    -> Result<integrations::command_router::RouterOutcome, String>
                    + Send
                    + Sync
                    + 'static,
            > = std::sync::Arc::new(router_fn);

            let api = Arc::new(integrations::http_api::AdminApiState::new(
                auth, host_id.clone(), router_arc.clone(),
            ));
            if let Err(e) = integrations::http_api::spawn_loopback_server(api.clone(), [127, 0, 0, 1], 8765).await {
                log::error!("loopback HTTP API failed to spawn: {e}");
            } else {
                *holder.lock().await = Some(api);
            }

            // v2.2 — Receipts ledger for chat-channel events.
            {
                let app_data = crate::auth::AuthState::storage_dir();
                install_ledger(&app_data, &host_id);
                if let Some(ledger) = shared_ledger().read().as_ref().cloned() {
                    // Bind the global receipt emitter so every chat adapter
                    // and bridge call can append receipts without jugglery.
                    integrations::receipt_emit::install_emitter(ledger.clone());
                    let _ = ledger.append(
                        serde_json::json!({
                            "kind": "boot",
                            "loopback_port": 8765,
                            "loopback_host_id": host_id,
                        }),
                        receipts::Stage::ChannelIngress,
                    );
                }
            }

            // v2.1 — Convex publisher disabled-by-default; turn on when
            // CONVEX_URL env var is set. Real wiring lives in Hito 3.
            if let Ok(url) = std::env::var("CONVEX_URL") {
                let secret = std::env::var("CONVEX_SHARED_SECRET").unwrap_or_default();
                if !secret.is_empty() {
                    if let Some(api) = holder.lock().await.clone() {
                        let handle = integrations::convex_push::spawn_publisher(api, url, secret).await;
                        log::info!("convex publisher started (handle: {handle:?})");
                    }
                }
            }

            // v2.2 — multi-channel remote admin (router_arc was already
            // constructed above so the loopback HTTP server can share it
            // with /hooks/whatsapp + /api/v1/internal/dispatch).
            // Below: spawn each channel adapter using its dedicated env vars.

            // Telegram
            let bot_cfg = integrations::telegram::TelegramConfig::default();
            if bot_cfg.enabled {
                let bot = integrations::telegram::TelegramBot::new(bot_cfg);
                let _ = tauri::async_runtime::spawn(
                    integrations::telegram::spawn_looper(bot, router_arc.clone()),
                );
                log::info!("telegram bot looper started");
            }

            // v2.2 — Discord bot (real WebSocket gateway).
            let discord_cfg = integrations::discord::DiscordConfig::default();
            if discord_cfg.is_active() {
                let discord_bot = integrations::discord::DiscordBot::new(discord_cfg);
                let _ = tauri::async_runtime::spawn(
                    integrations::discord::spawn_looper(discord_bot, router_arc.clone()),
                );
                log::info!("discord bot gateway started");
            }

            // v2.2 — Slack bot (Socket Mode WebSocket, no public URL required).
            let slack_cfg = integrations::slack::SlackConfig::default();
            if slack_cfg.is_active() {
                let slack_bot = integrations::slack::SlackBot::new(slack_cfg);
                let _ = tauri::async_runtime::spawn(
                    integrations::slack::spawn_looper(slack_bot, router_arc.clone()),
                );
                log::info!("slack socket mode looper started");
            }

            // v2.2 — Audit-log database (default = SQLite at APPDATA).
            let db_cfg = integrations::database::DatabaseConfig::default();
            if db_cfg.is_active() {
                let db_cfg_clone = db_cfg.clone();
                tauri::async_runtime::spawn(async move {
                    match integrations::database::build_dao(&db_cfg_clone).await {
                        Ok(_dao) => {
                            log::info!("audit DAO ready ({})", db_cfg_clone.backend.label());
                        }
                        Err(e) => log::warn!("audit DAO init failed: {e}"),
                    }
                });
            }

            // Convex periodic push was wired at the holder.* branch
            // above (real spawn of `convex_push::spawn_publisher`).
            // No additional periodic-style job here — the publisher
            // itself maintains the 60 s `tokio::time::interval`.

            // v2.1 — Tailscale wizard wiring: a one-shot probe at
            // boot that logs tailscale presence. Real `tailscale up`
            // is triggered by the UI via the Tauri command, never
            // autonomously.
            tauri::async_runtime::spawn(async move {
                let installed = integrations::tailscale::detect_tailscale_cli();
                if installed {
                    log::info!("tailscale CLI binary detected on PATH");
                } else {
                    log::info!(
                        "tailscale CLI not installed — Tailscale wizard \
                         will offer an installation link"
                    );
                }
            });

            // v2.1 — Signal subprocess: real wire-up via
            // `signal::maybe_spawn_subprocess`. If signal-cli is on
            // PATH and the operator configured `phone_e164`, spawn
            // --json daemon. Otherwise log gracefully.
            let signal_cfg = integrations::signal::SignalConfig::from_secrets_or_env();
            if let Some(_handle) =
                integrations::signal::maybe_spawn_subprocess(&signal_cfg) {
                log::info!(
                    "signal subprocess spawned; plug the operator's auth \
                     into signal-cli + add this hook as a 'linked device'"
                );
            } else {
                log::debug!(
                    "signal subprocess skipped (phone_e164 empty or signal-cli missing)"
                );
            }
        });
    }

    tauri::Builder::default()
        .manage(PingState(Mutex::new(None)))
        .manage(tray_state)
        .manage(on_demand_state)
        .setup(|app| {
            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            use tauri::tray::TrayIconBuilder;

            // Start one process-wide publisher for live server, network,
            // version, lifecycle and log snapshots.
            events::spawn_event_publisher(app.handle().clone());

            let show_item = MenuItemBuilder::with_id("show", "Show").build(app)?;
            let sep       = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show_item, &sep, &quit_item]).build()?;

            let tray = TrayIconBuilder::new()
                .icon(tauri::include_image!("icons/icon.png"))
                .tooltip("ARK ASA Server Manager")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "quit" => {
                        let _ = app.emit("tray-quit", ());
                        let app2 = app.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(600));
                            app2.exit(0);
                        });
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    use tauri::tray::TrayIconEvent;
                    if matches!(event, TrayIconEvent::Click { .. }) {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            // In Tauri v2, TrayIcon::drop() removes the icon from the OS tray.
            // We must keep the handle alive for the entire app lifetime.
            // std::mem::forget prevents the destructor from running.
            std::mem::forget(tray);

            // Wire the Stronghold plugin inside setup so we can use
            // `app.path().app_local_data_dir()` for the salt file (per
            // https://v2.tauri.app/plugin/stronghold/ example). The salt
            // must persist across launches under the per-user app data dir,
            // not in temp_dir which the OS may clear.
            let salt_path = app
                .path()
                .app_local_data_dir()
                .expect("app_local_data_dir should resolve on all desktop platforms")
                .join("stronghold-salt.txt");
            log::debug!("stronghold salt path: {salt_path:?}");
            app.handle().plugin(
                tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build()
            )?;

            // Auto-migrate any v1 plaintext `secret_store.toml` files into the
            // OS keyring on first launch of 2.1.0 GA. Idempotent - safe to run
            // every launch. See `secret_store_v2::migrate_secrets`.
            match crate::plugins::secret_store_v2::migrate_secrets() {
                Ok(n) if n > 0 => log::info!("migrated {n} secrets from v1 TOML into keyring"),
                Ok(_) => log::debug!("secret migration: nothing to lift (keyring already primary)"),
                Err(e) => log::warn!("secret migration failed (continuing with v1 fallback): {e}"),
            }

            Ok(())
        })
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Config
            crate::commands::config::load_config,
            crate::commands::config::load_config_or_default,
            crate::commands::config::validate_config,
            crate::commands::config::save_config,
            crate::commands::config::get_default_config,
            crate::commands::config::get_config_schema,
            // Remote admin (v2.1)
            crate::commands::integrations::admin_token,
            crate::commands::integrations::rotate_admin_token,
            crate::commands::integrations::set_admin_feature_flag,
            // Secret-store migration (P2): lift v1 TOML into OS keyring. Auto-run on startup.
            crate::commands::integrations::migrate_secrets,
            // Plugin auth flows (v2.1) — Convex uses CLI bridge, not OAuth
            crate::plugins::convex::begin_convex_link,
            crate::plugins::convex::paste_convex_deploy_key,
            crate::plugins::convex::convex_deploy,
            crate::plugins::convex::convex_status,
            crate::plugins::convex::convex_push_schema,
            // Vercel: also CLI-bridge (uses VERCEL_TOKEN env var)
            crate::plugins::vercel::begin_vercel_link,
            crate::plugins::vercel::paste_vercel_token,
            crate::plugins::vercel::vercel_deploy_web,
            crate::plugins::vercel::vercel_deploy_one_click,
            crate::plugins::vercel::vercel_status,
            // Plugin Hub (Session 6 / P1) — list / enable / disable
            crate::plugins::pluginhub::list_plugin_catalog,
            crate::plugins::pluginhub::enable_plugin,
            crate::plugins::pluginhub::disable_plugin,
            crate::plugins::pluginhub::plugin_registry_snapshot,
            // Marketplace install state — operator chooses which plugins
            // they want in their setup. Persisted in server-config.toml
            // (`installed_plugins: Vec<String>`).
            crate::plugins::pluginhub::set_plugin_installed,
            // Plugin runtime hooks (Session 9) — surfaces what each
            // plugin's start() should be doing right now.
            crate::plugins::runtime_hooks::runtime_status,
            // WhatsApp (Session 7)
            crate::plugins::whatsapp_bridge::begin_whatsapp_link,
            crate::plugins::whatsapp_bridge::paste_whatsapp_credentials,
            crate::plugins::whatsapp_bridge::whatsapp_status,
            // Signal / WeChat / SSH / REST (Session 8)
            crate::plugins::extra_bridges::paste_signal_credentials,
            crate::plugins::extra_bridges::signal_status,
            crate::plugins::extra_bridges::paste_wechat_credentials,
            crate::plugins::extra_bridges::wechat_status,
            crate::plugins::extra_bridges::paste_ssh_credentials,
            crate::plugins::extra_bridges::ssh_status,
            crate::plugins::extra_bridges::paste_rest_credentials,
            crate::plugins::extra_bridges::rest_status,
            // Connection plugin catalog (Session 6 / P2) — declarative
            // metadata for the 7 VPS providers. Layered on top of the
            // existing `HostProvider` enum so the React-side contracts
            // and tests keep working unchanged.
            crate::plugins::connection::list_connection_plugins,
            crate::plugins::connection::get_connection_plugin,
            // AI model plugin catalog (Session 6 / P3) — 8 OpenAI-API-
            // compatible adapters (OpenAI, Cerebras, NVIDIA NIM, llama.cpp,
            // Ollama, vLLM, LM Studio, Custom). Discoverable shape only;
            // `integrations::ai::AiClient` already speaks OpenAI chat
            // completions so all 8 work transparently.
            crate::plugins::model::list_model_plugins,
            crate::plugins::model::get_model_plugin,
            // CurseForge
            crate::commands::mods::get_curseforge_api_key,
            crate::commands::mods::set_curseforge_api_key,
            crate::commands::mods::fetch_curseforge_mods,
            crate::commands::mods::get_curseforge_mod_by_id,
            crate::commands::mods::check_mods_available,
            crate::commands::mods::check_client_only_mods,
            crate::commands::mods::clear_mods_cache,
            // Server control
            crate::commands::server::server_status,
            crate::commands::server::start_server,
            crate::commands::server::stop_server,
            crate::commands::server::is_server_running,
            crate::commands::server::get_cluster_instance_status,
            crate::commands::server::start_server_instance,
            crate::commands::server::stop_server_instance,
            crate::commands::server::read_text_file,
            crate::commands::server::write_text_file,
            crate::commands::server::merge_config_from_ini,
            crate::commands::server::restart_server,
            crate::commands::server::get_server_logs,
            crate::commands::server::get_server_metrics,
            // Cloud backup
            backup::backup_saves,
            backup::read_backup_metadata,
            backup::read_server_log,
            backup::start_gdrive_oauth,
            backup::start_onedrive_oauth,
            backup::refresh_gdrive_token,
            backup::refresh_onedrive_token,
            backup::test_s3_connection,
            backup::list_cloud_backups,
            backup::restore_backup_from_cloud,
            // IP detection
            crate::commands::integrations::detect_ips,
            crate::commands::network::check_internet,
            // Diagnostics & repair for the "advertises + IP connect OK but
            // invisible in in-game Unofficial PC list" ASA symptom.
            crate::ark::diagnostics::diagnose_server_list,
            // Version sync (Steam app 2430930)
            crate::commands::integrations::check_server_version,
            crate::commands::integrations::update_server,
            // Tailscale wizard (v2.1, Network blocker #4)
            crate::commands::integrations::tailscale_installed,
            crate::commands::integrations::tailscale_download_url,
            crate::commands::integrations::tailscale_status_combined,
            crate::commands::integrations::tailscale_setup,
            crate::commands::integrations::parse_config_from_toml,
            crate::commands::integrations::config_to_toml,
            crate::commands::receipts::parse_config_from_zip,
            // v2.2 — multi-cloud hosting & database adapters
            crate::commands::integrations::render_hosting_script,
            crate::commands::integrations::render_provider_run_script,
            crate::commands::integrations::render_local_provision_plan,
            crate::commands::integrations::list_hosting_providers,
            crate::commands::integrations::list_database_backends,
            crate::commands::integrations::validate_database_config,
            crate::commands::integrations::record_hosting_deployment,
            // v2.2 — Receipts ledger
            crate::commands::receipts::receipts_probe,
            crate::commands::receipts::receipts_today_path,
            crate::commands::receipts::receipts_tail,
            // Ping / Tailscale
            crate::commands::utilities::start_ping,
            crate::commands::utilities::stop_ping,
            // Utilities
            crate::commands::utilities::open_external_url,
            // Tray
            crate::commands::app::set_minimize_to_tray,
            crate::commands::app::quit_app,
            // On-demand stubs
            crate::commands::app::enable_on_demand,
            crate::commands::app::disable_on_demand,
            crate::commands::app::disable_all_on_demand,
            crate::commands::app::get_on_demand_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
