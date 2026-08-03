//! Unit tests for header and status field defaults.

use crate::value::Value;

use super::values::{map, native, str_value};
use super::{shape, Coding};

#[test]
fn sse_headers_default_but_never_override() {
    let spec = shape::parse(&map(&[("stream", native())])).unwrap();
    let header = |name: &str| spec.headers.get(name).cloned().unwrap_or_default();
    assert_eq!(header("content-type"), "text/event-stream; charset=utf-8");
    assert_eq!(header("cache-control"), "no-cache");
    assert_eq!(spec.status, 200);
    assert_eq!(spec.coding, Coding::Close);
}

#[test]
fn an_explicit_content_type_wins() {
    let given = map(&[("Content-Type", str_value("text/plain"))]);
    let spec = shape::parse(&map(&[("stream", native()), ("headers", given)])).unwrap();
    assert_eq!(
        spec.headers.get("content-type").map(String::as_str),
        Some("text/plain")
    );
}

#[test]
fn an_out_of_range_status_is_refused() {
    let response = map(&[("stream", native()), ("status", Value::Int(42))]);
    let error = shape::parse(&response).unwrap_err();
    assert!(error.contains("100..=599"), "{error}");
}

#[test]
fn a_non_int_status_is_refused_by_name() {
    let response = map(&[("stream", native()), ("status", str_value("200"))]);
    let error = shape::parse(&response).unwrap_err();
    assert!(error.contains("response.status"), "{error}");
}

#[test]
fn a_non_bool_chunked_is_refused_by_name() {
    let response = map(&[("stream", native()), ("chunked", Value::Int(1))]);
    let error = shape::parse(&response).unwrap_err();
    assert!(error.contains("response.chunked"), "{error}");
}
