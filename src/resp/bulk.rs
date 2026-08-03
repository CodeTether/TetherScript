//! # Length-prefixed payloads: `$` and `=`
//!
//! Bulk strings are the only RESP type whose payload is counted rather than
//! delimited, and that is exactly why they are binary safe. The count is a number
//! of **bytes**: a payload may contain `\r\n` in the middle, may be invalid
//! UTF-8, and may be a serialised image. So the decoder reads exactly `len`
//! bytes, then requires the trailing CRLF as a framing check, and never scans the
//! payload looking for a delimiter.
//!
//! `-1` is the null bulk string and decodes to [`Reply::Nil`], which is a
//! different value from a zero-length payload; see [`Reply::Nil`] for why the
//! codec refuses to blur the two.

use super::cursor::Cursor;
use super::error::DecodeError;
use super::limits::MAX_BULK_LEN;
use super::reply::Reply;
use super::scalar;

/// Decode the body of a `$` bulk string; the type byte is already consumed.
///
/// # Errors
///
/// [`DecodeError::Incomplete`] when the announced payload or its CRLF has not
/// fully arrived; [`DecodeError::Malformed`] for a length below `-1`, a length
/// above [`MAX_BULK_LEN`], or a payload not followed by CRLF.
pub(super) fn bulk(cursor: &mut Cursor<'_>) -> Result<Reply, DecodeError> {
    match payload(cursor, "bulk string")? {
        None => Ok(Reply::Nil),
        Some(bytes) => Ok(Reply::Bulk(bytes.to_vec())),
    }
}

/// Decode the body of a `=` verbatim string; the type byte is already consumed.
///
/// The payload is `xxx:` — a three-byte format hint and a colon — then the text.
/// The hint is split off into the `format` field of [`Reply::Verbatim`]; the
/// remainder stays bytes, since the length is still a byte count.
///
/// # Errors
///
/// As [`bulk`], plus [`DecodeError::Malformed`] when the payload is shorter than
/// four bytes, lacks the colon, or is the null form `=-1`, which RESP3 does not
/// define.
pub(super) fn verbatim(cursor: &mut Cursor<'_>) -> Result<Reply, DecodeError> {
    let bytes = payload(cursor, "verbatim string")?
        .ok_or_else(|| DecodeError::malformed("verbatim string has no null form"))?;
    if bytes.len() < 4 || bytes[3] != b':' {
        return Err(DecodeError::malformed(
            "verbatim string must begin with a three-byte format hint and `:`",
        ));
    }
    Ok(Reply::Verbatim {
        format: scalar::text(&bytes[..3], "verbatim format")?,
        text: bytes[4..].to_vec(),
    })
}

/// Read a length header, then that many bytes plus the terminating CRLF.
///
/// # Returns
///
/// `Ok(None)` for the null length `-1`, otherwise `Ok(Some(payload))` borrowed
/// from the caller's buffer.
fn payload<'a>(cursor: &mut Cursor<'a>, what: &str) -> Result<Option<&'a [u8]>, DecodeError> {
    let len = scalar::integer(cursor.line()?, what)?;
    if len == -1 {
        return Ok(None);
    }
    if len < -1 {
        return Err(DecodeError::malformed(format!(
            "{what} has negative length {len}"
        )));
    }
    if len > MAX_BULK_LEN {
        return Err(DecodeError::malformed(format!(
            "{what} length {len} exceeds the {MAX_BULK_LEN}-byte limit"
        )));
    }
    let bytes = cursor.take(len as usize)?;
    cursor.crlf()?;
    Ok(Some(bytes))
}
