//! `{% ... %}` tag dispatch.

use super::template_block::Render;
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
        "set" => super::template_set_var::set_var(pieces, index, body, context, state, out),
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
