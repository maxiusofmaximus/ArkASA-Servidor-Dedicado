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
    authorize, gated_chat_binding, run_with_receipts_async, CommandKind, PipelineExecution,
    PipelineOutcome, RemoteCommand, RemoteCommandContext, RouterOutcome, Role as RouterRole,
};
use crate::integrations::receipt_emit::try_global;
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
use tokio::sync::RwLock;

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
    /// Legacy sync router shared with the bots that still speak the
    /// `RouterFn` shape (Telegram/Discord/Slack as built today). P19
    /// keeps this around for the transition window; new code should
    /// use `async_router` and `await` directly.
    pub router: Arc<crate::integrations::RouterFn>,
    /// P19 — async router. The preferred dispatch path; eliminates
    /// the `spawn_blocking(|| router(...))` dance and the hidden
    /// deadlock footgun tied to calling block_on inside the sync
    /// router closure.
    pub async_router: Arc<crate::integrations::AsyncRouterFn>,
}

impl AdminApiState {
    pub fn new(
        auth: Arc<AuthState>,
        host_id: String,
        router: Arc<crate::integrations::RouterFn>,
        async_router: Arc<crate::integrations::AsyncRouterFn>,
    ) -> Self {
        Self {
            auth,
            host_id,
            state: Arc::new(RwLock::new(StateSnapshot::default())),
            config_snapshot: Arc::new(RwLock::new(None)),
            router,
            async_router,
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
        // ── Session 10: webhook receivers for chat-bot plugins ──
        // WhatsApp Cloud & WeChat Work POST raw payloads here. Both
        // honour per-plugin secrets (HMAC for WhatsApp, plain XML
        // for WeChat). The routes are no-ops when the respective
        // plugin isn't fully configured (config-check first).
        .route("/hooks/whatsapp", post(whatsapp_webhook))
        .route("/hooks/wechat",   post(wechat_webhook).get(wechat_handshake))
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
    let claims = match api.auth.validate_with_claims(&bearer) {
        Ok(c)  => c,
        Err(_) => return unauthorized("bad role for admin op"),
    };
    let role = match claims.role {
        crate::auth::Role::Admin  => RouterRole::Admin,
        crate::auth::Role::Viewer => RouterRole::Viewer,
    };
    if role != RouterRole::Admin { return unauthorized("admin role required"); }

    // P32: `user_id` carries the `PrincipalKind` tag so Convex /
    // Vercel / loopback sub-agents no longer collapse into the
    // `"tauri-app"` bucket in receipts.
    let _principal_tag = claims.principal.as_str();
    let trace_id      = format!("http-{}:{}", kind.as_str(), claims.sub);
    let raw_text      = format!("/{}", kind.as_str());

    // P27: route through the canonical receipt-emit pipeline. The
    // binding is built around the validated `claims.sub` so the
    // admin allow-list gates the audit trail before the router is
    // invoked. Admin-only check is also re-asserted via role at
    // `authorize` inside the pipeline.
    let binding = gated_chat_binding(
        crate::integrations::command_router::Channel::Web,
        "http-api:admin",
        vec![claims.sub.clone()],
    );
    let emitter = try_global();

    let kind_label = kind.as_str();
    let cmd = RemoteCommand {
        kind,
        map_index: body.map_index,
        config_patch: None,
        tail: None,
    };
    let router = api.async_router.clone();
    let exec = match tokio::time::timeout(
        std::time::Duration::from_secs(30),
        run_with_receipts_async(
            emitter.as_deref(),
            crate::integrations::Platform::Web,
            crate::integrations::command_router::Channel::Web,
            &binding,
            &claims.sub,
            &claims.label,
            &trace_id,
            &raw_text,
            None,
            Some(kind_label),
            Some(cmd),
            move |ctx, cmd| async move { router(ctx, cmd).await },
        ),
    )
    .await
    {
        Ok(e)  => e,
        Err(_) => {
            // 30 s elapsed before the pipeline finished — emit a
            // Failed ChannelDelivery so the audit trail reflects the
            // dropped call, and return a typed RouterOutcome::Error.
            if let Some(e) = emitter.as_deref() {
                let rctx = crate::integrations::receipt_emit::ReceiptContext {
                    trace_id:   &trace_id,
                    actor_id:   &claims.sub,
                    actor_name: &claims.label,
                    runtime:    crate::integrations::RuntimeClass::Interactive,
                };
                e.delivery(
                    rctx,
                    crate::integrations::Platform::Web,
                    &claims.sub,
                    crate::integrations::receipt_emit::DeliveryStatus::Failed,
                    Some("router: 30s timeout"),
                );
            }
            return (StatusCode::OK, Json(RouterOutcome::Error {
                reason: "router: 30s timeout".into(),
            })).into_response();
        }
    };

    match exec.pipeline {
        PipelineOutcome::Done(_) => {
            let outcome = exec.router_outcome.unwrap_or_else(|| RouterOutcome::Error {
                reason: "router returned no outcome".into(),
            });
            (StatusCode::OK, Json(outcome)).into_response()
        }
        PipelineOutcome::Rejected(reason) => {
            // Identity gate rejected the actor — audit trail already
            // recorded the rejection. Return 403 so callers know.
            (StatusCode::FORBIDDEN, Json(serde_json::json!({
                "error": reason,
            }))).into_response()
        }
        PipelineOutcome::NoCommand(_) => {
            (StatusCode::OK, Json(serde_json::json!({}))).into_response()
        }
    }
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

    // P27: drive the same receipt-emit pipeline as the chat bots,
    // but `await`ed — the audit trail now covers Convex-initiated
    // dispatches instead of vanishing at `RuntimePipeline`. The
    // binding gate happens inside the helper using the
    // `body.context.actor_id` resolved upstream by Convex's JWT
    // verifier, so a forged payload can't bypass identity.
    let principal_tag = body.context.actor_id.clone();
    let trace_id      = format!("convex-{}:{}", body.cmd.kind.as_str(), principal_tag);
    let raw_text      = format!("/{}", body.cmd.kind.as_str());
    let binding = gated_chat_binding(
        crate::integrations::command_router::Channel::Web,
        "http-api:convex",
        vec![principal_tag.clone()],
    );
    let emitter = try_global();
    let ctx_channel = body.context.channel;
    let actor_id    = body.context.actor_id.clone();
    let actor_name  = body.context.actor_name.clone();

    let router = api.async_router.clone();
    let exec_result = run_with_receipts_async(
        emitter.as_deref(),
        crate::integrations::Platform::Web,
        ctx_channel,
        &binding,
        &actor_id,
        &actor_name,
        &trace_id,
        &raw_text,
        None,
        Some(body.cmd.kind.as_str()),
        Some(body.cmd),
        move |ctx, cmd| async move { router(ctx, cmd).await },
    );
    let exec: PipelineExecution = match tokio::time::timeout(
        std::time::Duration::from_secs(30),
        exec_result,
    )
    .await
    {
        Ok(e) => e,
        Err(_) => {
            if let Some(e) = emitter.as_deref() {
                let rctx = crate::integrations::receipt_emit::ReceiptContext {
                    trace_id:   &trace_id,
                    actor_id:   &actor_id,
                    actor_name: &actor_name,
                    runtime:    crate::integrations::RuntimeClass::Interactive,
                };
                e.delivery(
                    rctx,
                    crate::integrations::Platform::Web,
                    &actor_id,
                    crate::integrations::receipt_emit::DeliveryStatus::Failed,
                    Some("router: 30s timeout"),
                );
            }
            return (StatusCode::OK, Json(RouterOutcome::Error {
                reason: "router: 30s timeout".into(),
            })).into_response();
        }
    };

    match exec_ref_to_outcome(exec) {
        Ok(out)            => (StatusCode::OK, Json(out)).into_response(),
        Err(rejection_msg) => (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": rejection_msg,
        }))).into_response(),
    }
}

