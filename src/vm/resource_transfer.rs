//! VM-side resource transfer boundaries.

use std::{cell::RefCell, rc::Rc};

use crate::value::{resource::transfer, Env, Value};

use super::Unwind;

pub(super) fn define(
    env: &Rc<RefCell<Env>>,
    name: &str,
    value: Value,
    mutable: bool,
) -> Result<(), Unwind> {
    transfer::validate(&value, &format!("binding `{name}`"))?;
    env.borrow_mut().define(name, value, mutable);
    Ok(())
}

pub(super) fn assign(env: &Rc<RefCell<Env>>, name: &str, value: &Value) -> Result<(), Unwind> {
    let value = transfer::retained(value, &format!("assignment to `{name}`"))?;
    env.borrow_mut().assign(name, value)?;
    Ok(())
}

pub(super) fn list(values: Vec<Value>) -> Result<Value, Unwind> {
    for value in &values {
        transfer::validate(value, "list literal")?;
    }
    Ok(Value::List(Rc::new(RefCell::new(values))))
}

pub(super) fn validate(value: &Value, operation: &str) -> Result<(), Unwind> {
    transfer::validate(value, operation).map_err(Unwind::Error)
}

pub(super) fn returned(value: Option<&Value>) -> Result<(), Unwind> {
    value.map_or(Ok(()), |value| validate(value, "function return"))
}
