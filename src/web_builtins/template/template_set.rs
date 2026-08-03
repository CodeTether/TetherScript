//! `{% set var = expr %}` support.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::template_block::{render_with, Render};
use super::template_scan::Piece;
use crate::value::Value;

/// Bind `set var = expr` and render remaining pieces in the new scope.
pub(super) fn run(
    pieces: &[Piece],
    index: usize,
    body: &str,
    context: &Value,
    state: &Render,
    out: &mut String,
) -> Result<usize, String> {
    let scope = apply(body, context)?;
    out.push_str(&render_with(&pieces[index + 1..], &scope, state)?);
    Ok(pieces.len())
}

/// Apply a `set var = expr` to the context, returning a new context with the binding.
pub(super) fn apply(body: &str, context: &Value) -> Result<Value, String> {
    let rest = body.trim().strip_prefix("set").unwrap_or(body).trim();
    let eq = rest.find('=').ok_or("template: `set` needs `=`")?;
    let name = rest[..eq].trim();
    if name.is_empty() {
        return Err("template: `set` needs a variable name".into());
    }
    let expr = rest[eq + 1..].trim();
    let value = super::template_emit_default::value_of(expr, context).unwrap_or(Value::Nil);
    let Value::Map(fields) = context else {
        return Err(format!(
            "template: context must be a map, got {}",
            context.type_name()
        ));
    };
    let mut scope: HashMap<String, Value> = fields.borrow().clone();
    scope.insert(name.to_string(), value);
    Ok(Value::Map(Rc::new(RefCell::new(scope))))
}
