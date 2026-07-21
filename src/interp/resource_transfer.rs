//! Interpreter-side resource transfer boundaries.

use std::{cell::RefCell, rc::Rc};

use crate::value::{resource::transfer, Env, Value};

use super::{EvalResult, Unwind};

pub(super) fn define(
    env: &Rc<RefCell<Env>>,
    name: &str,
    value: Value,
    mutable: bool,
) -> EvalResult {
    transfer::validate(&value, &format!("binding `{name}`"))?;
    env.borrow_mut().define(name, value, mutable);
    Ok(Value::Nil)
}

pub(super) fn list(values: Vec<Value>, operation: &str) -> EvalResult {
    for value in &values {
        transfer::validate(value, operation)?;
    }
    Ok(Value::List(Rc::new(RefCell::new(values))))
}

pub(super) fn returned(value: Value) -> EvalResult {
    transfer::validate(&value, "function return").map_err(Unwind::Error)?;
    Ok(value)
}

pub(super) fn validate(value: &Value, operation: &str) -> Result<(), Unwind> {
    transfer::validate(value, operation).map_err(Unwind::Error)
}
