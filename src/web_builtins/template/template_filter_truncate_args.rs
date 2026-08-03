//! `truncate` argument parsing.

use super::template_filter_arg::literal_of;
use crate::value::Value;

/// Parse `length=N` and optional `end="..."`, defaulting the suffix to an ellipsis.
///
/// # Errors
///
/// Returns an error for a malformed pair, an unknown key, or a missing `length`.
pub(super) fn parse(argument: &str) -> Result<(usize, String), String> {
    let mut length = None;
    let mut suffix = String::from("…");
    for part in super::template_filter_split::split_outside_quotes(argument, ',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (key, raw) = part
            .split_once('=')
            .ok_or_else(|| format!("template: `truncate({part})` must be `key=value`"))?;
        match key.trim() {
            "length" => length = Some(number(raw.trim())?),
            "end" => suffix = text_of(&literal_of(raw.trim())),
            other => return Err(format!("template: `truncate` has no argument `{other}`")),
        }
    }
    Ok((
        length.ok_or("template: `truncate` needs `length=N`")?,
        suffix,
    ))
}

/// Parse a non-negative length.
fn number(text: &str) -> Result<usize, String> {
    text.parse::<usize>().map_err(|_| {
        format!("template: `truncate` length must be a non-negative integer, got `{text}`")
    })
}

/// Plain text of a literal value.
fn text_of(value: &Value) -> String {
    match value {
        Value::Str(text) => (**text).clone(),
        Value::Nil => String::new(),
        other => format!("{other}"),
    }
}
