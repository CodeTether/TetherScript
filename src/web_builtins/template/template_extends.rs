//! `{% extends %}` and `{% include %}` name resolution.
//!
//! Both tags name another template, so finding the name and turning it into source is
//! one concern. Templates come from a caller-supplied map rather than the filesystem:
//! `template_*` are pure built-ins, so opening files inside them would bypass the `fs`
//! capability entirely.

use super::template_scan::Piece;
use crate::value::Value;

/// The parent named by a leading `{% extends %}`, if any.
///
/// # Errors
///
/// Returns an error when the quoted name is missing or unterminated.
pub(super) fn parent_of<'a>(pieces: &[Piece<'a>]) -> Result<Option<&'a str>, String> {
    for piece in pieces {
        match piece {
            // Whitespace before `extends` is formatting, not content.
            Piece::Text(text) if text.trim().is_empty() => continue,
            Piece::Tag(body) if body.split_whitespace().next() == Some("extends") => {
                return unquote(body).map(Some);
            }
            _ => return Ok(None),
        }
    }
    Ok(None)
}

/// Extract the quoted name from `extends '<name>'` or `include "<name>"`.
///
/// Both quote styles are accepted; the reference views use single quotes.
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
        return Err(format!("template: `{body}` needs a quoted template name"));
    };
    let inner = &rest[quote.len_utf8()..];
    let end = inner
        .find(quote)
        .ok_or_else(|| format!("template: unterminated name in `{body}`"))?;
    Ok(&inner[..end])
}

/// Look up a named template in the caller-supplied map.
///
/// # Errors
///
/// Returns an error naming the template when absent: a missing parent would otherwise
/// render as a blank page.
pub(super) fn source_of(templates: &Value, name: &str) -> Result<String, String> {
    let Value::Map(map) = templates else {
        return Err(format!(
            "template: `{name}` is referenced but no template map was supplied; \
             use template_render_inherited(template, context, templates)"
        ));
    };
    match map.borrow().get(name) {
        Some(Value::Str(text)) => Ok((**text).clone()),
        Some(other) => Err(format!(
            "template: template `{name}` must be str, got {}",
            other.type_name()
        )),
        None => Err(format!("template: unknown template `{name}`")),
    }
}
