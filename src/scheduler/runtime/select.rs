//! Deterministic selection of the first ready cooperative task.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::resource::ResourceKind;
use crate::value::{Runtime, Value};

use super::await_value;

pub(crate) fn value(runtime: &mut dyn Runtime, tasks: &[Value]) -> Result<Value, String> {
    if tasks.is_empty() {
        return Err("select: expected at least one task".into());
    }
    for task in tasks {
        let Value::Resource(resource) = task else {
            return Err(format!("select: expected task, got {}", task.type_name()));
        };
        if resource.borrow().kind() != ResourceKind::Task {
            return Err(format!(
                "select: expected task, got {}",
                resource.borrow().kind().type_name()
            ));
        }
    }
    let index = tasks
        .iter()
        .position(|task| match task {
            Value::Resource(resource) => resource.borrow().task_ready(),
            _ => false,
        })
        .unwrap_or(0);
    let selected = await_value(runtime, tasks[index].clone())?;
    let mut result = HashMap::new();
    result.insert("index".into(), Value::Int(index as i64));
    result.insert("value".into(), selected);
    Ok(Value::Map(Rc::new(RefCell::new(result))))
}
