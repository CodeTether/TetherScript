//! Task validation and runtime-driven awaiting.

use crate::value::resource::ResourceKind;
use crate::value::{Runtime, Value};

pub(crate) fn value(runtime: &mut dyn Runtime, value: Value) -> Result<Value, String> {
    let resource = match value {
        Value::Resource(resource) => resource,
        other => {
            return Err(format!(
                "await: expected task resource, got {}",
                other.type_name()
            ))
        }
    };
    if resource.borrow().kind() != ResourceKind::Task {
        return Err(format!(
            "await: expected task resource, got {}",
            resource.borrow().kind().type_name()
        ));
    }
    resource.borrow_mut().await_task(runtime)
}
