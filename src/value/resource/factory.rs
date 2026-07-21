//! Shared resource-factory value construction.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

use super::{result, OwnedResource};

pub(super) fn resource(value: Result<OwnedResource, String>) -> Value {
    result::value(value.map(|resource| Value::Resource(Rc::new(RefCell::new(resource)))))
}

pub(super) fn direct(resource: OwnedResource) -> Value {
    result::value(Ok(Value::Resource(Rc::new(RefCell::new(resource)))))
}

pub(super) fn port(value: &Value, label: &str) -> Result<u16, String> {
    match value {
        Value::Int(port) => u16::try_from(*port)
            .map_err(|_| format!("{label}: port must be in 0..=65535, got {port}")),
        other => Err(format!(
            "{label}: port must be int, got {}",
            other.type_name()
        )),
    }
}
