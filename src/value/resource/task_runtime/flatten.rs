//! Recursive flattening of nested asynchronous task results.

use crate::value::resource::ResourceKind;
use crate::value::{Runtime, Value};

pub(super) fn value(value: Value, runtime: &mut dyn Runtime) -> Result<Value, String> {
    let Value::Resource(resource) = &value else {
        return Ok(value);
    };
    if resource.borrow().kind() != ResourceKind::Task {
        return Ok(value);
    }
    let resource = resource.clone();
    resource.borrow_mut().await_task(runtime)
}
