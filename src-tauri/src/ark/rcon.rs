//! Source RCON protocol client for graceful ARK server management.
//!
//! Protocol summary:
//!   TCP connect → send auth packet (type=3) → read response (id=-1 means fail)
//!   → send command packet (type=2) → read response body.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

const CONNECT_TIMEOUT:  Duration = Duration::from_secs(5);
const AUTH_TIMEOUT:     Duration = Duration::from_secs(4);
const COMMAND_TIMEOUT:  Duration = Duration::from_secs(5);
const SAVE_GRACE_SECS:  u64      = 3;

/// RCON connection to one ARK instance.
pub struct RconClient {
    rcon_port: u16,
    password:  String,
}

impl RconClient {
    pub fn new(rcon_port: u16, password: impl Into<String>) -> Self {
        Self { rcon_port, password: password.into() }
    }

    /// Open a TCP connection, authenticate, send one command and return the body.
    pub async fn exec(&self, command: &str) -> Result<String, String> {
        let addr = format!("127.0.0.1:{}", self.rcon_port);
        let mut stream = timeout(CONNECT_TIMEOUT, TcpStream::connect(&addr))
            .await
            .map_err(|_| format!("RCON connect timeout (port {})", self.rcon_port))?
            .map_err(|e| format!("RCON connect failed: {}", e))?;

        // ── Authenticate ──────────────────────────────────────────────────────
        stream
            .write_all(&rcon_packet(1, 3, &self.password))
            .await
            .map_err(|e| e.to_string())?;

        let mut buf = [0u8; 4096];
        let _n = timeout(AUTH_TIMEOUT, stream.read(&mut buf))
            .await
            .map_err(|_| "RCON auth timeout")?
            .map_err(|e| e.to_string())?;

        // id == -1 in response means auth rejected
        if buf.len() >= 8 {
            let resp_id = i32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
            if resp_id == -1 {
                return Err("RCON auth failed — contraseña de admin incorrecta".to_string());
            }
        }

        // ── Send command ──────────────────────────────────────────────────────
        stream
            .write_all(&rcon_packet(2, 2, command))
            .await
            .map_err(|e| e.to_string())?;

        let n = timeout(COMMAND_TIMEOUT, stream.read(&mut buf))
            .await
            .map_err(|_| "RCON response timeout")?
            .unwrap_or(0);

        // Response body: bytes [12..n-2] (strip header + 2 trailing NULs)
        let body = if n > 12 {
            String::from_utf8_lossy(&buf[12..n.saturating_sub(2)])
                .trim()
                .to_string()
        } else {
            String::new()
        };
        Ok(body)
    }

    /// `saveworld` → wait → `doexit`. Returns Err on the first failure.
    pub async fn graceful_shutdown(&self) -> Result<(), String> {
        self.exec("saveworld").await?;
        tokio::time::sleep(Duration::from_secs(SAVE_GRACE_SECS)).await;
        self.exec("doexit").await?;
        Ok(())
    }
}

/// Build a Source RCON framed packet.
/// Layout: size(4 LE) | id(4 LE) | type(4 LE) | body | NUL | NUL
fn rcon_packet(id: i32, ptype: i32, body: &str) -> Vec<u8> {
    let body_bytes = body.as_bytes();
    let size = 4 + 4 + body_bytes.len() + 2; // id + type + body + 2×NUL
    let mut pkt = Vec::with_capacity(4 + size);
    pkt.extend_from_slice(&(size as i32).to_le_bytes());
    pkt.extend_from_slice(&id.to_le_bytes());
    pkt.extend_from_slice(&ptype.to_le_bytes());
    pkt.extend_from_slice(body_bytes);
    pkt.push(0); // body NUL
    pkt.push(0); // trailing NUL
    pkt
}
