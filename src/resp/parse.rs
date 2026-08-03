//! # Type-byte dispatch
//!
//! Every RESP value starts with one byte naming its type, so decoding is a single
//! dispatch followed by a type-specific body parse. This module owns that table
//! and nothing else; the bodies live in [`super::bulk`], [`super::aggregate`] and
//! [`super::scalar`].
//!
//! An unknown type byte is [`DecodeError::Malformed`], never
//! [`DecodeError::Incomplete`]: no number of further bytes can make it valid, and
//! guessing would desynchronise every reply after it.

use super::aggregate;
use super::bulk;
use super::cursor::Cursor;
use super::error::DecodeError;
use super::reply::Reply;
use super::scalar;

/// Decode one value at `depth` aggregates deep.
///
/// # Arguments
///
/// * `cursor` — position in the receive buffer; advanced only on success.
/// * `depth` — aggregates already entered; `0` at the top level.
///
/// # Errors
///
/// [`DecodeError::Incomplete`] when the value is a valid but unfinished prefix;
/// [`DecodeError::Malformed`] on an unknown type byte, a bound violation, or a
/// body that does not match its type.
pub(super) fn value(cursor: &mut Cursor<'_>, depth: usize) -> Result<Reply, DecodeError> {
    match cursor.byte()? {
        b'+' => simple(cursor),
        b'-' => Ok(Reply::Error(scalar::text(cursor.line()?, "error")?)),
        b':' => Ok(Reply::Integer(scalar::integer(cursor.line()?, "integer")?)),
        b'$' => bulk::bulk(cursor),
        b'=' => bulk::verbatim(cursor),
        b',' => Ok(Reply::Double(scalar::double(cursor.line()?)?)),
        b'#' => Ok(Reply::Boolean(scalar::boolean(cursor.line()?)?)),
        b'(' => Ok(Reply::BigNumber(scalar::big_number(cursor.line()?)?)),
        b'_' => null(cursor),
        b'!' => blob_error(cursor),
        b'*' => aggregate::sequence(cursor, "array", Reply::Array, depth + 1),
        b'~' => aggregate::sequence(cursor, "set", Reply::Set, depth + 1),
        b'>' => aggregate::sequence(cursor, "push", Reply::Push, depth + 1),
        b'%' => aggregate::map(cursor, depth + 1),
        other => Err(DecodeError::malformed(format!(
            "unknown type byte {:?}",
            other as char
        ))),
    }
}

/// Decode a `+` simple string: the rest of the line, as text.
fn simple(cursor: &mut Cursor<'_>) -> Result<Reply, DecodeError> {
    let line = cursor.line()?;
    Ok(Reply::Simple(scalar::text(line, "simple string")?))
}

/// Decode the RESP3 null `_\r\n`, whose line must be empty.
fn null(cursor: &mut Cursor<'_>) -> Result<Reply, DecodeError> {
    match cursor.line()? {
        b"" => Ok(Reply::Nil),
        other => Err(DecodeError::malformed(format!(
            "null must be `_` alone, found trailing {other:?}"
        ))),
    }
}

/// Decode the RESP3 blob error `!21\r\nSYNTAX invalid syntax\r\n`.
///
/// Length-prefixed like a bulk string but semantically an error, so it becomes the
/// same [`Reply::Error`] a `-` line would, keeping the client's error handling in
/// one place.
fn blob_error(cursor: &mut Cursor<'_>) -> Result<Reply, DecodeError> {
    match bulk::bulk(cursor)? {
        Reply::Bulk(bytes) => Ok(Reply::Error(scalar::text(&bytes, "blob error")?)),
        _ => Err(DecodeError::malformed("blob error has no null form")),
    }
}