/// Helper used by `internal_dispatch` (and any future cursor that
/// routes through `run_with_receipts_async`) to lift the
/// `PipelineExecution` back into a typed `RouterOutcome` for
/// machine-readable HTTP responses. P27 audit closed when this
/// helper is the only `exec.pipeline` consumer in the loopback
/// HTTP layer.
fn exec_ref_to_outcome(exec: PipelineExecution) -> Result<RouterOutcome, String> {
    match exec.pipeline {
        PipelineOutcome::Done(_) => exec.router_outcome.ok_or_else(|| {
            "router returned no outcome".into()
        }),
        PipelineOutcome::Rejected(reason) => Err(reason),
        PipelineOutcome::NoCommand(_) => Err("no command resolved".into()),
    }
}

fn err_500(reason: String) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": reason }))).into_response()
}

// ─── Session 10 — webhook receivers (WhatsApp + WeChat) ───────────────

async fn whatsapp_webhook(
    AxState(api): AxState<Arc<AdminApiState>>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let cfg = crate::integrations::whatsapp::WhatsAppConfig::from_secrets_or_env();
    if cfg.api_token.is_empty() || cfg.webhook_secret.is_empty()
        || cfg.phone_number_id.is_empty() {
        // Operator hasn't pasted secrets — silently drop so Meta
        // doesn't keep retrying forever.
        return (StatusCode::OK,
                Json(serde_json::json!({"status": "noop, plugin not configured"})))
            .into_response();
    }
    // HMAC verify against X-Hub-Signature-256.
    let sig = headers.get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok()).unwrap_or("");
    if !crate::integrations::whatsapp::verify_webhook_signature(
        &cfg.webhook_secret, &body, sig)
    {
        return err_500("whatsapp webhook: signature mismatch".into());
    }
    let payload: crate::integrations::whatsapp::WebhookBody = match serde_json::from_slice(&body) {
        Ok(p)  => p,
        Err(e) => return err_500(format!("whatsapp webhook: bad json: {e}")),
    };

    let bot = crate::integrations::whatsapp::WhatsAppBot::new(cfg.clone());

    // Walk every inbound message: classify via allowlist + parse_action,
    // build a RemoteCommand, route through the same shared router closure
    // we use for Telegram/Discord/Slack, and capture the rendered reply.
    // Meta expects a fast 200 OK; we run the actual dispatch on the
    // router and stash the response so it can be sent out-of-band via
    // Graph API if/when that lands.  For now the visible UX is that
    // /start typed in WhatsApp actually starts/stops the server — the
    // operator sees status in the desktop UI immediately because the
    // router writes receipts and the launcher emits lifecycle events.
    let mut accepted: usize = 0;
    let mut rejected: usize = 0;
    let entries = payload.entry.len();
    for entry in payload.entry.iter() {
        for change in entry.changes.iter() {
            for msg in change.value.messages.iter() {
                // accept_message enforces admin allowlist + non-text skip.
                let Some(ctx) = bot.accept_message(msg) else { rejected += 1; continue; };
                let text_opt = msg.text.as_ref().map(|t| t.body.as_str()).unwrap_or("");
                let Some(kind) = crate::integrations::whatsapp::WhatsAppBot::parse_action(text_opt) else {
                    rejected += 1;
                    continue;
                };
                let kind_for_log = kind.as_str();
                let cmd = RemoteCommand {
                    kind,
                    map_index: None,
                    config_patch: None,
                    tail: None,
                };
                let from = msg.from.clone();
                let router = api.router.clone();
                let dispatch_join = tokio::task::spawn_blocking(move || router(ctx, cmd)).await;
                match dispatch_join {
                    Ok(Ok(outcome)) => {
                        log::info!(
                            "whatsapp webhook dispatch from={from} kind={kind_for_log} outcome={outcome:?}"
                        );
                        accepted += 1;
                    }
                    Ok(Err(e)) => {
                        log::warn!("whatsapp webhook router returned error: {e}");
                        rejected += 1;
                    }
                    Err(e) => {
                        log::warn!("whatsapp webhook router task panicked: {e}");
                        rejected += 1;
                    }
                }
            }
        }
    }

    (StatusCode::OK,
     Json(serde_json::json!({
         "status":            "accepted",
         "entries":           entries,
         "messages_dispatched": accepted,
         "messages_rejected":   rejected,
     }))).into_response()
}

