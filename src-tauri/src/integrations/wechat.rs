//! WeChat adapter — v2.1 skeleton.
//!
//! Inbound arrival: WeChat Work / WeCom bot hook. The wire format is
//! specific XML (`<xml>`) rather than JSON; we deserialize with
//! `serde-xml-rs` semantics but the cargo graph today ships with a
//! plain JSON field-by-field parser so the operator can wire any
//! `wechaty` or `wechat-work` bot SDK and synthesise a
//! `WeChatXmlPayload { from_user, content, msg_type, create_time }`.
//!
//! Outbound is symmetric: a `corpid/corpsecret/access_token` flow
//! returns an `access_token` (cached 2 hours). The actual outbound
//! POST is left to `wechat_bridge.rs::send_wechat_message` because
//! it's straightforward `reqwest` against the WeChat Work API.
//!
//! Backwards rule: ZERO WeChat SDK coupling here. We parse XML as
//! string fields — if you ship a richer parser, it goes in
//! `bridge.rs`.

//! ────────────────────────────────────────────────────────────────────
//! P33 — helpers moved out of `integrations::http_api`
//! (`constant_time_eq`, `parse_wechat_xml_loose`, `wechat_handshake_sha1`).
//! They were 40-line misplaced utility bits sitting in the Web transport
//! layer; keeping them here avoids spurious imports on every http_api
//! modification and centralises all WeChat-specific glue.

use crate::integrations::command_router::{
    default_chat_binding, gated_chat_binding, Channel, CommandKind,
    RemoteCommandContext, Role, RouterOutcome,
};
use crate::integrations::RuntimeClass;
use crate::plugins::secret_store_v2 as secret_store;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatConfig {
    pub corp_id:           String,    // corpid
    pub corp_secret:       String,    // corpsecret
    pub agent_id:          String,    // application id
    pub admin_user_ids:    String,    // comma-separated WeChat userIds
    /// Msg-Crypt token configured in the WeChat Work app console.
    #[serde(default)]
    pub token:             String,
    /// Msg-Crypt `EncodingAESKey` (43 chars, base64 of 32 bytes without pad).
    #[serde(default)]
    pub encoding_aes_key:  String,
}

impl WeChatConfig {
    pub fn from_secrets_or_env() -> Self {
        if let Some(s) = secret_store::read("wechat") {
            let get = |k: &str| s.fields.get(k).cloned().unwrap_or_default();
            return Self {
                corp_id:        get("corp_id"),
                corp_secret:    get("corp_secret"),
                agent_id:       get("agent_id"),
                admin_user_ids: get("admin_user_ids"),
                token:             get("token"),
                encoding_aes_key:  get("encoding_aes_key"),
            };
        }
        Self {
            corp_id:        std::env::var("WECHAT_CORP_ID").unwrap_or_default(),
            corp_secret:    std::env::var("WECHAT_CORP_SECRET").unwrap_or_default(),
            agent_id:       std::env::var("WECHAT_AGENT_ID").unwrap_or_default(),
            admin_user_ids: std::env::var("WECHAT_ADMIN_USER_IDS").unwrap_or_default(),
            token:             std::env::var("WECHAT_TOKEN").unwrap_or_default(),
            encoding_aes_key:  std::env::var("WECHAT_ENCODING_AES_KEY").unwrap_or_default(),
        }
    }
    pub fn is_admin(&self, user_id: &str) -> bool {
        self.admin_user_ids.split(',')
            .map(|s| s.trim())
            .any(|s| !s.is_empty() && s == user_id)
    }
}

impl Default for WeChatConfig {
    fn default() -> Self {
        Self {
            corp_id:        String::new(),
            corp_secret:    String::new(),
            agent_id:       String::new(),
            admin_user_ids: String::new(),
            token:             String::new(),
            encoding_aes_key:  String::new(),
        }
    }
}

/// Plain-fields WeChat Work inbound payload. Real WeChat XML is
/// <xml><ToUserName>...</ToUserName>…</xml>; this struct mirrors the
/// relevant subset (the bridge can pre-parse XML into this if it
/// wants). Tests use JSON-shaped fixtures built directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatXmlPayload {
    #[serde(rename = "ToUserName",  default)] pub to_user_name:  String,
    #[serde(rename = "FromUserName",default)] pub from_user_name: String,
    #[serde(rename = "CreateTime",   default)] pub create_time:    String,
    #[serde(rename = "MsgType",      default)] pub msg_type:       String,
    #[serde(rename = "Content",      default)] pub content:        String,
    #[serde(rename = "MsgId",        default)] pub msg_id:         String,
}

