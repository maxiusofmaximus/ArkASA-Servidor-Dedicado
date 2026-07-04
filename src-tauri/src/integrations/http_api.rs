//! Loopback HTTP API the Convex backend (and bots that route through
//! Convex) call to control / observe the ARK server.
//!
//! Listens ONLY on `127.0.0.1:8765` — never exposed to the LAN or internet.
//! Convex reaches it via the Vercel proxy tier (see Hito 11) or via a
//! developer-only SSH tunnel.
//!
//! Endpoints (all `application/json`):
//!   GET  /api/v1/health                  → 200 ok
//!   GET  /api/v1/state                   → cluster snapshot (status / maps)
//!   GET  /api/v1/logs?tail=N             → last N server log lines
//!   GET  /api/v1/config                  → full TOML config
//!   POST /api/v1/start                   (admin) start {map_index?}
//!   POST /api/v1/stop                    (admin) stop {map_index?}
//!   POST /api/v1/restart                 (admin) restart {map_index?}
//!   POST /api/v1/push_state  (admin)     manually force a Convex push
//!   POST /api/v1/internal/dispatch      (service-to-service) accepts
//!                                            a normalized RemoteCommand
//!                                            from Convex internal action
//!
//! Auth: every request must carry `Authorization: Bearer <token>` where
//! `<token>` is the active token from `AuthState::active_token()`. Failed
//! auth → 401 with `WWW-Authenticate: Bearer`.

