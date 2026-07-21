//! Unit coverage for owned-resource invariants.

mod buffers;
mod lifecycle;
mod os;
mod render_surface;
mod transfer;
mod transfer_nested;

use crate::value::{ResultValue, Value};

pub(super) fn ok(value: Value) -> Value {
    match value {
        Value::Result(result) => match result.as_ref() {
            ResultValue::Ok(value) => value.clone(),
            ResultValue::Err(error) => panic!("expected Ok resource result, got {error}"),
        },
        other => panic!("expected resource result, got {other}"),
    }
}

pub(super) fn error(value: Value) -> String {
    match value {
        Value::Result(result) => match result.as_ref() {
            ResultValue::Err(error) => error.clone(),
            ResultValue::Ok(value) => panic!("expected Err resource result, got {value}"),
        },
        other => panic!("expected resource result, got {other}"),
    }
}
