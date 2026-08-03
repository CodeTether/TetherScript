//! Byte-exact `text/event-stream` framing on the wire.
//!
//! The framing built-ins already have unit coverage in `tests/web_sse.rs`. What
//! is new here is that the bytes they produce reach the socket *unaltered*: the
//! streaming path must not re-frame, re-terminate, or normalise anything.

use std::time::Duration;

use super::response::stream_response;
use super::server::start;

#[test]
fn data_framing_is_byte_exact() {
    let server = start();
    let (_, body) = stream_response(server.port, "/events", Duration::from_secs(5));
    assert_eq!(
        body,
        b"data: tick 1\n\ndata: tick 2\n\ndata: tick 3\n\n".to_vec(),
        "body was {:?}",
        String::from_utf8_lossy(&body)
    );
}

#[test]
fn multi_line_data_gets_one_data_line_per_line() {
    let server = start();
    let (_, body) = stream_response(server.port, "/multiline", Duration::from_secs(5));
    // A raw newline inside a field value terminates the field, so a two-line
    // payload must be two `data:` lines that the client rejoins with `\n`.
    assert_eq!(body, b"data: first\ndata: second\n\n".to_vec());
}

#[test]
fn a_comment_and_a_retry_line_are_emitted_correctly() {
    let server = start();
    let (_, body) = stream_response(server.port, "/mixed", Duration::from_secs(5));
    let text = String::from_utf8_lossy(&body).to_string();
    // A comment is not an event: `: ` prefix, one newline, dispatches nothing.
    assert!(text.starts_with(": keep-alive\n"), "{text:?}");
    // A retry frame *is* an event, so it carries the blank-line terminator.
    assert!(text.contains("retry: 2500\n\n"), "{text:?}");
    assert!(text.ends_with("data: after\n\n"), "{text:?}");
    assert_eq!(text, ": keep-alive\nretry: 2500\n\ndata: after\n\n");
}
