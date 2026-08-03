//! Assertion helpers for the `chan_*` built-in tests.
//!
//! Every built-in answers with a language `Result` wrapping either a scalar or a
//! status map, so these helpers unwrap that shape once instead of repeating the
//! same five lines in every test. [`reset`] exists because channel state is
//! thread-local and `--test-threads=1` runs every test on one thread.

use std::rc::Rc;

use crate::scheduler::channel;
use crate::value::{ResultValue, Value};

/// Discard wakeups and parks left by a previously executed test on this thread.
///
/// The built-ins park whatever `current_task` reports, which is the same id for
/// every test in this file, so each test must start from a clean park table when
/// the harness runs them all on one thread.
pub(super) fn reset() {
    let _stale = channel::take_wakeups();
    let _parked = channel::cancel_task(channel::current_task());
}

/// Build a `str` value.
pub(super) fn text(value: &str) -> Value {
    Value::Str(Rc::new(value.to_string()))
}

/// Unwrap the `Ok` payload of a built-in result, panicking with the error.
pub(super) fn ok(value: Value) -> Value {
    let Value::Result(result) = value else {
        panic!("expected a result value, got {}", value.type_name());
    };
    match result.as_ref() {
        ResultValue::Ok(inner) => inner.clone(),
        ResultValue::Err(message) => panic!("expected Ok, got Err({message})"),
    }
}

/// Unwrap the `Err` message of a built-in result.
pub(super) fn err(value: Value) -> String {
    let Value::Result(result) = value else {
        panic!("expected a result value, got {}", value.type_name());
    };
    match result.as_ref() {
        ResultValue::Err(message) => message.clone(),
        ResultValue::Ok(_) => panic!("expected Err, got Ok"),
    }
}

/// Unwrap an opened channel handle.
pub(super) fn handle(value: Value) -> i64 {
    match ok(value) {
        Value::Int(handle) => handle,
        other => panic!("expected an int handle, got {}", other.type_name()),
    }
}
