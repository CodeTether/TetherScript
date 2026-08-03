//! Path splitting for a macro call: `ns::name(args)` into its three parts.
//!
//! One concern, kept apart from dispatch so the `self` aliases and the parenthesis rules
//! are each stated in exactly one place. `self::row`, `_self::row`, and bare `row` are the
//! same request: the macro defined by the template currently being rendered.

/// Split a call body into its path and its raw argument text.
///
/// # Arguments
///
/// * `body` — Trimmed hole body, such as `ui::badge(kind="new")`.
///
/// # Returns
///
/// The path text and the text between the parentheses, both trimmed.
///
/// # Errors
///
/// Returns an error when the parentheses are missing or unbalanced.
///
/// # Examples
///
/// ```text
/// split_call("ui::badge(kind=\"new\")")  →  ("ui::badge", "kind=\"new\"")
/// ```
pub(super) fn split_call(body: &str) -> Result<(&str, &str), String> {
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
/// `(Some(namespace), name)` when the path names another template, or `(None, name)` when
/// it refers to the current one.
///
/// # Errors
///
/// None; an unresolvable name is reported by the caller, which knows what is in scope.
///
/// # Examples
///
/// ```text
/// split_path("ui::badge")  →  (Some("ui"), "badge")
/// split_path("self::row")  →  (None, "row")
/// split_path("row")        →  (None, "row")
/// ```
pub(super) fn split_path(path: &str) -> (Option<&str>, &str) {
    match path.rsplit_once("::") {
        // `self`/`_self` are Tera's spelling for "this template"; both are accepted so a
        // ported view does not have to be rewritten.
        Some(("self" | "_self", name)) => (None, name),
        Some((namespace, name)) => (Some(namespace), name),
        None => (None, path),
    }
}
