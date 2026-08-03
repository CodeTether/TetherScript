//! Unit tests for response-head bytes.
//!
//! The head is asserted as a string rather than over a socket so the absence of
//! `Content-Length` is visible at a glance; `tests/http_sse_stream.rs` proves the
//! same absence on the wire.

use crate::value::Value;

use super::values::{map, native, str_value};
use super::{render_head, shape, Coding, StreamSpec};

/// Parse a streaming response from `entries`, panicking on invalid input.
fn spec_of(entries: &[(&str, Value)]) -> StreamSpec {
    shape::parse(&map(entries)).expect("spec should parse")
}

#[test]
fn the_head_never_carries_a_content_length() {
    let head = render_head(&spec_of(&[("stream", native())]), "OK");
    assert!(
        !head.to_ascii_lowercase().contains("content-length"),
        "head must not promise a length it cannot know: {head}"
    );
    assert!(head.starts_with("HTTP/1.1 200 OK\r\n"), "{head}");
    assert!(head.contains("Connection: close\r\n"), "{head}");
    assert!(head.ends_with("\r\n\r\n"), "{head}");
}

#[test]
fn a_handler_supplied_content_length_is_dropped() {
    let given = map(&[("Content-Length", str_value("12"))]);
    let head = render_head(&spec_of(&[("stream", native()), ("headers", given)]), "OK");
    let lowered = head.to_ascii_lowercase();
    assert!(!lowered.contains("content-length"), "{head}");
}

#[test]
fn chunked_advertises_transfer_encoding_instead() {
    let spec = spec_of(&[("stream", native()), ("chunked", Value::Bool(true))]);
    assert_eq!(spec.coding, Coding::Chunked);
    let head = render_head(&spec, "OK");
    assert!(head.contains("Transfer-Encoding: chunked\r\n"), "{head}");
    let lowered = head.to_ascii_lowercase();
    assert!(!lowered.contains("content-length"), "{head}");
}

#[test]
fn the_status_reason_comes_from_the_caller() {
    let spec = spec_of(&[("stream", native()), ("status", Value::Int(503))]);
    let head = render_head(&spec, "Service Unavailable");
    let expected = "HTTP/1.1 503 Service Unavailable\r\n";
    assert!(head.starts_with(expected), "{head}");
}