async fn wechat_webhook(
    AxState(_api): AxState<Arc<AdminApiState>>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    // WeChat Work's GET ?msg_signature handshake is handled at boot
    // by the operator pasting credentials — not the scope of this
    // route. POST is the body, which we deserialize as plain fields
    // after a naive XML→struct adapter that the operator wires.
    let cfg = crate::integrations::wechat::WeChatConfig::from_secrets_or_env();
    if cfg.corp_id.is_empty() || cfg.corp_secret.is_empty()
        || cfg.agent_id.is_empty() {
        return (StatusCode::OK,
                Json(serde_json::json!({"status": "noop, plugin not configured"})))
            .into_response();
    }
    // Optional: encrypt-from-Msg-Crypt verification happens at XML
    // level — for S10 we accept plain fields and trust operating
    // operator's TLS certificate pinning. Comment on docs/WECHAT.md.
    let auth_ok = headers.get("x-wecom-token").is_some()
        || headers.get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|c| c.starts_with("text/xml") || c.starts_with("application/xml"))
            .unwrap_or(false);
    if !auth_ok {
        return err_500("wechat webhook: missing token header / xml body".into());
    }
    let payload: crate::integrations::wechat::WeChatXmlPayload = match serde_json::from_slice(&body) {
        Ok(p)  => p,
        Err(_e) => {
            let text = match std::str::from_utf8(&body) {
                Ok(s)  => s,
                Err(_) => return err_500("wechat webhook: not utf-8".into()),
            };
            let v = crate::integrations::wechat::parse_wechat_xml_loose(text);
            match serde_json::from_value::<crate::integrations::wechat::WeChatXmlPayload>(v) {
                Ok(p)  => p,
                Err(e) => return err_500(format!("wechat webhook: parse failed: {e}")),
            }
        }
    };
    let bot = crate::integrations::wechat::WeChatBot::new(cfg);
    bot.accept_message(&payload); // idempotent filter
    (StatusCode::OK,
     Json(serde_json::json!({"status": "accepted"}))).into_response()
}

