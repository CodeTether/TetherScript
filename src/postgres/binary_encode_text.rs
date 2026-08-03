//! # Text-like parameter encoders: `text`, `json`, `jsonb`, `bytea`, `uuid`
//!
//! The body is simply the payload bytes; the field's length prefix, written by the
//! caller, is what delimits it. So there is no framing to get wrong here — the
//! subtleties are elsewhere:
//!
//! - **`jsonb` needs the leading version byte** (`1`) that `json` must not have.
//!   Omitting it makes the server reject the parameter; adding it to a `json`
//!   parameter stores a control character at the front of the document.
//! - **An empty string is a zero-length value, not NULL.** `Some(vec![])` versus
//!   `None` is the whole distinction, and it is preserved by construction here since
//!   NULL is filtered out before dispatch.
//! - **`bytea` takes bytes, not a string.** A [`Value::Str`] is *also* accepted and
//!   sent as its UTF-8 bytes, which is what a caller passing a string literal means.
//! - **`uuid` must be 16 bytes**, so the canonical string is parsed rather than sent
//!   as text. A malformed UUID is rejected by name.

use crate::value::Value;

use super::super::error::DecodeError;
use super::super::uuid as uuid_fmt;
use super::mismatch;

/// Encode a UTF-8 text parameter: `text`, `varchar`, `bpchar`, `name`, `xml`, `json`.
///
/// # Arguments
///
/// * `value` — a [`Value::Str`].
/// * `what` — the SQL type name, used only in the error message.
///
/// # Returns
///
/// The string's UTF-8 bytes; empty for an empty string, which is a present value.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for a non-string value. Numbers are not stringified:
/// binding `42` to a `text` column is usually a mistake, and the caller can convert
/// deliberately.
pub(super) fn utf8(value: &Value, what: &'static str) -> Result<Vec<u8>, DecodeError> {
    match value {
        Value::Str(text) => Ok(text.as_bytes().to_vec()),
        other => Err(mismatch(what, other)),
    }
}

/// Encode a `jsonb` parameter: the version byte `1`, then the document.
///
/// # Arguments
///
/// * `value` — a [`Value::Str`] holding a JSON document.
///
/// # Returns
///
/// `[1]` followed by the document's UTF-8 bytes.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for a non-string value.
pub(super) fn jsonb(value: &Value) -> Result<Vec<u8>, DecodeError> {
    let mut out = vec![1u8]; // jsonb binary version; json has no such byte
    out.extend_from_slice(&utf8(value, "jsonb")?);
    Ok(out)
}

/// Encode a `bytea` parameter: raw bytes, no framing, no charset.
///
/// # Arguments
///
/// * `value` — a [`Value::Bytes`], or a [`Value::Str`] sent as its UTF-8 bytes.
///
/// # Returns
///
/// The bytes verbatim; empty is a present, zero-length value.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for any other value kind.
pub(super) fn bytea(value: &Value) -> Result<Vec<u8>, DecodeError> {
    match value {
        Value::Bytes(bytes) => Ok(bytes.borrow().clone()),
        Value::Str(text) => Ok(text.as_bytes().to_vec()),
        other => Err(mismatch("bytea", other)),
    }
}

/// Encode a `uuid` parameter: exactly 16 bytes parsed from the canonical form.
///
/// # Arguments
///
/// * `value` — a [`Value::Str`], hyphenated or not, in either letter case.
///
/// # Returns
///
/// The 16 raw bytes in network order.
///
/// # Errors
///
/// [`DecodeError::BadValue`] for a non-string value, or a string that is not 32 hex
/// digits — reported by name rather than padded or truncated into something the
/// server would accept as a different UUID.
pub(super) fn uuid(value: &Value) -> Result<Vec<u8>, DecodeError> {
    let text = match value {
        Value::Str(text) => text,
        other => return Err(mismatch("uuid", other)),
    };
    match uuid_fmt::parse(text) {
        Some(bytes) => Ok(bytes.to_vec()),
        None => Err(DecodeError::BadValue {
            what: "uuid",
            detail: format!("{text:?} is not a 32-hex-digit UUID"),
        }),
    }
}
