//! `{% for item in items %}` evaluation.
//!
//! Each iteration renders the body against a child context holding the loop variable.
//! Cloning the context rather than mutating it means the binding cannot outlive the loop
//! or permanently shadow a parent key of the same name.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::template_block::{iterable, matching_end, render_with, Render};
use super::template_scan::Piece;
use crate::value::Value;

/// Render a `for` block, returning the index just past its `endfor`.
///
/// # Errors
///
/// Returns an error for a malformed header, a non-list subject, or an unbalanced block.
pub(super) fn run(
    pieces: &[Piece<'_>],
    index: usize,
    body: &str,
    context: &Value,
    state: &Render<'_>,
    out: &mut String,
) -> Result<usize, String> {
    let (name, subject) = parse(body)?;
    let end = matching_end(pieces, index)?;
    for item in iterable(context, subject)? {
        let scope = child(context, name, item)?;
        out.push_str(&render_with(&pieces[index + 1..end], &scope, state)?);
    }
    Ok(end + 1)
}

/// Parse `for <name> in <subject>`.
///
/// # Errors
///
/// Returns an error naming what was found when the header is malformed.
fn parse(body: &str) -> Result<(&str, &str), String> {
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
/// # Errors
///
/// Returns an error when the context is not a map.
fn child(context: &Value, name: &str, item: Value) -> Result<Value, String> {
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
