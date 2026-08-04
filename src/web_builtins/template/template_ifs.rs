//! `{% if %}` conditional rendering.

use super::template_block::{render_with, Render};
use super::template_branch::branches;
use super::template_condition::evaluate;
use super::template_scan::Piece;
use crate::value::Value;

/// Render the first satisfied branch of an `if`/`elif`/`else` chain.
pub(super) fn ifs(
    p: &[Piece],
    i: usize,
    c: &Value,
    s: &Render,
    o: &mut String,
) -> Result<usize, String> {
    let (found, end) = branches(p, i)?;
    for (pos, br) in found.iter().enumerate() {
        let taken = match br.condition {
            Some(k) => evaluate(c, k)?,
            None => true,
        };
        if taken {
            let stop = found.get(pos + 1).map_or(end, |n| n.at);
            o.push_str(&render_with(&p[br.at + 1..stop], c, s)?);
            break;
        }
    }
    Ok(end + 1)
}
