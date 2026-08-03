//! Call-path splitting: separating `ns::name(args)` into its three parts.
//!
//! Kept apart from [`crate::tmplmacro::call`] dispatch so the `self` aliases and the
//! parenthesis rules are each stated in exactly one place. `self::row`, `_self::row`, and
//! bare `row` are the same request: the macro defined by the template currently being
//! rendered. All three spellings appear across the reference views, so all three are
//! accepted rather than requiring a ported view to be rewritten.

/// Split a call body into its path text and the raw text between its parentheses.
///
/// # Arguments
///
/// * `body` — Trimmed hole body, such as `ui::badge(kind="new")`.
///
/// # Returns
///
/// `(path, arguments)`, both trimmed. The arguments are empty for `name()`.
///
/// # Errors
///
/// Returns an error when `(` is absent, or when no `)` follows it.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::call_path::split_call;
///
/// assert_eq!(split_call("ui::b(k=1)").unwrap(), ("ui::b", "k=1"));
/// assert_eq!(split_call("row()").unwrap(), ("row", ""));
/// assert!(split_call("ui::b").is_err());
/// ```
pub fn split_call(body: &str) -> Result<(&str, &str), String> {
    let body = body.trim();
    let open = body
        .find('(')
        .ok_or_else(|| format!("template: macro call `{body}` needs `(...)`"))?;
    let close = body
        .rfind(')')
        .filter(|close| *close > open)
        .ok_or_else(|| format!("template: unclosed argument list in `{body}`"))?;
    Ok((body[..open].trim(), body[open + 1..close].trim()))
}

/// Split a call path into its namespace and macro name.
///
/// # Arguments
///
/// * `path` — Identifier path such as `ui::badge`, `self::row`, or `row`.
///
/// # Returns
///
/// `(Some(namespace), name)` when the path names another template, or `(None, name)` for
/// the `self::`, `_self::`, and bare forms.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::call_path::split_path;
///
/// assert_eq!(split_path("ui::badge"), (Some("ui"), "badge"));
/// assert_eq!(split_path("_self::row"), (None, "row"));
/// assert_eq!(split_path("row"), (None, "row"));
/// ```
pub fn split_path(path: &str) -> (Option<&str>, &str) {
    match path.rsplit_once("::") {
        Some(("self" | "_self", name)) => (None, name),
        Some((namespace, name)) => (Some(namespace), name),
        None => (None, path),
    }
}
