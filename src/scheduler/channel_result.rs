//! Result and map construction for channel built-ins.
//!
//! Channel operations are recoverable, so every built-in returns a
//! `Result` value rather than raising. Receiving additionally has three
//! non-error shapes — value, end-of-stream, and parked — so it answers with a
//! small status map instead of overloading `nil`, which a script could not tell
//! apart from a legitimately sent `nil`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::{ResultValue, Value};

/// Wrap a fallible channel operation as a language `Result` value.
pub(super) fn result(outcome: Result<Value, String>) -> Value {
    Value::Result(Rc::new(
        outcome.map_or_else(ResultValue::Err, ResultValue::Ok),
    ))
}

/// Wrap a unit-returning channel operation as `Ok(nil)` or `Err(message)`.
pub(super) fn unit(outcome: Result<(), String>) -> Value {
    result(outcome.map(|()| Value::Nil))
}

/// Build a `str` value without allocating a temporary at the call site.
pub(super) fn text(value: &str) -> Value {
    Value::Str(Rc::new(value.to_string()))
}

/// Build a status map from `(key, value)` pairs.
pub(super) fn map(entries: Vec<(&str, Value)>) -> Value {
    let mut fields = HashMap::new();
    for (key, value) in entries {
        fields.insert(key.to_string(), value);
    }
    Value::Map(Rc::new(RefCell::new(fields)))
}
