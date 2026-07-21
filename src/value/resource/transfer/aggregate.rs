//! Traversal of resource-owning aggregate values.

use std::{collections::HashSet, rc::Rc};

use crate::value::{ResultValue, Value};

use super::{scan, unique};

pub(super) fn list(
    values: &Rc<std::cell::RefCell<Vec<Value>>>,
    operation: &str,
    seen: &mut HashSet<*const ()>,
) -> Result<Option<&'static str>, String> {
    let count = Rc::strong_count(values);
    if !seen.insert(Rc::as_ptr(values).cast()) {
        return Ok(None);
    }
    let mut found = None;
    for value in values.borrow().iter() {
        found = scan::visit(value, operation, seen)?.or(found);
    }
    unique::aggregate(count, operation, found, "list")
}

pub(super) fn map(
    values: &Rc<std::cell::RefCell<std::collections::HashMap<String, Value>>>,
    operation: &str,
    seen: &mut HashSet<*const ()>,
) -> Result<Option<&'static str>, String> {
    let count = Rc::strong_count(values);
    if !seen.insert(Rc::as_ptr(values).cast()) {
        return Ok(None);
    }
    let mut found = None;
    for value in values.borrow().values() {
        found = scan::visit(value, operation, seen)?.or(found);
    }
    unique::aggregate(count, operation, found, "map")
}

pub(super) fn result(
    result: &Rc<ResultValue>,
    operation: &str,
    seen: &mut HashSet<*const ()>,
) -> Result<Option<&'static str>, String> {
    if !seen.insert(Rc::as_ptr(result).cast()) {
        return Ok(None);
    }
    let ResultValue::Ok(value) = result.as_ref() else {
        return Ok(None);
    };
    let found = scan::visit(value, operation, seen)?;
    unique::aggregate(Rc::strong_count(result), operation, found, "result")
}
