//! `{% block name %}` collection and rendering.
//!
//! Collection runs before rendering so inheritance can override blocks by name: a
//! child's blocks are needed while rendering its *parent*, not itself. Rendering then
//! substitutes an override when one exists, and falls back to the parent's own body
//! otherwise — which is what makes a block body a default rather than a requirement.

use std::collections::HashMap;

use super::template_block::render_with;
use super::template_block::Render;
use super::template_delimit::matching_end;
use super::template_scan::{scan, Piece};
use crate::value::Value;

/// Blocks defined by a template, keyed by name.
pub(super) type Blocks<'a> = HashMap<String, Vec<Piece<'a>>>;

/// Collect every block a template defines, nested ones included.
///
/// Nested blocks are collected because Tera allows a child to override an inner block
/// without restating its enclosing one.
///
/// # Errors
///
/// Returns an error for an unnamed block or an unbalanced `endblock`.
pub(super) fn collect<'a>(pieces: &[Piece<'a>]) -> Result<Blocks<'a>, String> {
    let mut blocks = HashMap::new();
    for (index, piece) in pieces.iter().enumerate() {
        let Piece::Tag(body) = piece else { continue };
        if body.split_whitespace().next() != Some("block") {
            continue;
        }
        let name = name_of(body)?;
        let end = matching_end(pieces, index)?;
        blocks.insert(name.to_string(), pieces[index + 1..end].to_vec());
    }
    Ok(blocks)
}

/// Extract the name from `block <name>`.
///
/// # Errors
///
/// Returns an error when the name is missing: an anonymous block cannot be
/// overridden, so it is always a mistake rather than a shorthand.
pub(super) fn name_of(body: &str) -> Result<&str, String> {
    body.split_whitespace()
        .nth(1)
        .ok_or_else(|| "template: `block` needs a name".to_string())
}

/// Render a `block` tag, returning the index just past its `endblock`.
///
/// # Errors
///
/// Returns an error for an unnamed block, an unbalanced `endblock`, or any failure
/// rendering the chosen body.
pub(super) fn run(
    pieces: &[Piece<'_>],
    index: usize,
    body: &str,
    context: &Value,
    state: &Render<'_>,
    out: &mut String,
) -> Result<usize, String> {
    let name = name_of(body)?;
    let end = matching_end(pieces, index)?;
    let rendered = match state.overrides.get(name) {
        // Re-scanned here rather than at collection time so a block body may itself
        // contain blocks, ifs, and loops.
        Some(source) => render_with(&scan(source)?, context, state)?,
        None => render_with(&pieces[index + 1..end], context, state)?,
    };
    out.push_str(&rendered);
    Ok(end + 1)
}
