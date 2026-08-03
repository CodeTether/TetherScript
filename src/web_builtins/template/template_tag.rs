//! `{% ... %}` tag dispatch.

use super::template_block::Render;
use super::template_branch::branches;
use super::template_condition::evaluate;
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
        "if" => ifs(pieces, index, context, state, out),
        "for" => super::template_loop::run(pieces, index, body, context, state, out),
        "block" => super::template_blocks::run(pieces, index, body, context, state, out),
        "set" => super::template_set::run(pieces, index, body, context, state, out),
        "macro" => super::template_macro::run(pieces, index, body, context, state, out),
        "include" => super::template_include::run(body, context, state, out).map(|()| index + 1),
        "extends" => Err("template: `extends` must be the first tag".into()),
        k @ ("else" | "elif" | "endif" | "endfor" | "endblock" | "endmacro") => {
            Err(format!("template: `{k}` without a matching opener"))
        }
        other => super::template_step::reject(other),
    }
}

fn ifs(p: &[Piece], i: usize, c: &Value, s: &Render, o: &mut String) -> Result<usize, String> {
    let (found, end) = branches(p, i)?;
    for (pos, br) in found.iter().enumerate() {
        let taken = match br.condition {
            Some(k) => evaluate(c, k)?,
            None => true,
        };
        if taken {
            let stop = found.get(pos + 1).map_or(end, |n| n.at);
            o.push_str(&super::template_block::render_with(
                &p[br.at + 1..stop],
                c,
                s,
            )?);
            break;
        }
    }
    Ok(end + 1)
}
