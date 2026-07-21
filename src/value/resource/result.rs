//! Recoverable resource-operation results.

use std::rc::Rc;

use crate::value::{ResultValue, Value};

pub(super) fn value(result: Result<Value, String>) -> Value {
    Value::Result(Rc::new(
        result.map_or_else(ResultValue::Err, ResultValue::Ok),
    ))
}

pub(super) fn nil(result: Result<(), String>) -> Value {
    value(result.map(|()| Value::Nil))
}
