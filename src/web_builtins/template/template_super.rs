//! `{{ super() }}` — rendering the parent's block body inside an override.
//!
//! The parent body is carried as source text and re-scanned when `super()` is reached.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::template_block::{render_with, Render};
use super::template_scan::scan;
use crate::value::Value;

/// Context key holding the parent block's source while an override renders.
pub(super) const SUPER_KEY: &str = "@super";

/// Clone `context` with the parent block's body bound.
pub(super) fn with_parent(context: &Value, parent: &str) -> Result<Value, String> {
    let Value::Map(fields) = context else {
        return Err(format!(
            "template: context must be a map, got {}",
            context.type_name()
        ));
    };
    let mut scope: HashMap<String, Value> = fields.borrow().clone();
    scope.insert(
        SUPER_KEY.to_string(),
        Value::Str(Rc::new(parent.to_string())),
    );
    Ok(Value::Map(Rc::new(RefCell::new(scope))))
}

/// The parent body bound for the current block, if any.
pub(super) fn parent_of(context: &Value) -> Option<String> {
    let Value::Map(fields) = context else {
        return None;
    };
    match fields.borrow().get(SUPER_KEY) {
        Some(Value::Str(text)) => Some((**text).clone()),
        _ => None,
    }
}

/// Render the parent block's body in place of `{{ super() }}`.
///
/// # Errors
/// Returns an error when `super()` appears outside an overriding block.
pub(super) fn render(context: &Value, state: &Render<'_>) -> Result<String, String> {
    let Some(parent) = parent_of(context) else {
        return Err("template: `super()` is only valid inside an overriding block".into());
    };
    render_with(&scan(&parent)?, context, state)
}
