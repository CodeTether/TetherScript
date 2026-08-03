//! Macro header parsing: the name and the parameter list.
//!
//! Split from [`super::template_macro`] so declaration syntax is one concern and
//! collection is another. A default value is kept as its raw literal text rather than
//! being converted eagerly: conversion belongs to
//! [`super::template_filter_arg::literal_of`], which already decides int vs float vs
//! bool vs str for filter arguments, and reusing it keeps `size="sm"` and
//! `default(value="sm")` behaving identically.

use super::template_filter_split::split_outside_quotes;

/// One declared parameter: its name and optional default literal.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Param<'a> {
    /// Parameter name as written.
    pub name: &'a str,
    /// Raw default literal text, or `None` when the parameter is required.
    pub default: Option<&'a str>,
}

/// Parse `macro <name>(<params>)`.
///
/// # Arguments
///
/// * `body` — Trimmed tag body beginning with the `macro` keyword.
///
/// # Returns
///
/// The macro name and its parameters in source order. An empty list for `macro n()`.
///
/// # Errors
///
/// Returns an error when the keyword is absent, the name is missing, the parameter
/// list is absent or unclosed, or a parameter name is empty. An unparenthesized
/// header is rejected rather than assumed to take no parameters, because
/// `{% macro row %}` is a typo, not a shorthand.
///
/// # Examples
///
/// ```text
/// \{% macro badge(kind, size="sm") %\}  →  ("badge", [kind, size="sm"])
/// ```
pub(super) fn parse_header(body: &str) -> Result<(&str, Vec<Param<'_>>), String> {
    let rest = body
        .strip_prefix("macro")
        .ok_or_else(|| format!("template: `{body}` is not a `macro` header"))?
        .trim();
    let open = rest
        .find('(')
        .ok_or_else(|| format!("template: macro `{rest}` needs a `(...)` parameter list"))?;
    let close = rest
        .rfind(')')
        .filter(|close| *close > open)
        .ok_or_else(|| format!("template: unclosed parameter list in `{body}`"))?;
    let name = rest[..open].trim();
    if name.is_empty() {
        return Err(format!("template: `{body}` needs a macro name"));
    }
    Ok((name, list_of(rest[open + 1..close].trim(), body)?))
}

/// Parse the comma-separated interior of a parameter list.
fn list_of<'a>(inside: &'a str, body: &str) -> Result<Vec<Param<'a>>, String> {
    if inside.is_empty() {
        return Ok(Vec::new());
    }
    // Quote-aware, since a default such as `sep=", "` legally contains a comma.
    split_outside_quotes(inside, ',')
        .into_iter()
        .map(|part| one(part.trim(), body))
        .collect()
}

/// Parse a single `name` or `name=<literal>` parameter.
fn one<'a>(text: &'a str, body: &str) -> Result<Param<'a>, String> {
    let (name, default) = match text.split_once('=') {
        Some((name, literal)) => (name.trim(), Some(literal.trim())),
        None => (text, None),
    };
    if name.is_empty() {
        return Err(format!("template: empty parameter name in `{body}`"));
    }
    Ok(Param { name, default })
}
