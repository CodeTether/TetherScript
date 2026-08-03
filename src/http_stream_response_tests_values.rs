//! Value constructors shared by the streaming-response unit tests.
//!
//! Split from [`super::support`] so value building and fake I/O sinks stay
//! separate concerns, and so each file stays inside the 50-line limit.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::{NativeFn, NativeFunc, Value};

/// Build a `Value::Str`.
///
/// # Arguments
///
/// * `text` — Contents of the string.
///
/// # Returns
///
/// The wrapped value. Infallible.
pub(super) fn str_value(text: &str) -> Value {
    Value::Str(Rc::new(text.to_string()))
}

/// Build a `Value::Map` from key/value pairs.
///
/// # Arguments
///
/// * `entries` — Keys and their values, in any order.
///
/// # Returns
///
/// The wrapped map. Infallible.
pub(super) fn map(entries: &[(&str, Value)]) -> Value {
    let mut inner = HashMap::new();
    for (key, value) in entries {
        inner.insert((*key).to_string(), value.clone());
    }
    Value::Map(Rc::new(RefCell::new(inner)))
}

/// A zero-argument native that yields nothing; a stand-in callable.
///
/// # Returns
///
/// A `Value::Native`. The body is never called by the tests that only need the
/// value to *be* callable; the pump tests drive returns through
/// [`super::support::ScriptedRuntime`] instead.
pub(super) fn native() -> Value {
    Value::Native(Rc::new(NativeFn {
        name: "test_stream".to_string(),
        arity: Some(0),
        func: NativeFunc::Pure(Box::new(|_| Ok(Value::Nil))),
    }))
}
