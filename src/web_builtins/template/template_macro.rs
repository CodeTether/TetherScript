//! `{% macro %}` definition collection and definition-site handling.
//!
//! A macro is a named, parameterized fragment. Definitions are *collected* from a
//! template — in the same spirit as [`super::template_blocks::collect`] gathers blocks —
//! and are rendered only when called. Reaching a `{% macro %}` tag while rendering
//! therefore emits nothing at all: the definition is a declaration, not output, so a
//! macro that is defined and never called contributes no text.
//!
//! # Semantics this file fixes
//!
//! * **Definitions are hoisted.** Every `macro` tag in the piece list is collected,
//!   including ones nested inside `{% if %}` or `{% for %}`, because a definition is not
//!   conditional in Tera either.
//! * **Last definition wins.** Two macros of one name resolve to the later, matching how
//!   [`super::template_blocks`] keys blocks.
//! * **Both end forms are accepted.** `{% endmacro %}` and `{% endmacro name %}` are
//!   equally valid, exactly as `endblock` accepts both. The trailing name is a reader's
//!   annotation, so it is not checked against the opener.
//!
//! # Examples
//!
//! ```text
//! let t = map()
//! t["ui"] = "\{% macro badge(kind) %\}\{\{ kind \}\}\{% endmacro badge %\}"
//! println(template_render_inherited("\{\{ ui::badge(kind=\"new\") \}\}", map(), t).unwrap())
//! ```
//!
//! # Integration note
//!
//! [`super::template_delimit::matching_end`] must count `macro`/`endmacro` alongside
//! `if`/`for`/`block`, or a macro inside an `if` lets the `if`'s search stop early.

use std::collections::HashMap;

use super::template_block::Render;
use super::template_delimit::matching_end;
use super::template_macro_param::{parse_header, Param};
use super::template_scan::Piece;
use crate::value::Value;

/// One collected macro definition: its header, parameters, and unrendered body.
pub(super) struct Macro<'a> {
    /// Original tag body, such as `macro badge(kind, size="sm")`. Retained so
    /// [`super::template_macro_body`] can restate the definition verbatim, which is what
    /// keeps a macro visible to itself and to its siblings during a call.
    pub header: &'a str,
    /// Declared parameters, in source order, each with an optional default.
    pub params: Vec<Param<'a>>,
    /// Body pieces between the `macro` tag and its `endmacro`.
    pub body: Vec<Piece<'a>>,
}

/// Macros defined by a template, keyed by name.
pub(super) type Macros<'a> = HashMap<String, Macro<'a>>;

/// Collect every macro a template defines.
///
/// # Arguments
///
/// * `pieces` — Scanned pieces of one template.
///
/// # Returns
///
/// A name-to-definition map, empty when the template defines no macros.
///
/// # Errors
///
/// Returns an error for a malformed header or an unbalanced `endmacro`.
pub(super) fn collect<'a>(pieces: &[Piece<'a>]) -> Result<Macros<'a>, String> {
    let mut macros = HashMap::new();
    for (index, piece) in pieces.iter().enumerate() {
        // Dereferenced rather than used through the `&&str` the pattern binds, so the
        // parsed parameters borrow the template text and not this loop iteration.
        let Piece::Tag(tag) = piece else { continue };
        let header: &'a str = tag;
        if header.split_whitespace().next() != Some("macro") {
            continue;
        }
        let (name, params) = parse_header(header)?;
        let end = matching_end(pieces, index)?;
        let body = pieces[index + 1..end].to_vec();
        let entry = Macro {
            header,
            params,
            body,
        };
        macros.insert(name.to_string(), entry);
    }
    Ok(macros)
}

/// Skip a `{% macro %}` definition, returning the index just past its `endmacro`.
///
/// # Arguments
///
/// * `pieces` — The enclosing piece list.
/// * `index` — Index of the `macro` tag.
/// * `body` — Trimmed tag body, such as `macro badge(kind, size="sm")`.
/// * `_context`, `_state`, `_out` — Unused: a definition reads nothing and emits nothing.
///   They exist so the signature matches the other tag handlers.
///
/// # Returns
///
/// The index following the matching `endmacro`.
///
/// # Errors
///
/// Returns an error for a malformed header or an unbalanced `endmacro`. The header is
/// parsed although unused here, so a typo in a macro that is never called is still
/// reported rather than lying in wait.
pub(super) fn run(
    pieces: &[Piece<'_>],
    index: usize,
    body: &str,
    _context: &Value,
    _state: &Render<'_>,
    _out: &mut String,
) -> Result<usize, String> {
    parse_header(body)?;
    Ok(matching_end(pieces, index)? + 1)
}
