//! Chunked transfer coding, read as raw bytes off the socket.
//!
//! Three ways to get chunked framing wrong, all silent from the server's side:
//! a decimal length instead of hex, a missing per-chunk trailing CRLF, or a
//! missing terminating zero-length chunk. Reading the bytes directly is the only
//! way to catch any of them, so that is what this file does.

use std::time::Duration;

use super::response::stream_response;
use super::server::start;

#[test]
fn chunked_coding_uses_hex_lengths_and_a_zero_terminator() {
    let server = start();
    let (head, body) = stream_response(server.port, "/chunked", Duration::from_secs(5));
    let lowered = head.to_ascii_lowercase();
    assert!(lowered.contains("transfer-encoding: chunked"), "{head}");
    assert!(
        !lowered.contains("content-length"),
        "chunked coding and Content-Length are mutually exclusive: {head}"
    );
    // `data: tick 1\n\n` is 14 bytes, hex `e`. A decimal bug would write `14`,
    // which the client reads as hex 0x14 and then waits for 20 bytes.
    let expected = b"e\r\ndata: tick 1\n\n\r\ne\r\ndata: tick 2\n\n\r\n0\r\n\r\n".to_vec();
    assert_eq!(
        body,
        expected,
        "body was {:?}",
        String::from_utf8_lossy(&body)
    );
}

#[test]
fn the_body_ends_with_the_terminating_zero_length_chunk() {
    let server = start();
    let (_, body) = stream_response(server.port, "/chunked", Duration::from_secs(5));
    assert!(
        body.ends_with(b"0\r\n\r\n"),
        "without the terminator a client reports a truncated body: {:?}",
        String::from_utf8_lossy(&body)
    );
}
