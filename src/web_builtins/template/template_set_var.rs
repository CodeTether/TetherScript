//! `{% set var = expr %}` tag handler.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::template_block::{render_with, Render};
use super::template_scan::Piece;
use crate::value::Value;

pub(super) fn set_var(
    p: &[Piece],
    i: usize,
    b: &str,
    c: &Value,
    s: &Render,
    o: &mut String,
) -> Result<usize, String> {
    let rest = b.trim().strip_prefix("set").unwrap_or(b).trim();
    let eq = rest.find('=').ok_or("template: `set` needs `=`")?;
    let name = rest[..eq].trim();
    if name.is_empty() {
        return Err("template: `set` needs a variable name".into());
    }
    let val =
        super::template_emit_default::value_of(rest[eq + 1..].trim(), c).unwrap_or(Value::Nil);
    let Value::Map(fields) = c else {
        return Err(format!(
            "template: context must be a map, got {}",
            c.type_name()
        ));
    };
    let mut scope: HashMap<String, Value> = fields.borrow().clone();
    scope.insert(name.to_string(), val);
    let scope_val = Value::Map(Rc::new(RefCell::new(scope)));
    o.push_str(&render_with(&p[i + 1..], &scope_val, s)?);
    Ok(p.len())
}