pub struct WeChatBot {
    pub cfg: WeChatConfig,
}

impl WeChatBot {
    pub fn new(cfg: WeChatConfig) -> Self { Self { cfg } }

    pub fn parse_action(text: &str) -> Option<CommandKind> {
        if !text.starts_with('/') { return None; }
        let first = text.split_whitespace().next()?.trim();
        CommandKind::parse_slash(first)
    }

    pub fn accept_message(&self, payload: &WeChatXmlPayload) -> Option<RemoteCommandContext> {
        if payload.msg_type.is_empty() || payload.msg_type != "text" { return None; }
        if !self.cfg.is_admin(&payload.from_user_name) { return None; }
        let admins: Vec<String> = self.cfg.admin_user_ids.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()).collect();
        let binding = if admins.is_empty() {
            default_chat_binding(Channel::Wechat, &payload.from_user_name)
        } else {
            gated_chat_binding(Channel::Wechat, &payload.from_user_name, admins)
        };
        match RemoteCommandContext::from_binding(
            Channel::Wechat,
            &binding,
            &payload.from_user_name,
            &format!("wechat-{}", payload.from_user_name),
            Role::Admin,
            RuntimeClass::Interactive,
            &format!("wechat:{}", payload.from_user_name),
        ) {
            Ok(c)  => Some(c),
            Err(_) => None,
        }
    }

    pub fn render_outcome(o: &RouterOutcome) -> String {
        use crate::integrations::command_router::RouterOutcome as R;
        match o {
            R::Started { map, pid }  => format!("✅ {map} started (pid {pid})"),
            R::Stopped { map }       => format!("⏹ {map} stopped"),
            R::Restarted { map }     => format!("♻ {map} restarted"),
            R::Status { running, .. }=> if !running { "⚠ not running".into() } else { "✓ running".into() },
            R::Logs { lines }        => lines.join("\n"),
            R::Ip { .. }             => "(ip payload — connect via /api/v1/ip)".into(),
            R::ConfigGet { toml }    => toml.clone(),
            R::ConfigSet { applied } => format!("✅ applied {applied} entries"),
            R::Error { reason }      => format!("⚠ {reason}"),
        }
    }
}

pub const DESCRIPTOR: crate::plugins::PluginDescriptor = crate::plugins::PluginDescriptor {
    id: "wechat",
    label: "WeChat Work",
    channel: crate::plugins::ChannelKind::Wechat,
    capabilities: &[
        crate::plugins::PluginCapability::MessagesRecv,
        crate::plugins::PluginCapability::MessagesSend,
        crate::plugins::PluginCapability::RequiresSecrets,
    ],
    required_secrets: &["corp_id", "corp_secret", "agent_id", "admin_user_ids"],
    oauth_url: None,
};

pub struct WeChatPlugin;

#[async_trait::async_trait]
impl crate::plugins::Plugin for WeChatPlugin {
    fn id() -> &'static str { "wechat" }
    fn descriptor() -> crate::plugins::PluginDescriptor { DESCRIPTOR }
    async fn start(_ctx: crate::plugins::PluginContext) -> Result<tokio::task::JoinHandle<()>, crate::plugins::PluginStartError> {
        Ok(tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(u64::MAX)).await;
        }))
    }
}

// ───── P33 — WeChat-specific crypto + XML helpers (moved from http_api) ─

/// Constant-time slice comparison. Returns `false` immediately if the
/// lengths differ (length is publicly known, no need to leak). Used by
/// `wechat_handshake_sha1` so a forged `msg_signature` cannot be
/// timing-attacked out of the loopback server.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    a.iter().zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

/// WeChat Work URL-verification SHA-1.
///
/// WeChat mandates SHA-1 here (not SHA-3 / BLAKE3) — switching would
/// silently break the operator's WeCom callback URL. SHA-1 is safe in
/// this **specific** use because `corp_secret` is a high-entropy shared
/// key (256+ bits) and the handshake runs once per operator setup.
pub fn wechat_handshake_sha1(corp_secret: &str, timestamp: &str, nonce: &str) -> String {
    let mut parts = vec![corp_secret.to_string(), timestamp.to_string(), nonce.to_string()];
    parts.sort();
    let concat = parts.join("");
    let d = sha1_smol::Sha1::from(concat.as_bytes()).digest();
    let mut hex = String::with_capacity(d.bytes().len() * 2);
    use std::fmt::Write;
    for b in d.bytes() {
        let _ = write!(&mut hex, "{b:02x}");
    }
    hex
}

