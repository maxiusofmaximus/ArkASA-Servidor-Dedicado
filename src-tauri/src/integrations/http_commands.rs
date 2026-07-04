//! Public HTTP REST command surface (Hito 11).
//!
//! Distinct from `http_api.rs` so the local-only loopback server does not
//! accidentally expose itself. Activation requires an explicit
//! `[integrations] http_admin_enabled = true` flag in TOML.
//!
//! Built on `axum` again so the middleware/router pattern is identical and
//! easy to audit. Listening port: configurable, default 8766 (one above the
//! loopback port so they never collide).

use crate::auth::AuthState;
use crate::integrations::command_router::{
    CommandKind, MapDigest, RemoteCommand, RemoteCommandContext, RouterOutcome,
};
use crate::integrations::http_api::AdminApiState;
use axum::{
    middleware::{self, Next},
    extract::State as AxState,
    http::{Request, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;

pub async fn spawn_public_server(
    api: Arc<AdminApiState>,
    host:   [u8; 4],
    port:   u16,
) -> Result<tokio::task::JoinHandle<()>, String> {
    let auth = api.auth.clone();
    let app = Router::new()
        .route("/api/v1/health",  get(health))
        .route("/api/v1/status",  get(status))
        .route("/api/v1/logs",    get(logs))
        .route("/api/v1/start",   post(start))
        .route("/api/v1/stop",    post(stop))
        .layer(middleware::from_fn_with_state(auth, auth_layer));

    let addr: SocketAddr = format!("{}:{}", host.iter().map(|b| b.to_string()).collect::<Vec<_>>().join("."), port)
        .parse()
        .map_err(|e| format!("bad public addr: {e}"))?;
    let listener = tokio::net::TcpListener::bind(&addr).await
        .map_err(|e| format!("bind {addr} public: {e}"))?;
    log::info!("Public HTTP API listening on http://{addr}");
    Ok(tokio::spawn(async move {
        let app_with_state = app.with_state(api);
        if let Err(e) = axum::serve(listener, app_with_state.into_make_service()).await {
            log::error!("public HTTP api exited: {e}");
        }
    }))
}

async fn auth_layer(
    AxState(auth): AxState<Arc<AuthState>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();
    if path == "/api/v1/health" { return next.run(req).await; }
    let header = req.headers().get("authorization")
        .and_then(|v| v.to_str().ok()).map(|s| s.to_string());
    let Some(h) = header else { return unauthorized("missing Authorization header"); };
    match auth.validate(&h) {
        Ok(_) => next.run(req).await,
        Err(e) => unauthorized(&e),
    }
}
fn unauthorized(reason: &str) -> Response {
    let mut resp = (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": reason }))).into_response();
    resp.headers_mut().insert(
        "WWW-Authenticate",
        "Bearer realm=\"ark-asa-public\"".parse().unwrap(),
    );
    resp
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}
async fn status(AxState(api): AxState<Arc<AdminApiState>>) -> Json<serde_json::Value> {
    let snap = api.snapshot().await;
    Json(serde_json::json!({
        "running": snap.cluster_running,
        "maps":    snap.map_statuses,
    }))
}
async fn logs() -> Json<serde_json::Value> {
    // Hito 12 will stream real log lines. For now we expose the same shape
    // so the Vercel proxy can integrate without changes.
    Json(serde_json::json!({ "tail": 50, "lines": Vec::<String>::new() }))
}

#[derive(serde::Deserialize, Default, Debug)]
struct ActionBody {
    /// Optional map index. The Hito-12 stubs currently ignore this —
    /// the real implementation in `http_api.rs` routes through the
    /// command router. Marked `#[allow(dead_code)]` so we don't drift
    /// the deserializer as soon as the body grows.
    #[allow(dead_code)]
    map_index: Option<u32>,
}

async fn start(
    AxState(_api): AxState<Arc<AdminApiState>>,
    Json(body): Json<ActionBody>,
) -> (StatusCode, Json<RouterOutcome>) {
    log::debug!("public /v1/start called with {body:?}");
    (StatusCode::OK, Json(stub_outcome(CommandKind::Start)))
}
async fn stop(
    AxState(_api): AxState<Arc<AdminApiState>>,
    Json(body): Json<ActionBody>,
) -> (StatusCode, Json<RouterOutcome>) {
    log::debug!("public /v1/stop called with {body:?}");
    (StatusCode::OK, Json(stub_outcome(CommandKind::Stop)))
}
fn stub_outcome(kind: CommandKind) -> RouterOutcome {
    RouterOutcome::Error { reason: format!("public {:?} bridged at Hito 12", kind) }
}

// Silence unused-import warnings for adapters wired in later hitos.
#[allow(dead_code)]
fn _ensure_imports(_ctx: RemoteCommandContext, _cmd: RemoteCommand, _mid: MapDigest) {}
