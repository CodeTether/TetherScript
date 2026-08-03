//! # Text-like decoders: `text`, `json`, `jsonb`, `bytea`, `uuid`
//!
//! These types have no fixed length: the body runs to the end of the field, whose
//! extent the 4-byte length prefix already established. So there is nothing to
//! bounds-check *within* the body — the checking happened when the field was cut
//! out of the `DataRow` — except for `uuid`, which is exactly 16 bytes, and
//! `jsonb`, which has a one-byte header.
//!
//! ## `jsonb` carries a version byte that `json` does not
//!
//! Binary `json` is the document bytes verbatim. Binary `jsonb` is a **1-byte
//! version prefix** (currently always `1`) followed by the document. Feeding a
//! `jsonb` body to a `json` decoder therefore yields a string starting with a
//! control character, which will not parse as JSON — a confusing failure a long
//! way from its cause. [`jsonb`] strips the byte and rejects an unknown version
//! rather than guessing.
//!
//! ## UTF-8 is validated, not lossily replaced
//!
//! `String::from_utf8_lossy` would substitute U+FFFD and hand back corrupted text
//! that looks fine until it is written back. Invalid bytes are a named
//! [`DecodeError::BadUtf8`] instead. `bytea` is exempt: it is *defined* as
//! arbitrary bytes, so it becomes [`Value::Bytes`] with no charset assumption.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

use super::super::error::DecodeError;
use super::super::read::Reader;
use super::super::uuid as uuid_fmt;

/// Decode a UTF-8 text body: `text`, `varchar`, `bpchar`, `name`, `xml`, `json`.
///
/// # Arguments
///
/// * `body` — the whole field body; an empty slice is a legitimate empty string,
///   distinct from SQL NULL, which never reaches this function.
/// * `what` — field name for the error message.
///
/// # Returns
///
/// [`Value::Str`].
///
/// # Errors
///
/// [`DecodeError::BadUtf8`] when the bytes are not valid UTF-8.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::{decode_field, oid};
/// use tetherscript::value::Value;
///
/// // An empty body is the empty string, not nil.
/// assert_eq!(decode_field(oid::TEXT, b"").unwrap(), Value::Str(std::rc::Rc::new(String::new())));
/// ```
pub(super) fn utf8(body: &[u8], what: &'static str) -> Result<Value, DecodeError> {
    let text = std::str::from_utf8(body).map_err(|_| DecodeError::BadUtf8 { what })?;
    Ok(Value::Str(Rc::new(text.to_string())))
}

/// Decode a `jsonb` body: a version byte, then the UTF-8 document.
///
/// # Arguments
///
/// * `body` — the whole field body.
///
/// # Returns
///
/// [`Value::Str`] holding the document, version byte removed.
///
/// # Errors
///
/// [`DecodeError::Truncated`] on an empty body, [`DecodeError::BadValue`] for a
/// version other than `1`, and [`DecodeError::BadUtf8`] for invalid UTF-8.
pub(super) fn jsonb(body: &[u8]) -> Result<Value, DecodeError> {
    let mut reader = Reader::new(body);
    let version = reader.take("jsonb version", 1)?[0];
    if version != 1 {
        return Err(DecodeError::BadValue {
            what: "jsonb",
            detail: format!("unsupported jsonb version byte {version}, expected 1"),
        });
    }
    utf8(reader.rest(), "jsonb")
}

/// Decode a `bytea` body: arbitrary bytes with no framing and no charset.
///
/// # Arguments
///
/// * `body` — the whole field body.
///
/// # Returns
///
/// [`Value::Bytes`]. Never fails: every byte sequence, including empty, is valid.
pub(super) fn bytea(body: &[u8]) -> Value {
    Value::Bytes(Rc::new(RefCell::new(body.to_vec())))
}

/// Decode a `uuid`: exactly 16 raw bytes, rendered canonically.
///
/// # Arguments
///
/// * `body` — exactly 16 bytes.
///
/// # Returns
///
/// [`Value::Str`] in lowercase `8-4-4-4-12` form.
///
/// # Errors
///
/// [`DecodeError::Truncated`] or [`DecodeError::Overlong`] when the body is not
/// exactly 16 bytes.
pub(super) fn uuid(body: &[u8]) -> Result<Value, DecodeError> {
    let mut reader = Reader::new(body);
    let bytes = reader.take("uuid", 16)?;
    reader.finish("uuid")?;
    Ok(Value::Str(Rc::new(uuid_fmt::hyphenate(bytes))))
}
