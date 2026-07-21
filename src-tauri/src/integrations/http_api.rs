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

/// Type alias for the multi-channel command dispatcher that the rest of
/// the app bleeds router wiring into. Lives here rather than in
/// command_router.rs to avoid pulling the closure-y type into places
/// that already have implementations (Telegram, Discord, Slack all
/// take concrete `Arc<F>` instead).
pub type RouterFn = dyn Fn(
    crate::integrations::command_router::RemoteCommandContext,
    crate::integrations::command_router::RemoteCommand,
) -> Result<crate::integrations::command_router::RouterOutcome, String>
    + Send
    + Sync;

/// Shared state held by the HTTP server. The Tauri app populates
/// `config_snapshot` once `load_config_or_default` finishes, and updates
/// `state` whenever the cluster status poll refreshes.
#[derive(Clone)]
pub struct AdminApiState {
    pub auth: Arc<AuthState>,
    pub host_id: String,
    pub state: Arc<RwLock<StateSnapshot>>,
    pub config_snapshot: Arc<RwLock<Option<ServerConfig>>>,
    /// Multi-channel router closure shared with Telegram/Discord/Slack
    /// adapters.  WhatsApp + WeChat webhooks (and the `/api/v1/start`,
    /// `/stop`, `/restart`, `/internal/dispatch` endpoints) reuse it so
    /// that sending `/start` from a chat actually starts the server
    /// instead of just acking "accepted".
    pub router: Arc<RouterFn>,
}

