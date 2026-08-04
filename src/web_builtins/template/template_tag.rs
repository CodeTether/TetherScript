//! `{% ... %}` tag dispatch, including `{% set %}`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::template_block::{render_with, Render};
use super::template_scan::Piece;
use crate::value::Value;

pub(super) fn tag(
    pieces: &[Piece<'_>],
    index: usize,
    body: &str,
    context: &Value,
    state: &Render<'_>,
    out: &mut String,
) -> Result<usize, String> {
    let mut words = body.split_whitespace();
    match words.next().unwrap_or("") {
        "if" => super::template_ifs::ifs(pieces, index, context, state, out),
        "for" => super::template_loop::run(pieces, index, body, context, state, out),
        "block" => super::template_blocks::run(pieces, index, body, context, state, out),
        "set" => set_var(pieces, index, body, context, state, out),
        "macro" => super::template_macro::run(pieces, index, body, context, state, out),
        "include" => super::template_include::run(body, context, state, out).map(|()| index + 1),
        "extends" => Err("template: `extends` must be the first tag".into()),
        k @ ("else" | "elif" | "endif" | "endfor" | "endblock" | "endmacro") => {
            Err(format!("template: `{k}` without a matching opener"))
        }
        other => reject(other),
    }
}

fn reject(keyword: &str) -> Result<usize, String> {
    Err(format!(
        "template: unsupported tag `{keyword}` (have: if, else, endif, for, endfor, \
         block, endblock, extends, include, set, macro)"
    ))
}

fn set_var(
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
