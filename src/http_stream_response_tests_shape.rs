//! Unit tests for streaming-response recognition.
//!
//! Recognition is the load-bearing rule — a false positive would change the
//! meaning of an existing handler — so it is asserted apart from field
//! validation and byte layout.

use crate::value::Value;

use super::values::{map, native, str_value};
use super::{is_stream, shape};

#[test]
fn a_plain_map_is_not_a_stream() {
    assert!(!is_stream(&map(&[("body", str_value("hi"))])));
}

#[test]
fn a_str_response_is_not_a_stream() {
    assert!(!is_stream(&str_value("hi")));
}

#[test]
fn a_nil_response_is_not_a_stream() {
    assert!(!is_stream(&Value::Nil));
}

#[test]
fn a_callable_under_stream_is_a_stream() {
    assert!(is_stream(&map(&[("stream", native())])));
}

#[test]
fn a_non_callable_stream_key_is_not_mistaken_for_a_stream() {
    // Recognition declines, so the ordinary path still applies...
    assert!(!is_stream(&map(&[("stream", Value::Int(1))])));
    // ...but asking to parse it names the key rather than degrading silently.
    let error = shape::parse(&map(&[("stream", Value::Int(1))])).unwrap_err();
    assert!(error.contains("response.stream"), "{error}");
}

#[test]
fn missing_stream_key_is_named() {
    let error = shape::parse(&map(&[("status", Value::Int(200))])).unwrap_err();
    assert!(error.contains("stream"), "{error}");
}

#[test]
fn a_non_map_streaming_response_is_refused() {
    assert!(shape::parse(&str_value("hi")).is_err());
}
