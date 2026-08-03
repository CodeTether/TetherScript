//! `{{ super() }}` — rendering the parent's block body inside an override.
//!
//! A child that overrides `{% block head %}` usually wants to *add* to the parent's head rather
//! than replace it: the reference's `pages/page.html.tera` does exactly that, and the parent's head
//! is where every `<link rel="stylesheet">` lives. Without `super()` an override silently drops
//! them, and the page arrives unstyled with no error anywhere.
//!
//! The parent body is carried as source text rather than as pieces because an override is itself
//! re-scanned at render time, so both sides have to be in the same form.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// Context key holding the parent block's source while an override renders.
///
/// A reserved name rather than a field on `Render`, so `super()` needs no change to the signature
/// every evaluation layer already threads. The leading `@` cannot collide with a template variable:
/// the scanner would never produce it as an identifier.
pub(super) const SUPER_KEY: &str = "@super";

/// Clone `context` with the parent block's body bound for `super()`.
///
/// # Arguments
///
/// * `context` — The context the override renders against.
/// * `parent` — Source text of the block body being overridden.
///
/// # Returns
///
/// A child context. Cloned rather than mutated so the binding cannot outlive this block or leak
/// into a sibling — the same reason a loop builds a child scope.
///
/// # Errors
///
/// Returns an error when the context is not a map.
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
///
/// Returns `None` outside an override, which is what makes a bare `{{ super() }}` in a
/// non-overriding block an error a caller can act on rather than silently empty output.
pub(super) fn parent_of(context: &Value) -> Option<String> {
    let Value::Map(fields) = context else {
        return None;
    };
    match fields.borrow().get(SUPER_KEY) {
        Some(Value::Str(text)) => Some((**text).clone()),
        _ => None,
    }
}
