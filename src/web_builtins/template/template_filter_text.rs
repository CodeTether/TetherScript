//! Text-shortening filter.

use std::rc::Rc;

use crate::value::Value;

/// Truncate a string to `length=N`, appending `end=".."` when it was shortened.
///
/// Counts characters rather than bytes, so truncation cannot split a multi-byte character
/// and produce invalid UTF-8 in the middle of a page.
///
/// # Errors
///
/// Returns an error for a non-string value or a malformed argument.
pub(super) fn truncate(value: &Value, argument: &str) -> Result<Value, String> {
    let Value::Str(text) = value else {
        return Err(format!(
            "template: `truncate` needs a str, got {}",
            value.type_name()
        ));
    };
    let (length, suffix) = super::template_filter_truncate_args::parse(argument)?;
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= length {
        return Ok(Value::Str(Rc::clone(text)));
    }
    let mut out: String = chars[..length].iter().collect();
    out.push_str(&suffix);
    Ok(Value::Str(Rc::new(out)))
}
