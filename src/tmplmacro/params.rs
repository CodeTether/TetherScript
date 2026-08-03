//! Macro header parsing: `macro <name>(<params>)`.
//!
//! Declaration syntax is one concern, kept apart from collection. The parameter-list
//! grammar lives in [`crate::tmplmacro::params_item`] and name validation in
//! [`crate::tmplmacro::params_name`]; this file only locates the three parts of a header.
//!
//! A default value is retained as its raw literal text and converted lazily by
//! [`crate::tmplmacro::literal::literal_of`], so a header default and a filter argument
//! agree on what `"sm"` means.

use crate::tmplmacro::params_item::{parse_params, Param};
use crate::tmplmacro::params_name::reject_bad_name;

/// Parse a `macro name(a, b="x")` header.
///
/// # Arguments
///
/// * `body` — Trimmed tag body beginning with the `macro` keyword.
///
/// # Returns
///
/// The macro name and its parameters in source order; an empty list for `macro n()`.
///
/// # Errors
///
/// Returns an error when the keyword is not `macro`, the parentheses are missing or
/// unbalanced, the name is not an identifier, or a parameter name is empty.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::params::parse_header;
///
/// let (name, params) = parse_header(r#"macro badge(kind, size="sm")"#).unwrap();
/// assert_eq!(name, "badge");
/// assert_eq!(params.len(), 2);
/// assert_eq!(params[1].default.as_deref(), Some("\"sm\""));
/// assert!(parse_header("block x()").is_err());
/// ```
pub fn parse_header(body: &str) -> Result<(String, Vec<Param>), String> {
    let rest = body
        .strip_prefix("macro")
        .ok_or_else(|| format!("template: `{body}` is not a `macro` header"))?;
    let open = rest
        .find('(')
        .ok_or_else(|| format!("template: macro header `{body}` needs `(...)`"))?;
    let close = rest
        .rfind(')')
        .filter(|c| *c > open)
        .ok_or_else(|| format!("template: unclosed parameter list in `{body}`"))?;
    let name = rest[..open].trim();
    reject_bad_name(name, body)?;
    Ok((name.to_string(), parse_params(rest[open + 1..close].trim())?))
}
