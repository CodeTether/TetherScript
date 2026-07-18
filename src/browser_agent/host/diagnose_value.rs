//! Runtime value construction for diagnostic collections.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

pub(super) fn list(values: Vec<Value>) -> Value {
    Value::List(Rc::new(RefCell::new(values)))
}

pub(super) fn strings(values: impl IntoIterator<Item = String>) -> Value {
    list(
        values
            .into_iter()
            .map(super::super::super::value::string)
            .collect(),
    )
}

pub(super) fn optional(value: Option<String>) -> Value {
    value
        .map(super::super::super::value::string)
        .unwrap_or(Value::Nil)
}
