//! Unit tests for chunked framing bytes.
//!
//! The hex-length rule is asserted at two payload sizes because a decimal-length
//! bug is invisible below ten bytes and still looks plausible at fifteen.

use super::{frame_bytes, TERMINATOR};

#[test]
fn chunk_lengths_are_hexadecimal() {
    // 10 bytes must be announced as `a`, never as `10`.
    let framed = frame_bytes(b"data: hi\n\n").expect("non-empty payload frames");
    assert_eq!(framed, b"a\r\ndata: hi\n\n\r\n".to_vec());
    // 15 bytes is where a decimal bug still looks plausible: `f` versus `15`.
    let framed = frame_bytes(b"data: 0123456\n\n").expect("frames");
    assert_eq!(framed, b"f\r\ndata: 0123456\n\n\r\n".to_vec());
}

#[test]
fn a_chunk_carries_its_own_trailing_crlf() {
    assert_eq!(frame_bytes(b"x").expect("frames"), b"1\r\nx\r\n".to_vec());
}

#[test]
fn an_empty_payload_never_becomes_a_zero_chunk() {
    assert!(frame_bytes(b"").is_none());
    assert_eq!(TERMINATOR, b"0\r\n\r\n");
}