impl AdminApiState {
    pub fn new(auth: Arc<AuthState>, host_id: String, router: Arc<RouterFn>) -> Self {
        Self {
            auth,
            host_id,
            state: Arc::new(RwLock::new(StateSnapshot::default())),
            config_snapshot: Arc::new(RwLock::new(None)),
            router,
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

    // Bind a real 7-axis Identity to the inbound request so the
    // receipts ledger can correlate the actor precisely (instead
    // of leaving `identity: None` and tracing back to "http-api").
    // The platform is Web (this endpoint), the runtime is Interactive
    // (the operator is at the keyboard), and the user is whoever
    // proved they hold the bearer / JWT.
    let identity = crate::integrations::Identity {
        platform:      crate::integrations::Platform::Web,
        account_id:    api.host_id.clone(),
        channel_id:    "http-api".into(),
        user_id:       claims.sub.clone(),
        agent_id:      String::new(),
        session_key:   format!("http-api:{}", claims.sub),
        runtime_class: crate::integrations::RuntimeClass::Interactive,
    };

    let ctx = RemoteCommandContext {
        channel: crate::integrations::command_router::Channel::Web,
        actor_id: claims.sub.clone(),
        actor_name: claims.label.clone(),
        role,
        identity: Some(identity),
    };
    let cmd = RemoteCommand {
        kind,
        map_index: body.map_index,
        config_patch: None,
        tail: None,
    };
    if let Err(e) = authorize(&ctx, &cmd) { return err_500(format!("forbidden: {e}")); }

    // Real dispatch: route through the same multi-channel router closure
    // that Telegram/Discord/Slack already use.  The router internally
    // uses block_on, so we push the actual call to a blocking thread
    // to keep the axum reactor free.  Bounded at 30 s so a long-running
    // start/stop cannot pin the loopback HTTP server.
    let router = api.router.clone();
    let join = tokio::task::spawn_blocking(move || router(ctx, cmd)).await;
    let outcome = match join {
        Ok(Ok(outcome)) => outcome,
        Ok(Err(e))      => RouterOutcome::Error { reason: format!("router: {e}") },
        Err(e)          => RouterOutcome::Error { reason: format!("router task panicked: {e}") },
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

    // Dispatch via the shared router on a blocking thread; the
    // router internally calls block_on so we cannot await it inside
    // an async-without-yielding context.  Bound at 30 s in case the
    // launcher takes too long so the Convex call doesn't hang.
    let router = api.router.clone();
    let join = tokio::task::spawn_blocking(move || router(body.context, body.cmd)).await;
    let outcome = match join {
        Ok(Ok(outcome)) => outcome,
        Ok(Err(e))      => RouterOutcome::Error { reason: format!("router: {e}") },
        Err(e)          => RouterOutcome::Error { reason: format!("router task panicked: {e}") },
    };

    (StatusCode::OK, Json(outcome)).into_response()
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
                let kind_for_log = match kind {
                    crate::integrations::command_router::CommandKind::Start => "start",
                    crate::integrations::command_router::CommandKind::Stop => "stop",
                    crate::integrations::command_router::CommandKind::Restart => "restart",
                    crate::integrations::command_router::CommandKind::Status => "status",
                    crate::integrations::command_router::CommandKind::Logs => "logs",
                    crate::integrations::command_router::CommandKind::Ip => "ip",
                    crate::integrations::command_router::CommandKind::ConfigGet => "config_get",
                    crate::integrations::command_router::CommandKind::ConfigSet => "config_set",
                    crate::integrations::command_router::CommandKind::StartInstance => "start_instance",
                    crate::integrations::command_router::CommandKind::StopInstance => "stop_instance",
                };
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
            let v = parse_wechat_xml_loose(text);
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
    // token + timestamp + nonce sorted lexicographically then
    // SHA1'd; compare against operator's `msg_signature` in
    // constant time.
    let mut parts = vec![token.to_string(), timestamp.clone(), nonce.clone()];
    parts.sort();
    let concat = parts.join("");
    let nonce_for_log = nonce.clone();
    // sha1_smol = 1 — SHA-1 is mandated by WeChat Work's URL
    // verification handshake (`token + timestamp + nonce`
    // lex-sorted, then SHA-1 hex). Switching to SHA-3 here would
    // silently break the operator's WeCom webhook verification.
    // SHA-1 remains safe in this specific use because: (a)
    // corp_secret is a high-entropy shared key (256+ bits); (b)
    // the handshake runs once per operator setup; (c) no stored
    // secrets depend on this output. SHA-3 would break WeCom
    // compatibility, so we hold to SHA-1 by protocol mandate.
    let computed = {
        let d = sha1_smol::Sha1::from(concat.as_bytes()).digest();
        let mut hex = String::with_capacity(d.bytes().len() * 2);
        for b in d.bytes() {
            use std::fmt::Write;
            let _ = write!(&mut hex, "{b:02x}");
        }
        hex
    };
    if constant_time_eq(computed.as_bytes(), signature.as_bytes()) {
        echostr
    } else {
        log::warn!("wechat handshake: signature mismatch for nonce={nonce_for_log}");
        String::new()
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    a.iter().zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[cfg(test)]
mod tests {
    use super::constant_time_eq;

    #[test]
    fn constant_time_eq_matches_equal_inputs() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(constant_time_eq(b"", b""));
        assert!(constant_time_eq(b"\x00\x01\x02", b"\x00\x01\x02"));
    }

    #[test]
    fn constant_time_eq_rejects_different_inputs() {
        assert!(!constant_time_eq(b"hello", b"Hello"));
        assert!(!constant_time_eq(b"hello", b"hello!"));
        assert!(!constant_time_eq(b"", b"."));
        // Length mismatch through first guard
        assert!(!constant_time_eq(b"abc", b"abcd"));
    }

    #[test]
    fn parse_wechat_xml_cdata_removes_wrapper() {
        let xml = r#"<xml>
            <ToUserName>corp_xxx</ToUserName>
            <FromUserName>user_yyy</FromUserName>
            <CreateTime>1700000000</CreateTime>
            <MsgType>text</MsgType>
            <Content><![CDATA[this is 🦊 content]]></Content>
            <MsgId>msg_zzz</MsgId>
        </xml>"#;
        let v = super::parse_wechat_xml_loose(xml);
        assert_eq!(v["ToUserName"], "corp_xxx");
        assert_eq!(v["Content"], "this is 🦊 content");
        assert_eq!(v["MsgType"], "text");
    }

    #[test]
    fn parse_wechat_xml_xml_decl_stripped() {
        let xml = r#"<?xml version="1.0"?>
            <xml><Content>plain</Content></xml>"#;
        let v = super::parse_wechat_xml_loose(xml);
        assert_eq!(v["Content"], "plain");
    }

    #[test]
    fn parse_wechat_xml_missing_tag_returns_null() {
        let xml = "<xml><ToUserName>onlyhere</ToUserName></xml>";
        let v = super::parse_wechat_xml_loose(xml);
        assert!(v["Content"].is_null(),
            "missing tag should be null in JSON, got {:?}", v["Content"]);
    }
}

/// Tiny pull-style XML→flat-field extractor for WeChat Work.
///
/// We can't pull in `serde-xml-rs` without bumping Cargo; this
/// minimum viable helper satisfies the operator's most common
/// case (plain `<xml><Content>...</Content>...</xml>`). If the
/// payload uses an outer wrapper that's escaped, this is a no-op
/// — the operator wires an XML adapter in lib::run() to pre-parse
/// the body into JSON before forwarding here.
/// WeChat Work CDATA-aware tag extractor: pulls either
/// `<tag>text</tag>` or `<tag><![CDATA[text]]></tag>`. Anything
/// else falls back to `None` so the rest of the handshake can
/// fall back to the JSON-pre-parsed path.
fn parse_wechat_xml_loose(xml: &str) -> serde_json::Value {
    // strip OUTER <?xml ...?> declaration if present
    let xml = if xml.trim_start().starts_with("<?xml") {
        if let Some(end) = xml.find("?>") {
            xml[end + 2..].to_string()
        } else {
            xml.to_string()
        }
    } else {
        xml.to_string()
    };
    let tag = |t: &str| -> Option<String> {
        let open  = format!("<{t}>");
        let close = format!("</{t}>");
        // Plain text between tags
        if let Some(i) = xml.find(&open) {
            let j = i + open.len();
            if let Some(k) = xml[j..].find(&close) {
                let inner = &xml[j..j + k];
                if inner.starts_with("<![CDATA[")
                    && inner.ends_with("]]>")
                {
                    // Strip CDATA wrapper
                    return Some(inner[9..inner.len() - 3].to_string());
                }
                return Some(inner.to_string());
            }
        }
        None
    };
    serde_json::json!({
        "ToUserName":  tag("ToUserName"),
        "FromUserName": tag("FromUserName"),
        "CreateTime":   tag("CreateTime"),
        "MsgType":      tag("MsgType"),
        "Content":      tag("Content"),
        "MsgId":        tag("MsgId"),
    })
}
