//! `{% for item in items %}` evaluation.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::template_block::{iterable, matching_end, render_with, Render};
use super::template_scan::Piece;
use crate::value::Value;

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
    for item in iterable(context, subject, state.lenient)? {
        let scope = child(context, name, item)?;
        out.push_str(&render_with(&pieces[index + 1..end], &scope, state)?);
    }
    Ok(end + 1)
}

/// Parse `for <name> in <subject>`.
///
/// The subject is everything after `in`, not just the next word: a real view writes
/// `range(end=average_rating_rounded | default(value=5))`, which contains spaces.
fn parse(body: &str) -> Result<(&str, &str), String> {
    let rest = body.strip_prefix("for").unwrap_or(body).trim();
    let mid = rest.find(" in ").ok_or("template: `for` needs ` in `")?;
    let name = rest[..mid].trim();
    let subject = rest[mid + 4..].trim();
    if name.is_empty() {
        return Err("template: `for` needs a variable name".into());
    }
    if subject.is_empty() {
        return Err("template: `for` needs something to iterate".into());
    }
    Ok((name, subject))
}

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
