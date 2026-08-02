//! Filter expression parsing for `{{ value | filter | filter(arg=x) }}`.
//!
//! Splitting is separate from application so a malformed pipeline is reported before
//! any lookup runs, and so the split can be reused by both escaped and raw holes.

/// One filter in a pipeline: its name and optional `key=value` argument.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Filter<'a> {
    /// Filter name, such as `default` or `safe`.
    pub name: &'a str,
    /// Raw argument text inside the parentheses, empty when there is none.
    pub argument: &'a str,
}

/// Split a hole body into its value key and filter pipeline.
///
/// # Arguments
///
/// * `body` — The trimmed text inside `{{ }}`.
///
/// # Returns
///
/// The key, and the filters in application order.
///
/// # Errors
///
/// Returns an error for an empty key or an empty filter name, since `{{ x | }}` is
/// always a typo rather than a shorthand.
pub(super) fn split(body: &str) -> Result<(&str, Vec<Filter<'_>>), String> {
    let mut parts = body.split('|');
    let key = parts.next().unwrap_or("").trim();
    if key.is_empty() {
        return Err(format!("template: no value before `|` in `{body}`"));
    }
    let mut filters = Vec::new();
    for part in parts {
        filters.push(one(part.trim(), body)?);
    }
    Ok((key, filters))
}

/// Parse a single `name` or `name(argument)` filter.
fn one<'a>(text: &'a str, body: &str) -> Result<Filter<'a>, String> {
    let Some(open) = text.find('(') else {
        if text.is_empty() {
            return Err(format!("template: empty filter name in `{body}`"));
        }
        return Ok(Filter {
            name: text,
            argument: "",
        });
    };
    let close = text
        .rfind(')')
        .ok_or_else(|| format!("template: unclosed filter argument in `{body}`"))?;
    Ok(Filter {
        name: text[..open].trim(),
        argument: text[open + 1..close].trim(),
    })
}