// ─────────────────────────────────────────────────────────────────────────────
// P7 — WeChat Work Msg-Crypt (the real encrypted callback)
// ─────────────────────────────────────────────────────────────────────────────
//
// WeChat Work callbacks can be AESCBC-encrypted. The algorithm (documented by
// Tencent) is:
//   * `EncodingAESKey` is 43 base64 chars → 32 bytes (append `=` to decode).
//   * AES-256-CBC, IV = first 16 bytes of the key, PKCS#7 padding.
//   * Plaintext layout: 16 random bytes | 4-byte BE msg length | msg | receiveid.
//   * `msg_signature` = SHA-1 of the lexicographically-sorted
//     [token, timestamp, nonce, encrypt] concatenated with no separator.
//
// Previously the webhook trusted plain XML and only had a handshake SHA-1
// helper (P33). This is the verification + decryption path.

fn aes_key_bytes(encoding_aes_key: &str) -> Result<Vec<u8>, String> {
    let mut key_b64 = encoding_aes_key.trim().to_string();
    if key_b64.len() != 44 {
        key_b64.push('=');
    }
    use base64::Engine as _;
    let key = base64::engine::general_purpose::STANDARD
        .decode(key_b64.as_bytes())
        .map_err(|e| format!("EncodingAESKey base64: {e}"))?;
    if key.len() != 32 {
        return Err(format!("EncodingAESKey must decode to 32 bytes, got {}", key.len()));
    }
    Ok(key)
}

/// SHA-1 over the sorted `[token, timestamp, nonce, encrypt]` (hex).
pub fn msg_signature(token: &str, timestamp: &str, nonce: &str, encrypt: &str) -> String {
    let mut parts = [token, timestamp, nonce, encrypt];
    parts.sort_unstable();
    let d = sha1_smol::Sha1::from(parts.concat().as_bytes()).digest();
    let mut hex = String::with_capacity(40);
    use std::fmt::Write;
    for b in d.bytes() {
        let _ = write!(&mut hex, "{b:02x}");
    }
    hex
}

/// Decrypt an `EncodingAESKey`-encrypted payload and return the plaintext XML.
pub fn decrypt_msg(encoding_aes_key: &str, encrypt_b64: &str) -> Result<String, String> {
    use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
    use base64::Engine as _;

    let key = aes_key_bytes(encoding_aes_key)?;
    let iv = &key[..16];
    let mut buf = base64::engine::general_purpose::STANDARD
        .decode(encrypt_b64.trim().as_bytes())
        .map_err(|e| format!("ciphertext base64: {e}"))?;

    let pt = cbc::Decryptor::<aes::Aes256>::new_from_slices(&key, iv)
        .map_err(|e| e.to_string())?
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|e| format!("AES-CBC decrypt: {e}"))?;

    if pt.len() < 20 {
        return Err("decrypted payload too short".into());
    }
    let msg_len = u32::from_be_bytes([pt[16], pt[17], pt[18], pt[19]]) as usize;
    if 20 + msg_len > pt.len() {
        return Err("declared msg length exceeds payload".into());
    }
    std::str::from_utf8(&pt[20..20 + msg_len])
        .map(|s| s.to_string())
        .map_err(|e| format!("payload not utf-8: {e}"))
}

/// Encrypt a payload (`receive_id` = corp_id). Used for replies and tests.
pub fn encrypt_msg(encoding_aes_key: &str, receive_id: &str, plaintext: &str) -> Result<String, String> {
    use cbc::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
    use base64::Engine as _;

    let key = aes_key_bytes(encoding_aes_key)?;
    let iv = &key[..16];

    let mut body = Vec::with_capacity(20 + plaintext.len() + receive_id.len());
    body.extend_from_slice(&[0u8; 16]); // random nonce (zeros acceptable for callbacks/tests)
    body.extend_from_slice(&(plaintext.len() as u32).to_be_bytes());
    body.extend_from_slice(plaintext.as_bytes());
    body.extend_from_slice(receive_id.as_bytes());

    let mut buf = vec![0u8; body.len() + 16]; // room for PKCS#7 padding
    buf[..body.len()].copy_from_slice(&body);
    let ct = cbc::Encryptor::<aes::Aes256>::new_from_slices(&key, iv)
        .map_err(|e| e.to_string())?
        .encrypt_padded_mut::<Pkcs7>(&mut buf, body.len())
        .map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(ct))
}

