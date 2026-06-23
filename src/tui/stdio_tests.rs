//! Tests for stdio protocol helper values.

use crate::json;
use crate::value::Value;

use super::{jsonrpc, jsonrpc_error, val};

#[test]
fn jsonrpc_response_encodes_result() {
    let msg = jsonrpc::response(&[Value::Int(7), val::strv("ok")]).unwrap();
    let text = json::encode_to_string(&msg).unwrap();
    assert!(text.contains("\"jsonrpc\":\"2.0\""));
    assert!(text.contains("\"id\":7"));
    assert!(text.contains("\"result\":\"ok\""));
}

#[test]
fn jsonrpc_error_accepts_optional_data() {
    let msg = jsonrpc_error::error(&[
        Value::Int(2),
        Value::Int(-32601),
        val::strv("missing"),
        val::strv("tools/list"),
    ])
    .unwrap();
    let text = json::encode_to_string(&msg).unwrap();
    assert!(text.contains("\"error\""));
    assert!(text.contains("\"code\":-32601"));
    assert!(text.contains("\"data\":\"tools/list\""));
}
