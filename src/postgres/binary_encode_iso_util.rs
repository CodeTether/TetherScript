//! # Shared helpers for ISO-8601 field parsing
//!
//! One error constructor and one numeric field parser, shared by the date and time
//! splitters so both produce identically shaped messages. Quoting the *whole* input
//! rather than just the failing field is deliberate: `"13"` out of context tells an
//! operator nothing, while `cannot parse "2024-13-01"` points straight at the value.

use super::super::super::super::error::DecodeError;

/// Parse one numeric field, naming the whole input on failure.
///
/// # Arguments
///
/// * `field` — the single field's text.
/// * `what` — the SQL type name for the error.
/// * `whole` — the complete input, quoted in the message.
///
/// # Returns
///
/// The parsed value.
///
/// # Errors
///
/// [`DecodeError::BadValue`] when `field` is not a valid `T`.
pub(super) fn number<T: std::str::FromStr>(
    field: &str,
    what: &'static str,
    whole: &str,
) -> Result<T, DecodeError> {
    field
        .parse()
        .map_err(|_| bad(what, whole, &format!("{field:?} is not a number")))
}

/// Build a named parse error, quoting the offending input.
///
/// # Arguments
///
/// * `what` — the SQL type name.
/// * `text` — the input that could not be parsed.
/// * `why` — the specific rule that was violated.
///
/// # Returns
///
/// A [`DecodeError::BadValue`] whose message names the value and the rule.
pub(super) fn bad(what: &'static str, text: &str, why: &str) -> DecodeError {
    DecodeError::BadValue {
        what,
        detail: format!("cannot parse {text:?}: {why}"),
    }
}
