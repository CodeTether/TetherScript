//! `{% extends %}` detection.
//!
//! A child template names a parent and supplies block bodies. Rendering therefore
//! runs the *parent*, substituting the child's blocks wherever the parent declares
//! one of the same name. That inversion is why blocks are collected up front.

use super::template_scan::Piece;

/// The parent named by a leading `{% extends %}`, if any.
///
/// # Returns
///
/// `Some(name)` when the first non-whitespace piece is an `extends` tag.
///
/// # Errors
///
/// Returns an error when the quoted name is missing or unterminated.
pub(super) fn parent_of<'a>(pieces: &[Piece<'a>]) -> Result<Option<&'a str>, String> {
    for piece in pieces {
        match piece {
            // Whitespace before `extends` is normal formatting, not content.
            Piece::Text(text) if text.trim().is_empty() => continue,
            Piece::Tag(body) if body.split_whitespace().next() == Some("extends") => {
                return unquote(body).map(Some);
            }
            _ => return Ok(None),
        }
    }
    Ok(None)
}

/// Extract the quoted template name from `extends '<name>'`.
///
/// Both quote styles are accepted because the reference views use single quotes,
/// which Tera permits.
///
/// # Errors
///
/// Returns an error when no quoted name follows the keyword.
pub(super) fn unquote(body: &str) -> Result<&str, String> {
    let rest = body
        .split_once(char::is_whitespace)
        .map(|(_, rest)| rest.trim())
        .unwrap_or("");
    let Some(quote) = rest.chars().next().filter(|c| *c == '"' || *c == '\'') else {
        return Err(format!(
            "template: `{body}` needs a quoted template name, e.g. extends \"base.html\""
        ));
    };
    let inner = &rest[quote.len_utf8()..];
    let end = inner
        .find(quote)
        .ok_or_else(|| format!("template: unterminated name in `{body}`"))?;
    Ok(&inner[..end])
}