use crate::auth::AuthState;
use crate::config::schema::ServerConfig;
use crate::integrations::command_router::{
    authorize, CommandKind, RemoteCommand, RemoteCommandContext, RouterOutcome,
    Role as RouterRole,
};
use axum::{
    extract::{Query, State as AxState},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapStatusDto {
    pub map_index: usize,
    pub map_id:    String,
    pub map_label: String,
    pub running:   bool,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct StateSnapshot {
    pub cluster_running: bool,
    pub map_statuses:    Vec<MapStatusDto>,
    pub primary_ip:      Option<String>,
    pub last_update_ms:  i64,
}

/// Shared state held by the HTTP server. The Tauri app populates
/// `config_snapshot` once `load_config_or_default` finishes, and updates
/// `state` whenever the cluster status poll refreshes.
#[derive(Clone)]
pub struct AdminApiState {
    pub auth: Arc<AuthState>,
    pub host_id: String,
    pub state: Arc<RwLock<StateSnapshot>>,
    pub config_snapshot: Arc<RwLock<Option<ServerConfig>>>,
}

impl AdminApiState {
    pub fn new(auth: Arc<AuthState>, host_id: String) -> Self {
        Self {
            auth,
            host_id,
            state: Arc::new(RwLock::new(StateSnapshot::default())),
            config_snapshot: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn snapshot(&self) -> StateSnapshot {
        self.state.read().await.clone()
    }
}

pub async fn spawn_loopback_server(
    api: Arc<AdminApiState>,
    host: [u8; 4],
    port: u16,
) -> Result<tokio::task::JoinHandle<()>, String> {
    let auth_for_layer = api.auth.clone();
    let app = Router::new()
        .route("/api/v1/health",       get(health))
        .route("/api/v1/state",        get(get_state))
        .route("/api/v1/logs",         get(get_logs))
        .route("/api/v1/config",       get(get_config))
        .route("/api/v1/start",        post(post_start))
        .route("/api/v1/stop",         post(post_stop))
        .route("/api/v1/restart",      post(post_restart))
        .route("/api/v1/internal/dispatch", post(internal_dispatch))
        .route("/api/v1/internal/auth-check", get(auth_check))
        .layer(axum::middleware::from_fn(move |req, next| {
            let auth = auth_for_layer.clone();
            async move { run_auth(req, next, auth).await }
        }));

    let addr: SocketAddr = format!("{}:{}", host.iter().map(|b| b.to_string()).collect::<Vec<_>>().join("."), port)
        .parse()
        .map_err(|e| format!("bad addr: {e}"))?;

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("bind {addr}: {e}"))?;
    log::info!("Admin HTTP API listening on http://{addr}");

    let handler = tokio::spawn(async move {
        let app_with_state = app.with_state(api);
        if let Err(e) = axum::serve(listener, app_with_state.into_make_service()).await {
            log::error!("admin HTTP api exited: {e}");
        }
    });
    Ok(handler)
}

// ── Auth middleware (axum 0.7 signature: Request, Next → Future) ────────
async fn run_auth(
    req: axum::extract::Request,
    next: axum::middleware::Next,
    auth: Arc<AuthState>,
) -> Response {
    let path = req.uri().path().to_string();
    if path == "/api/v1/health" {
        return next.run(req).await;
    }

    let header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let Some(h) = header else {
        return unauthorized("missing Authorization header");
    };

    match auth.validate(&h) {
        Ok(_role) => next.run(req).await,
        Err(e)    => unauthorized(&e),
    }
}

/// Used by Convex internal actions to confirm the JWT they hold is valid.
async fn auth_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

fn unauthorized(reason: &str) -> Response {
    let mut resp = (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": reason })),
    ).into_response();
    resp.headers_mut().insert(
        "WWW-Authenticate",
        "Bearer realm=\"ark-asa-admin\"".parse().unwrap(),
    );
    resp
}

// ── Handlers ────────────────────────────────────────────────────────────
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

async fn get_state(
    AxState(api): AxState<Arc<AdminApiState>>,
) -> Json<StateSnapshot> {
    let snap = api.snapshot().await;
    Json(snap)
}

#[derive(Deserialize)]
struct LogsQuery { #[serde(default = "default_tail")] tail: usize }

fn default_tail() -> usize { 100 }

async fn get_logs(
    AxState(_api): AxState<Arc<AdminApiState>>,
    Query(q): Query<LogsQuery>,
) -> Response {
    // Hito 12: read from the running on-disk log file; for now we return a
    // empty list — the publisher contract says "best effort", not "always
    // populated at Hito 2".
    let lines: Vec<String> = Vec::new();
    (StatusCode::OK, Json(serde_json::json!({
        "tail": q.tail.min(1000),
        "lines": lines,
    }))).into_response()
}

#[derive(Serialize)]
struct ConfigResponse {
    toml: String,
}

async fn get_config(
    AxState(api): AxState<Arc<AdminApiState>>,
) -> Response {
    let cfg = api.config_snapshot.read().await;
    match cfg.as_ref() {
        Some(c) => match toml::to_string_pretty(c) {
            Ok(s)  => (StatusCode::OK, Json(ConfigResponse { toml: s })).into_response(),
            Err(e) => err_500(format!("serialize: {e}")),
        },
        None    => err_500("config not loaded yet".into()),
    }
}

#[derive(Deserialize, Default)]
struct ActionBody {
    map_index: Option<u32>,
}

async fn admin_only_call(
    api: Arc<AdminApiState>,
    headers: HeaderMap,
    body: ActionBody,
    kind: CommandKind,
) -> Response {
    let bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_default();
    let raw_role = match api.auth.validate(&bearer) {
        Ok(r)  => r,
        Err(_) => return unauthorized("bad role for admin op"),
    };
    let role = match raw_role {
        crate::auth::Role::Admin  => RouterRole::Admin,
        crate::auth::Role::Viewer => RouterRole::Viewer,
    };
    if role != RouterRole::Admin { return unauthorized("admin role required"); }

    let ctx = RemoteCommandContext {
        channel: crate::integrations::command_router::Channel::Web,
        actor_id: api.host_id.clone(),
        actor_name: "http-api".into(),
        role,
        identity: None,
    };
    let cmd = RemoteCommand {
        kind,
        map_index: body.map_index,
        config_patch: None,
        tail: None,
    };
    if let Err(e) = authorize(&ctx, &cmd) { return err_500(format!("forbidden: {e}")); }

    // Hito 12: invoke the actual launcher here. For now we synthesise a
    // placeholder outcome so the wire contracts are usable end-to-end.
    let outcome = RouterOutcome::Error {
        reason: "start/stop/restart bridged to launcher in lib.rs at Hito 12".into(),
    };

    (StatusCode::OK, Json(outcome)).into_response()
}

async fn post_start(
    AxState(api): AxState<Arc<AdminApiState>>,
    headers: HeaderMap,
    Json(body): Json<ActionBody>,
) -> Response {
    admin_only_call(api, headers, body, CommandKind::Start).await
}
async fn post_stop(
    AxState(api): AxState<Arc<AdminApiState>>,
    headers: HeaderMap,
    Json(body): Json<ActionBody>,
) -> Response {
    admin_only_call(api, headers, body, CommandKind::Stop).await
}
async fn post_restart(
    AxState(api): AxState<Arc<AdminApiState>>,
    headers: HeaderMap,
    Json(body): Json<ActionBody>,
) -> Response {
    admin_only_call(api, headers, body, CommandKind::Restart).await
}

#[derive(Deserialize)]
struct InternalDispatchBody {
    cmd:     RemoteCommand,
    context: RemoteCommandContext,
}

async fn internal_dispatch(
    AxState(api): AxState<Arc<AdminApiState>>,
    Json(body): Json<InternalDispatchBody>,
) -> Response {
    // This is the path Convex internal actions call (`/api/v1/internal/dispatch`)
    // — the JWT carries the canonical role assigned by the Convex auth tier.
    let bearer_from_convex = body.context.role;
    if !matches!(bearer_from_convex, RouterRole::Admin) {
        return unauthorized("convex context must be admin for now");
    }
    if let Err(e) = authorize(&body.context, &body.cmd) {
        return err_500(format!("forbidden by authorize: {e}"));
    }

    // Safety net: only resolve within 5 s; otherwise return a clear error
    // rather than letting the Convex action hang.
    let outcome = match timeout(
        Duration::from_secs(5),
        async {
            // Hito 12: bridge to the launcher. For now we return a stub.
            Ok::<_, String>(RouterOutcome::Error {
                reason: "internal_dispatch bridged at Hito 12".into(),
            })
        },
    ).await {
        Ok(r)   => r.unwrap_or_else(|e| RouterOutcome::Error { reason: format!("bridge: {e}") }),
        Err(_)  => RouterOutcome::Error { reason: "timeout executing command".into() },
    };

    let _ = api; // keep alive — Hito 12 will use api here
    (StatusCode::OK, Json(outcome)).into_response()
}

fn err_500(reason: String) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": reason }))).into_response()
}
