//! `{% block name %}` extraction.
//!
//! A template's blocks are collected before rendering so inheritance can override
//! them by name. Collection is separate from rendering because a child template's
//! blocks are needed while rendering its *parent*, not itself.

use std::collections::HashMap;

use super::template_bounds::matching_end;
use super::template_scan::Piece;

/// Blocks defined by a template, keyed by name.
///
/// Values are the piece ranges between `{% block %}` and its `{% endblock %}`.
pub(super) type Blocks<'a> = HashMap<String, Vec<Piece<'a>>>;

/// Collect every top-level and nested block a template defines.
///
/// Nested blocks are collected too, because Tera allows a child to override an
/// inner block without restating its enclosing one.
///
/// # Errors
///
/// Returns an error for an unnamed block or an unbalanced `endblock`.
pub(super) fn collect<'a>(pieces: &[Piece<'a>]) -> Result<Blocks<'a>, String> {
    let mut blocks = HashMap::new();
    for (index, piece) in pieces.iter().enumerate() {
        let Piece::Tag(body) = piece else { continue };
        if !is_block_open(body) {
            continue;
        }
        let name = name_of(body)?;
        let (end, _) = matching_end(pieces, index)?;
        blocks.insert(name.to_string(), pieces[index + 1..end].to_vec());
    }
    Ok(blocks)
}

/// Whether a tag body opens a block.
pub(super) fn is_block_open(body: &str) -> bool {
    body.split_whitespace().next() == Some("block")
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