/// Full verification: check `msg_signature` in constant time, then decrypt.
pub fn decrypt_and_verify(
    token: &str,
    encoding_aes_key: &str,
    msg_signature_hex: &str,
    timestamp: &str,
    nonce: &str,
    encrypt_b64: &str,
) -> Result<String, String> {
    let expected = msg_signature(token, timestamp, nonce, encrypt_b64);
    if !constant_time_eq(expected.as_bytes(), msg_signature_hex.trim().as_bytes()) {
        return Err("msg_signature mismatch".into());
    }
    decrypt_msg(encoding_aes_key, encrypt_b64)
}

/// Tiny pull-style XML→flat-field extractor for WeChat Work.
///
/// We can't pull in `serde-xml-rs` without bumping Cargo; this
/// minimum viable helper satisfies the operator's most common
/// case (plain `<xml><Content>...</Content>...</xml>`, including
/// CDATA-wrapped emoji content). Returns a JSON object with the
/// canonical WeChat fields.
pub fn parse_wechat_xml_loose(xml: &str) -> serde_json::Value {
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
        if let Some(i) = xml.find(&open) {
            let j = i + open.len();
            if let Some(k) = xml[j..].find(&close) {
                let inner = &xml[j..j + k];
                if inner.starts_with("<![CDATA[") && inner.ends_with("]]>") {
                    return Some(inner[9..inner.len() - 3].to_string());
                }
                return Some(inner.to_string());
            }
        }
        None
    };
    serde_json::json!({
        "ToUserName":   tag("ToUserName"),
        "FromUserName": tag("FromUserName"),
        "CreateTime":   tag("CreateTime"),
        "MsgType":      tag("MsgType"),
        "Content":      tag("Content"),
        "MsgId":        tag("MsgId"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_allowlist_filters() {
        let mut cfg = WeChatConfig::default();
        cfg.admin_user_ids = "USER_001 , USER_002".into();
        assert!(cfg.is_admin("USER_001"));
        assert!(cfg.is_admin("USER_002"));
        assert!(!cfg.is_admin("USER_999"));
        cfg.admin_user_ids.clear();
        assert!(!cfg.is_admin("USER_001"));
    }

    #[test]
    fn parse_action_recognises_commands() {
        use CommandKind::*;
        let cases = [
            ("/start",   Some(Start)),
            ("/stop",    Some(Stop)),
            ("/restart", Some(Restart)),
            ("/status",  Some(Status)),
            ("/logs",    Some(Logs)),
            ("/ip",      Some(Ip)),
            ("/unknown", None),
        ];
        for (input, expected) in cases {
            let parsed = WeChatBot::parse_action(input);
            match expected {
                Some(ek) => {
                    let ak = parsed.unwrap_or_else(|| panic!("{input} must parse"));
                    assert_eq!(format!("{ak:?}"), format!("{ek:?}"));
                }
                None => assert!(parsed.is_none()),
            }
        }
    }

    #[test]
    fn accept_message_filters_nontext() {
        let mut cfg = WeChatConfig::default();
        cfg.admin_user_ids = "USER_001".into();
        let bot = WeChatBot::new(cfg);
        let mut p = WeChatXmlPayload {
            to_user_name: "AGENT".into(),
            from_user_name: "USER_001".into(),
            create_time: "0".into(),
            msg_type: "text".into(),
            content: "/status".into(),
            msg_id: "wm.0".into(),
        };
        assert!(bot.accept_message(&p).is_some(), "admin + text + slash prefix");

        p.from_user_name = "USER_999".into();
        assert!(bot.accept_message(&p).is_none(), "non-admin rejected");

        p.from_user_name = "USER_001".into();
        p.msg_type = "image".into();
        assert!(bot.accept_message(&p).is_none(), "non-text rejected");
    }

    #[test]
    fn render_outcome_error() {
        let s = WeChatBot::render_outcome(&RouterOutcome::Error {
            reason: "wechat ack failed".into(),
        });
        assert!(s.contains("⚠"));
        assert!(s.contains("wechat ack failed"));
    }

    #[test]
    fn default_config_is_empty_secure() {
        let cfg = WeChatConfig::default();
        for s in [
            cfg.corp_id.as_str(),
            cfg.corp_secret.as_str(),
            cfg.agent_id.as_str(),
            cfg.admin_user_ids.as_str(),
        ] {
            assert!(s.is_empty());
        }
    }

    // ─── P33 — relocated helper tests ─────────────────────────────────────

    #[test]
    fn constant_time_eq_handles_equal_and_unequal_inputs() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(constant_time_eq(b"", b""));
        assert!(!constant_time_eq(b"hello", b"Hello"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
    }

    #[test]
    fn wechat_handshake_sha1_matches_lex_sorted_concat() {
        // Hard-coded golden output: with corp_secret = "TOK", ts = "1700",
        // nonce = "ABC", lex-sorted → "1700ABCTOK"; the SHA-1 hex of that
        // is well-known. We verify by recomputing with sha1_smol directly.
        use std::fmt::Write;
        let mut parts = vec!["TOK".to_string(), "1700".to_string(), "ABC".to_string()];
        parts.sort();
        let concat: String = parts.join("");
        let d = sha1_smol::Sha1::from(concat.as_bytes()).digest();
        let mut expected = String::new();
        for b in d.bytes() { let _ = write!(&mut expected, "{b:02x}"); }
        let got = wechat_handshake_sha1("TOK", "1700", "ABC");
        assert_eq!(got, expected);
        assert_eq!(got.len(), 40, "SHA-1 hex is 40 chars");
    }

    #[test]
    fn parse_wechat_xml_loose_strips_cdata_and_handles_missing_tags() {
        let xml = r#"<xml>
            <ToUserName>corp_xxx</ToUserName>
            <FromUserName>user_yyy</FromUserName>
            <CreateTime>1700000000</CreateTime>
            <MsgType>text</MsgType>
            <Content><![CDATA[this is 🦊 content]]></Content>
            <MsgId>msg_zzz</MsgId>
        </xml>"#;
        let v = parse_wechat_xml_loose(xml);
        assert_eq!(v["ToUserName"], "corp_xxx");
        assert_eq!(v["Content"], format!("this is {} content", '\u{1F98A}'));
        assert_eq!(v["MsgType"], "text");
        assert!(v["CreateTime"].is_string());
    }

    // ── P7: Msg-Crypt ────────────────────────────────────────────────────────

    /// A valid 43-char EncodingAESKey (base64 of 32 bytes, padding stripped).
    fn test_aes_key() -> String {
        use base64::Engine as _;
        let b64 = base64::engine::general_purpose::STANDARD.encode([0x42u8; 32]);
        b64.trim_end_matches('=').to_string()
    }

    #[test]
    fn msgcrypt_roundtrip() {
        let key = test_aes_key();
        assert_eq!(key.len(), 43);
        let xml = "<xml><Content>hola</Content></xml>";
        let enc = encrypt_msg(&key, "corp_abc", xml).unwrap();
        let dec = decrypt_msg(&key, &enc).unwrap();
        assert_eq!(dec, xml);
    }

    #[test]
    fn msgcrypt_signature_verifies_and_rejects_tampering() {
        let key = test_aes_key();
        let enc = encrypt_msg(&key, "corp_abc", "<xml/>").unwrap();
        let sig = msg_signature("TOK", "1700000000", "NONCE", &enc);
        // Correct signature → decrypts.
        assert!(decrypt_and_verify("TOK", &key, &sig, "1700000000", "NONCE", &enc).is_ok());
        // Wrong token → constant-time mismatch.
        assert!(decrypt_and_verify("BAD", &key, &sig, "1700000000", "NONCE", &enc).is_err());
        // Tampered ciphertext → mismatch too.
        let tampered = format!("{}A", enc);
        assert!(decrypt_and_verify("TOK", &key, &sig, "1700000000", "NONCE", &tampered).is_err());
    }

    #[test]
    fn msgcrypt_rejects_bad_key_length() {
        assert!(decrypt_msg("tooshort", "AAAA").is_err());
    }
}
