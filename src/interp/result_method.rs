//! Method dispatch for recoverable result values.

use std::rc::Rc;

use crate::value::{ResultValue, Value};

pub(super) fn call(result: &Rc<ResultValue>, name: &str, args: &[Value]) -> Result<Value, String> {
    match (result.as_ref(), name, args) {
        (ResultValue::Ok(_), "is_ok", []) => Ok(Value::Bool(true)),
        (ResultValue::Err(_), "is_ok", []) => Ok(Value::Bool(false)),
        (ResultValue::Ok(_), "is_err", []) => Ok(Value::Bool(false)),
        (ResultValue::Err(_), "is_err", []) => Ok(Value::Bool(true)),
        (ResultValue::Ok(value), "unwrap", []) => Ok(value.clone()),
        (ResultValue::Err(error), "unwrap", []) => Err(format!("called unwrap on Err({error:?})")),
        (ResultValue::Ok(value), "unwrap_or", [_]) => Ok(value.clone()),
        (ResultValue::Err(_), "unwrap_or", [default]) => Ok(default.clone()),
        (ResultValue::Ok(value), "ok", []) => Ok(value.clone()),
        (ResultValue::Err(_), "ok", []) => Ok(Value::Nil),
        (ResultValue::Ok(_), "err", []) => Ok(Value::Nil),
        (ResultValue::Err(error), "err", []) => Ok(Value::Str(Rc::new(error.clone()))),
        (_, method, _) => Err(format!("no method `{method}` on result")),
    }
}
