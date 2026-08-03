//! Server-side frame encoding (RFC 6455 §5.1).
//!
//! A server **must not** mask, so there is deliberately no way to ask this
//! encoder for a masking key: the MASK bit is hard-wired clear and no key is ever
//! emitted. Making that unreachable rather than a default is the point — an
//! accidentally masked server frame is a protocol violation every conforming
//! client will close the connection over.
//!
//! The length field always uses the **minimal** form, matching the strictness the
//! decoder applies to peers.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::encode::encode_server;
//! use tetherscript::websocket::opcode::Opcode;
//!
//! // RFC 6455 §5.7: an unmasked server "Hello".
//! let wire = encode_server(true, Opcode::Text, b"Hello");
//! assert_eq!(wire, vec![0x81, 0x05, b'H', b'e', b'l', b'l', b'o']);
//! // Bit 7 of byte 1 is MASK, and it is clear.
//! assert_eq!(wire[1] & 0x80, 0);
//!
//! // 126 bytes crosses into the 16-bit length form.
//! let wire = encode_server(true, Opcode::Binary, &[0u8; 126]);
//! assert_eq!(&wire[..4], &[0x82, 0x7e, 0x00, 0x7e]);
//!
//! // 65536 bytes crosses into the 64-bit form; the MSB is zero.
//! let wire = encode_server(true, Opcode::Binary, &[0u8; 65_536]);
//! assert_eq!(&wire[..2], &[0x82, 0x7f]);
//! assert_eq!(&wire[2..10], &[0, 0, 0, 0, 0, 1, 0, 0]);
//! ```

use crate::websocket::opcode::Opcode;

/// Encode one unmasked frame for transmission by a server.
///
/// # Arguments
///
/// * `fin` — Whether this frame completes its message.
/// * `opcode` — The frame's opcode.
/// * `payload` — Payload bytes, sent verbatim and unmasked.
///
/// # Returns
///
/// The full wire representation: byte 0, the minimal length field, then the
/// payload. RSV bits are always zero and MASK is always clear.
///
/// # Panics
///
/// Never. No indexing is performed; the buffer is only appended to.
pub fn encode_server(fin: bool, opcode: Opcode, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 10);
    let fin_bit: u8 = if fin { 0x80 } else { 0x00 };
    out.push(fin_bit | opcode.to_bits());
    let len = payload.len();
    if len <= 125 {
        out.push(len as u8);
    } else if len <= u16::MAX as usize {
        out.push(126);
        out.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        out.push(127);
        out.extend_from_slice(&(len as u64).to_be_bytes());
    }
    out.extend_from_slice(payload);
    out
}
