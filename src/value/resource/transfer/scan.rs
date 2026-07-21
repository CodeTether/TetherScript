//! Cycle-safe discovery of resources inside aggregate values.

use std::{collections::HashSet, rc::Rc};

use crate::value::Value;

use super::{aggregate, unique};

pub(super) fn validate(value: &Value, operation: &str) -> Result<(), String> {
    visit(value, operation, &mut HashSet::new()).map(|_| ())
}

pub(super) fn visit(
    value: &Value,
    operation: &str,
    seen: &mut HashSet<*const ()>,
) -> Result<Option<&'static str>, String> {
    match value {
        Value::Resource(resource) => {
            let kind = resource.borrow().kind().type_name();
            unique::check(Rc::strong_count(resource), operation, kind, "resource")?;
            Ok(Some(kind))
        }
        Value::List(values) => aggregate::list(values, operation, seen),
        Value::Map(values) => aggregate::map(values, operation, seen),
        Value::Result(result) => aggregate::result(result, operation, seen),
        _ => Ok(None),
    }
}
