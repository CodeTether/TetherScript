//! Tag scanning for the Tera-compatible subset.
//!
//! Splits a template into literal text, `{{ expression }}` holes, and
//! `{% statement %}` tags. Keeping the scan separate from evaluation means the
//! block structure is validated once, before any value is looked up.

use super::template_delimit::{delimited, next_delimiter};

/// One lexical piece of a template.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Piece<'a> {
    /// Literal text, emitted verbatim.
    Text(&'a str),
    /// `{{ name }}`, HTML-escaped on output.
    Escaped(&'a str),
    /// `{{{ name }}}`, emitted raw.
    Raw(&'a str),
    /// `{% ... %}`, with the trimmed statement body.
    Tag(&'a str),
}

/// Split `template` into pieces in source order.
///
/// # Errors
///
/// Returns an error for an unclosed delimiter or an empty expression, naming the
/// delimiter so the offending construct is findable.
pub(super) fn scan(template: &str) -> Result<Vec<Piece<'_>>, String> {
    let mut pieces = Vec::new();
    let mut rest = template;
    while let Some(start) = next_delimiter(rest) {
        if start > 0 {
            pieces.push(Piece::Text(&rest[..start]));
        }
        let after = &rest[start..];
        let (piece, consumed) = one(after)?;
        pieces.push(piece);
        rest = &after[consumed..];
    }
    if !rest.is_empty() {
        pieces.push(Piece::Text(rest));
    }
    Ok(pieces)
}

/// Read one delimited piece from the start of `after`.
fn one(after: &str) -> Result<(Piece<'_>, usize), String> {
    if after.starts_with("{%") {
        let (body, consumed) = delimited(after, "{%", "%}")?;
        return Ok((Piece::Tag(body), consumed));
    }
    // The triple form must be tested first: it shares its opening characters with
    // the double form, which would otherwise match and leave a stray brace.
    if after.starts_with("{{{") {
        let (body, consumed) = delimited(after, "{{{", "}}}")?;
        return Ok((Piece::Raw(body), consumed));
    }
    let (body, consumed) = delimited(after, "{{", "}}")?;
    Ok((Piece::Escaped(body), consumed))
}
