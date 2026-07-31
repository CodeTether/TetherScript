//! Byte-level coverage for the extended-protocol messages.
//!
//! Framing errors here would surface as opaque server complaints or a hung read,
//! so the exact bytes are asserted against the documented layouts.

use super::extended::{bind, describe, execute, parse, sync};
use super::params::encode_all;
use crate::value::Value;

#[test]
fn parse_declares_an_unnamed_statement_with_inferred_types() {
    let message = parse("SELECT 1");
    assert_eq!(message[0], b'P');
    // len(4) + name(1) + sql(9) + param count(2) = 16
    assert_eq!(
        i32::from_be_bytes([message[1], message[2], message[3], message[4]]),
        16
    );
    assert_eq!(message[5], 0, "statement name must be empty");
    assert_eq!(&message[6..14], b"SELECT 1");
    assert_eq!(message[14], 0, "sql must be NUL terminated");
    // Zero parameter types asks the server to infer them.
    assert_eq!(i16::from_be_bytes([message[15], message[16]]), 0);
}

#[test]
fn bind_encodes_a_null_as_a_negative_length() {
    let encoded = encode_all(&[Value::Nil]).expect("nil should encode");
    let message = bind(&encoded);
    assert_eq!(message[0], b'B');
    // Trailing bytes: param count, the -1 length, then the result format count.
    let tail = &message[message.len() - 8..];
    assert_eq!(i16::from_be_bytes([tail[0], tail[1]]), 1, "one parameter");
    assert_eq!(
        i32::from_be_bytes([tail[2], tail[3], tail[4], tail[5]]),
        -1,
        "NULL is length -1 with no bytes"
    );
    assert_eq!(i16::from_be_bytes([tail[6], tail[7]]), 0, "text results");
}

#[test]
fn bind_encodes_a_value_with_its_byte_length() {
    let encoded = encode_all(&[Value::Int(42)]).expect("int should encode");
    let message = bind(&encoded);
    let tail = &message[message.len() - 10..];
    assert_eq!(i16::from_be_bytes([tail[0], tail[1]]), 1);
    assert_eq!(i32::from_be_bytes([tail[2], tail[3], tail[4], tail[5]]), 2);
    assert_eq!(&tail[6..8], b"42", "ints are sent in text format");
}

#[test]
fn describe_and_execute_target_the_unnamed_portal() {
    assert_eq!(describe()[0], b'D');
    assert_eq!(describe()[5], b'P', "describe a portal, not a statement");
    assert_eq!(execute()[0], b'E');
    assert_eq!(sync(), vec![b'S', 0, 0, 0, 4], "Sync carries no body");
}

#[test]
fn booleans_bind_as_the_text_forms_postgres_accepts() {
    let encoded = encode_all(&[Value::Bool(true), Value::Bool(false)]).expect("bools encode");
    assert_eq!(encoded[0].as_deref(), Some(&b"t"[..]));
    assert_eq!(encoded[1].as_deref(), Some(&b"f"[..]));
}

/// Unbindable values must name the position and type rather than stringifying.
#[test]
fn unsupported_parameter_types_are_rejected_by_position() {
    use std::cell::RefCell;
    use std::rc::Rc;
    let list = Value::List(Rc::new(RefCell::new(vec![Value::Int(1)])));
    let error = encode_all(&[Value::Int(1), list]).expect_err("a list must not bind");
    assert!(
        error.contains("$2"),
        "should name the position, got: {error}"
    );
    assert!(error.contains("list"), "should name the type, got: {error}");
}
