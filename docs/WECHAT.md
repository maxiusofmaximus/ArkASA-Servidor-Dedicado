# WeChat Work (WeCom) adapter

The WeChat Work bot lets admins drive the server from WeCom. It has two parts:

1. **Inbound** — WeCom calls `POST /hooks/wechat` on the loopback API
   (`127.0.0.1:8765`). Expose it publicly only through a reverse proxy with TLS.
2. **Outbound** — `corpid`/`corpsecret` → `access_token` (cached ~2h) for replies.

## Configuration

Provide via the Plugin Hub secret store (plugin id `wechat`) or env vars:

| Field | Env | Notes |
|---|---|---|
| `corp_id` | `WECHAT_CORP_ID` | company id (`corpid`) |
| `corp_secret` | `WECHAT_CORP_SECRET` | app secret (`corpsecret`) |
| `agent_id` | `WECHAT_AGENT_ID` | app id |
| `admin_user_ids` | `WECHAT_ADMIN_USER_IDS` | comma-separated WeCom user ids allowed to run commands |
| `token` | `WECHAT_TOKEN` | Msg-Crypt token (required for encrypted callbacks) |
| `encoding_aes_key` | `WECHAT_ENCODING_AES_KEY` | Msg-Crypt 43-char `EncodingAESKey` |

## Msg-Crypt (encrypted callbacks)

When WeCom's "encrypted mode" is on, the callback body is
`<xml><Encrypt>…</Encrypt></xml>` and the URL carries
`?msg_signature=&timestamp=&nonce=`. The handler:

1. Recomputes `msg_signature = sha1(sort([token, timestamp, nonce, encrypt]))`
   and compares it in **constant time** (401 on mismatch).
2. AES-256-CBC-decrypts with `EncodingAESKey` (PKCS#7), strips the
   `16 random | 4-byte length | msg | receiveid` envelope, and parses the
   plaintext XML.

If an encrypted body arrives without `token`/`EncodingAESKey` configured, the
handler returns a clear 500 rather than silently trusting plaintext.

Plain (unencrypted) callbacks keep working when `token`/`EncodingAESKey` are
unset — useful for local development.

## URL verification handshake

`GET /hooks/wechat?msg_signature=…&timestamp=…&nonce=…&echostr=…` is answered
using the same SHA-1 (`wechat_handshake_sha1`). Configure the WeCom callback URL
to point at your reverse-proxied `/hooks/wechat`.

## Security notes

- Bind loopback only; terminate TLS at the proxy.
- The `AccessToken` grants send rights — keep `corp_secret` in the secret store
  (OS keyring), never in the config TOML.
- Admin commands are gated by `admin_user_ids`; an unknown sender is ignored.
