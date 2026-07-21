//! Conversion of values into scheduled task resources.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::resource::{OwnedResource, ResourceKind};
use crate::value::Value;

pub(crate) fn value(value: Value) -> Result<Value, String> {
    if matches!(&value, Value::Resource(resource) if resource.borrow().kind() == ResourceKind::Task)
    {
        return Ok(value);
    }
    OwnedResource::completed_task(value).map(|task| Value::Resource(Rc::new(RefCell::new(task))))
}
