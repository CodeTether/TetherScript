//! Quote-aware splitting of comma-separated macro argument and parameter lists.
//!
//! A naive `text.split(',')` breaks on real reference-application call sites such as
//! `sep=", "` or `label="Book (today)"`: the comma and the parentheses inside a quoted
//! literal are **data**, not structure. Splitting naively truncates the literal at the
//! first inner comma and silently renders a partial value — the class of quiet breakage
//! this component reports instead. The engine already learned this lesson in
//! `template_filter_split::split_outside_quotes`; that function is `pub(super)` and so
//! unreachable from here, and this is a deliberate, behaviour-identical sibling.
//!
//! Parenthesis depth is tracked as well, so a nested call used as an argument value
//! (`cfg=other::inner(a=1)`) is not split at its inner comma either.

/// Split `text` on `separator`, ignoring separators inside quotes or parentheses.
///
/// # Arguments
///
/// * `text` — Source to split, typically the interior of a call's parentheses.
/// * `separator` — Structural byte to split on, normally `,`.
///
/// # Returns
///
/// The segments in source order, quotes preserved so a caller can still unquote
/// literals. Always at least one segment, so `""` yields `[""]`.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::split::split_outside_quotes;
///
/// let parts = split_outside_quotes(r#"a="x, y", b=1"#, ',');
/// assert_eq!(parts, vec![r#"a="x, y""#, " b=1"]);
///
/// let parens = split_outside_quotes(r#"a="f(1,2)", b=2"#, ',');
/// assert_eq!(parens, vec![r#"a="f(1,2)""#, " b=2"]);
/// ```
pub fn split_outside_quotes(text: &str, separator: char) -> Vec<&str> {
    let (mut parts, mut quote, mut depth, mut start) = (Vec::new(), None, 0usize, 0usize);
    for (index, ch) in text.char_indices() {
        match quote {
            Some(open) if ch == open => quote = None,
            Some(_) => {}
            None if ch == '"' || ch == '\'' => quote = Some(ch),
            None if ch == '(' => depth += 1,
            None if ch == ')' => depth = depth.saturating_sub(1),
            None if ch == separator && depth == 0 => {
                parts.push(&text[start..index]);
                start = index + ch.len_utf8();
            }
            None => {}
        }
    }
    parts.push(&text[start..]);
    parts
}
