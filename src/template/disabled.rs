//! Dependency-free response when optional Tera compatibility is disabled.

use std::rc::Rc;

use crate::value::{ResultValue, Value};

pub(super) fn call(_args: &[Value]) -> Result<Value, String> {
    Ok(Value::Result(Rc::new(ResultValue::Err(
        "tera_render: rebuild tetherscript with the `tera` feature".into(),
    ))))
}
