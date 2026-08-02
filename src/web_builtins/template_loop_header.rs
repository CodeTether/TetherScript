//! `for` header parsing and per-iteration scope construction.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// Parse `for <name> in <subject>`.
///
/// # Errors
///
/// Returns an error naming what was found when the header is malformed.
pub(super) fn parse(body: &str) -> Result<(&str, &str), String> {
    let mut words = body.split_whitespace();
    words.next();
    let name = words
        .next()
        .ok_or("template: `for` needs a variable name")?;
    let keyword = words.next().unwrap_or("");
    if keyword != "in" {
        return Err(format!(
            "template: `for {name}` must be followed by `in`, got `{keyword}`"
        ));
    }
    let subject = words
        .next()
        .ok_or("template: `for` needs something to iterate")?;
    Ok((name, subject))
}

/// Clone the parent context and bind the loop variable in the copy.
///
/// Cloning rather than mutating means the binding cannot outlive the loop or
/// permanently shadow a parent key of the same name.
///
/// # Errors
///
/// Returns an error when the context is not a map.
pub(super) fn child(context: &Value, name: &str, item: Value) -> Result<Value, String> {
    let Value::Map(parent) = context else {
        return Err(format!(
            "template: context must be a map, got {}",
            context.type_name()
        ));
    };
    let mut scope: HashMap<String, Value> = parent.borrow().clone();
    scope.insert(name.to_string(), item);
    Ok(Value::Map(Rc::new(RefCell::new(scope))))
}
