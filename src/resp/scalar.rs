//! # Parsing the scalar payload of a line
//!
//! Every RESP header and every non-bulk scalar is a single CRLF-terminated line,
//! so all of RESP's number and text parsing lives here. It is separate from the
//! framing in [`super::cursor`] because the two fail for different reasons:
//! framing runs out of bytes, whereas this module rejects bytes that have all
//! already arrived and are simply not what the protocol allows.
//!
//! Nothing here ever returns [`DecodeError::Incomplete`]: once a line is in hand
//! it is complete by definition.

use super::error::DecodeError;

/// Interpret a line as UTF-8 text.
///
/// # Arguments
///
/// * `line` — line contents, CRLF already stripped.
/// * `what` — the RESP type being decoded, used in the error message.
///
/// # Errors
///
/// [`DecodeError::Malformed`] when the line is not valid UTF-8. Requiring UTF-8
/// is safe here, unlike for bulk payloads: `+`, `-`, `:`, `,`, `#` and `(` are
/// specified as text, and only the length-prefixed types are binary safe.
pub(super) fn text(line: &[u8], what: &str) -> Result<String, DecodeError> {
    String::from_utf8(line.to_vec())
        .map_err(|_| DecodeError::malformed(format!("{what} payload is not valid UTF-8")))
}

/// Interpret a line as a signed 64-bit decimal integer.
///
/// Used both for `:` integer replies and for the length and element-count headers
/// of `$`, `=`, `*`, `~`, `>` and `%`. Signed rather than unsigned because `-1`
/// is the null header, and anything wider than an `i64` is rejected here rather
/// than saturating into a plausible-looking length.
///
/// # Errors
///
/// [`DecodeError::Malformed`] when the line is not a decimal integer that fits an
/// `i64`, naming the offending text.
pub(super) fn integer(line: &[u8], what: &str) -> Result<i64, DecodeError> {
    let raw = text(line, what)?;
    raw.parse::<i64>()
        .map_err(|_| DecodeError::malformed(format!("{what} has invalid integer {raw:?}")))
}

/// Interpret a line as a RESP3 double.
///
/// # Errors
///
/// [`DecodeError::Malformed`] when the text is neither one of the special forms
/// `inf`, `-inf`, `nan` nor a value Rust's float parser accepts.
pub(super) fn double(line: &[u8]) -> Result<f64, DecodeError> {
    let raw = text(line, "double")?;
    match raw.as_str() {
        "inf" | "+inf" => Ok(f64::INFINITY),
        "-inf" => Ok(f64::NEG_INFINITY),
        "nan" | "-nan" => Ok(f64::NAN),
        other => other
            .parse::<f64>()
            .map_err(|_| DecodeError::malformed(format!("double has invalid value {other:?}"))),
    }
}

/// Interpret a line as a RESP3 boolean: exactly `t` or `f`.
///
/// # Errors
///
/// [`DecodeError::Malformed`] for anything else, including `true`/`false` and the
/// empty line.
pub(super) fn boolean(line: &[u8]) -> Result<bool, DecodeError> {
    match line {
        b"t" => Ok(true),
        b"f" => Ok(false),
        other => Err(DecodeError::malformed(format!(
            "boolean must be `t` or `f`, found {other:?}"
        ))),
    }
}

/// Validate a line as a RESP3 big number: an optional sign then decimal digits.
///
/// The value is kept as text because the entire point of the type is that it does
/// not fit an `i64`; converting it here would lose exactly the precision the
/// server took the trouble to send.
///
/// # Errors
///
/// [`DecodeError::Malformed`] when the line is empty, is only a sign, or contains
/// a non-digit.
pub(super) fn big_number(line: &[u8]) -> Result<String, DecodeError> {
    let raw = text(line, "big number")?;
    let digits = raw.strip_prefix('-').or_else(|| raw.strip_prefix('+'));
    let digits = digits.unwrap_or(raw.as_str());
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(DecodeError::malformed(format!(
            "big number has invalid value {raw:?}"
        )));
    }
    Ok(raw)
}
