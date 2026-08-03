//! Unit tests for the `redis` capability seam.
//!
//! These exercise the parts of the capability that need no server: argument coercion and
//! arity, and the error wording. The other invariants have their own files —
//! `redis_cap_nil_vs_empty.rs`, `redis_cap_ttl.rs`, `redis_cap_ttl_reply.rs`, and
//! `redis_cap_crlf_safety.rs`.
//!
//! # Why there is no live-server test here
//!
//! Everything asserted below is a decision the capability makes *before* a round trip,
//! which is where the failures that matter live: a coerced-away type, a silently dropped
//! argument, a stringified key. Those are the bugs a script would suffer silently, and
//! none of them needs Redis running to prove.

use std::cell::RefCell;
use std::rc::Rc;

use tetherscript::redis_cap::{args, args_error, args_missing, coerce_bytes, coerce_int};
use tetherscript::value::Value;

/// A script `str` argument.
fn text(value: &str) -> Value {
    Value::Str(Rc::new(value.to_string()))
}

/// A script `bytes` argument.
fn raw(value: &[u8]) -> Value {
    Value::Bytes(Rc::new(RefCell::new(value.to_vec())))
}

// ---------------------------------------------------------------------------
// Coercion names the offending parameter and its actual type.
// ---------------------------------------------------------------------------

#[test]
fn a_key_mismatch_names_the_parameter_and_the_actual_type() {
    let error = coerce_bytes::bytes("redis.get", "key", &Value::Int(7)).unwrap_err();
    assert_eq!(
        error,
        "redis.get: parameter `key` must be a str or bytes, got int"
    );
}

#[test]
fn a_value_mismatch_names_the_value_parameter_not_the_key() {
    let error = coerce_bytes::bytes("redis.set", "value", &Value::Nil).unwrap_err();
    assert!(error.contains("`value`"), "got: {error}");
    assert!(error.contains("got nil"), "got: {error}");
}

#[test]
fn a_delta_mismatch_names_the_delta_parameter() {
    let error = coerce_int::int("redis.incrby", "delta", &Value::Float(1.9)).unwrap_err();
    assert_eq!(
        error,
        "redis.incrby: parameter `delta` must be an int, got float"
    );
}

/// A float is refused rather than truncated: `incrby(k, 1.9)` adding 1 is a wrong answer.
#[test]
fn a_float_delta_is_refused_rather_than_truncated() {
    assert!(coerce_int::int("redis.incrby", "delta", &Value::Float(1.9)).is_err());
}

/// A numeric string is refused rather than parsed, so a typo surfaces at the call site.
#[test]
fn a_numeric_string_is_not_silently_accepted_as_an_int() {
    assert!(coerce_int::int("redis.incrby", "delta", &text("5")).is_err());
}

/// An int key is refused rather than stringified: `get(1)` and `get("1")` must differ.
#[test]
fn an_int_key_is_refused_rather_than_stringified() {
    let error = coerce_bytes::bytes("redis.get", "key", &Value::Int(1)).unwrap_err();
    assert!(error.contains("`key`"), "got: {error}");
}

/// A negative delta is legitimate; it is how a script decrements.
#[test]
fn a_negative_delta_is_accepted() {
    assert_eq!(
        coerce_int::int("redis.incrby", "delta", &Value::Int(-5)).unwrap(),
        -5
    );
}

#[test]
fn both_str_and_bytes_are_accepted_as_keys_and_values() {
    assert_eq!(
        coerce_bytes::bytes("redis.get", "key", &text("session:42")).unwrap(),
        b"session:42".to_vec()
    );
    assert_eq!(
        coerce_bytes::bytes("redis.set", "value", &raw(&[0x00, 0xff])).unwrap(),
        vec![0x00, 0xff]
    );
}

/// A capability value is not a key, and the error says which type arrived.
#[test]
fn a_list_key_is_refused_naming_its_type() {
    let list = Value::List(Rc::new(RefCell::new(vec![text("k")])));
    let error = coerce_bytes::bytes("redis.del", "key", &list).unwrap_err();
    assert!(error.contains("got list"), "got: {error}");
}

// ---------------------------------------------------------------------------
// Arity: absent and extra arguments are both refused.
// ---------------------------------------------------------------------------

#[test]
fn a_missing_argument_is_reported_as_missing_not_as_nil() {
    let supplied = [text("k")];
    let error = args::at("redis.incrby", "delta", &supplied, 1).unwrap_err();
    assert_eq!(error, "redis.incrby: parameter `delta` is required");
    assert!(
        !error.contains("nil"),
        "absent and explicitly-nil are different mistakes: {error}"
    );
}

/// An extra argument is refused, not dropped: a silently ignored TTL is a real bug.
#[test]
fn an_extra_argument_is_refused_rather_than_ignored() {
    let supplied = [text("k"), text("v"), Value::Int(60)];
    let error = args::exactly("redis.set", &supplied, 2).unwrap_err();
    assert!(error.contains("takes 2"), "got: {error}");
    assert!(error.contains("got 3"), "got: {error}");
}

#[test]
fn ping_takes_no_arguments() {
    assert!(args::exactly("redis.ping", &[], 0).is_ok());
    assert!(args::exactly("redis.ping", &[text("x")], 0).is_err());
}

#[test]
fn setex_requires_all_three_arguments() {
    let two = [text("k"), text("v")];
    assert!(args::exactly("redis.setex", &two, 3).is_err());
    let three = [text("k"), text("v"), Value::Int(60)];
    assert!(args::exactly("redis.setex", &three, 3).is_ok());
}

// ---------------------------------------------------------------------------
// The wording helpers are the single source of truth, so they are asserted directly.
// ---------------------------------------------------------------------------

/// An error message must name the parameter's type, never echo its contents.
#[test]
fn the_mismatch_helper_never_prints_the_value() {
    let secret = text("s3cr3t-session-token");
    let error = args_error::mismatch("redis.set", "value", "an int", &secret);
    assert!(
        !error.contains("s3cr3t"),
        "an error message must not echo a stored value: {error}"
    );
    assert!(error.contains("got str"), "got: {error}");
}

/// The same guarantee for a `bytes` payload, which is what a session token looks like.
#[test]
fn a_bytes_mismatch_never_prints_the_payload() {
    let payload = raw(b"s3cr3t-bytes");
    let error = args_error::mismatch("redis.incrby", "delta", "an int", &payload);
    assert!(!error.contains("s3cr3t"), "leaked the payload: {error}");
    assert!(error.contains("got bytes"), "got: {error}");
}

#[test]
fn the_missing_helper_names_the_method_and_parameter() {
    assert_eq!(
        args_missing::missing("redis.get", "key"),
        "redis.get: parameter `key` is required"
    );
}
