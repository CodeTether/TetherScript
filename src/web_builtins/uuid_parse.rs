//! UUID parsing and validation.
//!
//! Accepts only the canonical 8-4-4-4-12 lowercase-or-uppercase hex form.
//! Every rejection names the specific problem, because "invalid UUID" gives a
//! caller nothing to act on.

use crate::value::Value;

/// Hyphen offsets in the canonical 36-character form.
const HYPHENS: [usize; 4] = [8, 13, 18, 23];

/// Parse a `Value` argument, returning the normalized lowercase UUID.
///
/// # Arguments
///
/// * `value` — The script argument; must be a str.
///
/// # Returns
///
/// The UUID lowercased, so callers can compare results directly.
///
/// # Errors
///
/// Returns an error naming the problem: a non-str argument, the wrong length, a
/// hyphen in the wrong position, or the offending non-hex character.
pub(super) fn parse_arg(value: &Value) -> Result<Value, String> {
    let text = match value {
        Value::Str(text) => (**text).clone(),
        other => {
            return Err(format!(
                "uuid_parse: text must be str, got {}",
                other.type_name()
            ))
        }
    };
    check(&text)?;
    Ok(Value::Str(std::rc::Rc::new(text.to_lowercase())))
}

/// Report whether `value` is a canonical UUID, without allocating an error.
pub(super) fn is_valid_arg(value: &Value) -> bool {
    match value {
        Value::Str(text) => check(text).is_ok(),
        _ => false,
    }
}

/// Validate the canonical form, naming the first problem found.
fn check(text: &str) -> Result<(), String> {
    if text.len() != 36 {
        return Err(format!(
            "uuid_parse: expected 36 characters, got {}",
            text.len()
        ));
    }
    for (index, ch) in text.char_indices() {
        let expect_hyphen = HYPHENS.contains(&index);
        if expect_hyphen && ch != '-' {
            return Err(format!(
                "uuid_parse: expected `-` at position {index}, got `{ch}`"
            ));
        }
        if !expect_hyphen && !ch.is_ascii_hexdigit() {
            return Err(format!(
                "uuid_parse: invalid character `{ch}` at position {index}"
            ));
        }
    }
    Ok(())
}