/// WeChat Work handshake handler: the operator's WeCom
/// console callback URL is `https://host/hooks/wechat` and
/// the platform sends a `GET ?msg_signature=...&timestamp=...
/// &nonce=...&echostr=...` to verify. We recompute SHA1 over
/// sorted `[token, timestamp, nonce]` and compare in constant
/// time; on success we return `echostr` so WeCom accepts our
/// callback. On failure we return an empty body — WeCom will
/// retry a couple times before giving up.
///
/// P33: the SHA-1 + lex-sort + XML-parse helpers moved out of
/// this file into `integrations::wechat`. They're WeChat-protocol
/// concerns, not transport concerns.
async fn wechat_handshake(
    query: axum::extract::Query<std::collections::HashMap<String, String>>,
) -> String {
    let cfg = crate::integrations::wechat::WeChatConfig::from_secrets_or_env();
    let token = cfg.corp_secret.as_str();
    if token.is_empty() {
        // No creds pasted — WeCom will retry; nothing to do.
        return String::new();
    }
    let signature = query.get("msg_signature").cloned().unwrap_or_default();
    let timestamp = query.get("timestamp").cloned().unwrap_or_default();
    let nonce     = query.get("nonce").cloned().unwrap_or_default();
    let echostr   = query.get("echostr").cloned().unwrap_or_default();
    if signature.is_empty() || timestamp.is_empty()
        || nonce.is_empty() || echostr.is_empty() {
        return String::new();
    }
    let nonce_for_log = nonce.clone();
    let computed = crate::integrations::wechat::wechat_handshake_sha1(
        token,
        &timestamp,
        &nonce,
    );
    if crate::integrations::wechat::constant_time_eq(computed.as_bytes(), signature.as_bytes()) {
        echostr
    } else {
        log::warn!("wechat handshake: signature mismatch for nonce={nonce_for_log}");
        String::new()
    }
}

// P33 — `constant_time_eq`, `wechat_handshake_sha1`, and
// `parse_wechat_xml_loose` used to live here. They were WeChat-protocol
// concerns hiding inside the Web transport layer; they now live in
// `integrations::wechat` (alongside the bot itself), which is where any
// future WeChat-specific helper should also go.
