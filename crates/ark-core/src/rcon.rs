//! Source RCON protocol: framing and parsing, with no I/O.
//!
//! This module deliberately stops at the byte level. Establishing the TCP
//! connection is a *transport* concern that differs per host (the desktop app
//! dials `127.0.0.1`, the phone dials a tunnel), so it lives with the caller,
//! while the wire format — which must never drift between platforms — lives
//! here and is covered by this crate's tests.
//!
//! Layout of every packet: `size(4 LE) | id(4 LE) | type(4 LE) | body | NUL | NUL`
//! where `size` counts the bytes after itself.

/// Server→client packet types we care about.
pub const TYPE_RESPONSE_VALUE: i32 = 0;
/// Client→server: authenticate.
pub const TYPE_AUTH: i32 = 3;
/// Client→server: run a command.
pub const TYPE_EXECCOMMAND: i32 = 2;

/// The id a server uses to reject authentication.
pub const AUTH_FAILURE_ID: i32 = -1;

/// Builds one framed RCON packet.
pub fn encode_packet(id: i32, packet_type: i32, body: &str) -> Vec<u8> {
    let body_bytes = body.as_bytes();
    let size = 4 + 4 + body_bytes.len() + 2; // id + type + body + 2×NUL
    let mut pkt = Vec::with_capacity(4 + size);
    pkt.extend_from_slice(&(size as i32).to_le_bytes());
    pkt.extend_from_slice(&id.to_le_bytes());
    pkt.extend_from_slice(&packet_type.to_le_bytes());
    pkt.extend_from_slice(body_bytes);
    pkt.push(0); // terminates the body
    pkt.push(0); // terminates the packet
    pkt
}

/// A decoded RCON packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub id: i32,
    pub packet_type: i32,
    pub body: String,
}

impl Packet {
    /// Whether this packet is a refused authentication.
    ///
    /// Source RCON signals failure by answering the auth request with id -1.
    pub fn is_auth_failure(&self) -> bool {
        self.id == AUTH_FAILURE_ID
    }
}

/// Decodes one packet from the front of `buf`.
///
/// Returns `Ok(None)` when `buf` does not yet hold a complete packet, which is
/// the normal case for a streaming TCP read and lets the caller keep reading
/// instead of erroring.
pub fn decode_packet(buf: &[u8]) -> Result<Option<(Packet, usize)>, RconError> {
    if buf.len() < 4 {
        return Ok(None);
    }
    let size = i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    if size < 10 {
        // 4(id) + 4(type) + at least 2 NULs
        return Err(RconError::Malformed(format!(
            "declared packet size {size} is below the 10-byte minimum"
        )));
    }
    let total = 4 + size as usize;
    if buf.len() < total {
        return Ok(None);
    }

    let id = i32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
    let packet_type = i32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
    // Body excludes the two trailing NULs.
    let body_bytes = &buf[12..total - 2];
    let body = String::from_utf8_lossy(body_bytes).into_owned();

    Ok(Some((
        Packet {
            id,
            packet_type,
            body,
        },
        total,
    )))
}

/// Why a packet could not be decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RconError {
    Malformed(String),
}

impl std::fmt::Display for RconError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RconError::Malformed(m) => write!(f, "malformed RCON packet: {m}"),
        }
    }
}

impl std::error::Error for RconError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_declares_the_correct_size() {
        let pkt = encode_packet(1, TYPE_AUTH, "secret");
        let declared = i32::from_le_bytes([pkt[0], pkt[1], pkt[2], pkt[3]]);
        // size counts everything after the size field itself.
        assert_eq!(declared as usize, pkt.len() - 4);
        // id(4) + type(4) + "secret"(6) + two NULs = 16
        assert_eq!(declared, 16);
    }

    #[test]
    fn encode_then_decode_roundtrips() {
        let pkt = encode_packet(42, TYPE_EXECCOMMAND, "saveworld");
        let (decoded, consumed) = decode_packet(&pkt).unwrap().expect("complete packet");
        assert_eq!(consumed, pkt.len());
        assert_eq!(decoded.id, 42);
        assert_eq!(decoded.packet_type, TYPE_EXECCOMMAND);
        assert_eq!(decoded.body, "saveworld");
    }

    #[test]
    fn an_empty_body_still_roundtrips() {
        let pkt = encode_packet(7, TYPE_RESPONSE_VALUE, "");
        let (decoded, _) = decode_packet(&pkt).unwrap().expect("complete packet");
        assert_eq!(decoded.body, "");
        assert_eq!(decoded.packet_type, TYPE_RESPONSE_VALUE);
    }

    #[test]
    fn a_partial_read_reports_incomplete_not_an_error() {
        let pkt = encode_packet(1, TYPE_AUTH, "password");
        for cut in [0, 1, 3, 4, 8, pkt.len() - 1] {
            assert_eq!(
                decode_packet(&pkt[..cut]).unwrap(),
                None,
                "a {cut}-byte prefix must read as incomplete"
            );
        }
        assert!(decode_packet(&pkt).unwrap().is_some());
    }

    #[test]
    fn auth_failure_is_recognisable() {
        let pkt = encode_packet(AUTH_FAILURE_ID, TYPE_RESPONSE_VALUE, "");
        let (decoded, _) = decode_packet(&pkt).unwrap().unwrap();
        assert!(decoded.is_auth_failure());
    }

    #[test]
    fn a_bogus_size_is_rejected_rather_than_trusted() {
        // Declares 2 bytes: below the 10-byte minimum.
        let bogus = [2i32.to_le_bytes(), [0u8; 4].into()].concat();
        assert!(matches!(
            decode_packet(&bogus),
            Err(RconError::Malformed(_))
        ));
    }
}
