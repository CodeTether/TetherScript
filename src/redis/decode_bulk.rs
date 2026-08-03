//! Bulk string decoding: `$<len>\r\n<payload>\r\n`.
//!
//! # Read exactly the declared length
//!
//! The payload is binary-safe, so it may contain CRLF. Scanning forward for the
//! next `\r\n` to find the end of the body is the classic RESP bug: given
//! `$7\r\na\r\nbcd\r\n`, a CRLF scan stops after `a` and returns a one-byte value,
//! then treats `bcd` as the start of the next reply and desynchronises the
//! connection permanently. This module instead takes exactly `len` bytes and then
//! *verifies* the two bytes that follow are the terminator.
//!
//! # Null versus empty
//!
//! `$-1\r\n` is the null bulk string — the key does not exist — and has no body at
//! all. `$0\r\n\r\n` is a key that exists holding the empty string. They decode to
//! [`RespValue::NullBulk`] and `RespValue::Bulk(vec![])` respectively and are not
//! interchangeable. Any negative length other than `-1` is malformed rather than
//! null, because RESP defines only `-1`.

use super::decode_int::parse_i64;
use super::error::RedisError;
use super::limits::MAX_BULK_LEN;
use super::value::RespValue;

/// Decode a bulk string whose length line is `line` and whose body starts at `pos`.
///
/// # Returns
///
/// `Ok(Some((value, next)))` on success, `Ok(None)` when the body or its
/// terminator has not fully arrived.
///
/// # Errors
///
/// [`RedisError::Protocol`] for a malformed length, a negative length other than
/// `-1`, a length above `MAX_BULK_LEN`, or a body not followed by CRLF.
pub(super) fn decode(
    input: &[u8],
    line: &[u8],
    pos: usize,
) -> Result<Option<(RespValue, usize)>, RedisError> {
    let declared = parse_i64(line, "bulk length")?;
    if declared == -1 {
        return Ok(Some((RespValue::NullBulk, pos)));
    }
    let len = checked_len(declared)?;
    // Guard the allocation *and* the slice: `pos + len` could overflow on a
    // hostile length, and the body plus terminator must both be present.
    let end = pos.saturating_add(len);
    if input.len() < end + 2 {
        return Ok(None);
    }
    if &input[end..end + 2] != b"\r\n" {
        return Err(RedisError::Protocol(format!(
            "bulk body of {len} bytes is not followed by CRLF"
        )));
    }
    Ok(Some((RespValue::Bulk(input[pos..end].to_vec()), end + 2)))
}

/// Validate a declared bulk length before it is used to size an allocation.
fn checked_len(declared: i64) -> Result<usize, RedisError> {
    if declared < 0 {
        return Err(RedisError::Protocol(format!(
            "bulk length {declared} is negative; only -1 means null"
        )));
    }
    let len = declared as u64;
    if len > MAX_BULK_LEN as u64 {
        return Err(RedisError::Protocol(format!(
            "bulk length {len} exceeds the {MAX_BULK_LEN}-byte limit"
        )));
    }
    Ok(len as usize)
}
