//! Placeholder scanning and substitution.
//!
//! `{{{ raw }}}` is checked before `{{ escaped }}`, because the triple form
//! starts with the same two characters and would otherwise be read as an escaped
//! placeholder whose name begins with `{`.

use super::template_context::lookup;
use super::template_escape::escape;
use crate::value::Value;

/// Render `template` against `context`.
///
/// # Arguments
///
/// * `template` — Source text containing `{{ name }}` or `{{{ name }}}` holes.
/// * `context` — Map supplying values.
/// * `escaping` — When true, `{{ }}` output is HTML-escaped.
///
/// # Returns
///
/// The rendered text.
///
/// # Errors
///
/// Returns an error for an unclosed or empty placeholder, or any lookup failure.
pub(super) fn render(template: &str, context: &Value, escaping: bool) -> Result<String, String> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start..];
        let (name, consumed, raw) = placeholder(after)?;
        let value = lookup(context, name)?;
        if raw || !escaping {
            out.push_str(&value);
        } else {
            out.push_str(&escape(&value));
        }
        rest = &after[consumed..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Split one placeholder, returning its trimmed name, byte length, and rawness.
fn placeholder(after: &str) -> Result<(&str, usize, bool), String> {
    let triple = after.starts_with("{{{");
    let (open, close) = if triple { ("{{{", "}}}") } else { ("{{", "}}") };
    let body_start = open.len();
    let end = after[body_start..]
        .find(close)
        .ok_or_else(|| format!("template_render: unclosed `{open}` placeholder"))?;
    let name = after[body_start..body_start + end].trim();
    if name.is_empty() {
        return Err("template_render: empty placeholder".to_string());
    }
    Ok((name, body_start + end + close.len(), triple))
}
