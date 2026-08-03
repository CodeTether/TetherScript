//! Shared frame-building helpers for the codec tests.
//!
//! Frames are built byte-by-byte here rather than by calling the encoder, so the
//! tests assert against the *wire format* and cannot be fooled by an
//! encoder/decoder pair that is symmetrically wrong.

use tetherscript::websocket::frame::{DecodeOutcome, Frame};
use tetherscript::websocket::role::Role;

/// A fixed masking key; any value works since masking is not secrecy.
pub const KEY: [u8; 4] = [0x37, 0xfa, 0x21, 0x3d];

/// Build a masked client frame with the minimal length form.
///
/// # Arguments
///
/// * `fin` — Sets the FIN bit.
/// * `opcode` — Low nibble of byte 0.
/// * `payload` — Unmasked payload; it is masked with [`KEY`] on the way out.
pub fn client_frame(fin: bool, opcode: u8, payload: &[u8]) -> Vec<u8> {
    let fin_bit: u8 = if fin { 0x80 } else { 0x00 };
    let mut out = vec![fin_bit | opcode];
    let len = payload.len();
    if len <= 125 {
        out.push(0x80 | len as u8);
    } else if len <= u16::MAX as usize {
        out.push(0x80 | 126);
        out.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        out.push(0x80 | 127);
        out.extend_from_slice(&(len as u64).to_be_bytes());
    }
    out.extend_from_slice(&KEY);
    let mut body = payload.to_vec();
    tetherscript::websocket::mask::apply(&mut body, KEY);
    out.extend_from_slice(&body);
    out
}

/// Decode `bytes` as a client frame, asserting it is complete.
///
/// # Panics
///
/// Panics if the bytes are malformed or incomplete; that is the test failure.
pub fn decode_client(bytes: &[u8]) -> (Frame, usize) {
    match Frame::decode(bytes, Role::Client).expect("frame should be valid") {
        DecodeOutcome::Frame { frame, consumed } => (frame, consumed),
        DecodeOutcome::Incomplete => panic!("frame should be complete"),
    }
}
