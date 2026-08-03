//! Array decoding: `*<count>\r\n` followed by `count` nested frames.
//!
//! Elements are themselves complete RESP frames of any type, so arrays nest and
//! decoding recurses through the frame dispatcher with the depth incremented.
//!
//! # Null versus empty
//!
//! `*-1\r\n` is the null array — no result, as returned by a timed-out blocking
//! command — and decodes to [`RespValue::NullArray`]. `*0\r\n` is an array that
//! exists and holds nothing, decoding to `RespValue::Array(vec![])`. As with bulk
//! strings, the two are kept distinct.
//!
//! # Incremental safety
//!
//! The count is *not* used to pre-allocate the full vector. A hostile `*1000000`
//! with no elements would otherwise reserve a million slots from four bytes of
//! input; capacity therefore grows with the elements actually decoded, and the
//! count is bounds-checked against `MAX_ARRAY_LEN` first.

use super::decode_frame::parse;
use super::decode_int::parse_i64;
use super::error::RedisError;
use super::limits::MAX_ARRAY_LEN;
use super::value::RespValue;

/// Decode an array whose count line is `line` and whose first element is at `pos`.
///
/// # Returns
///
/// `Ok(Some((value, next)))`, or `Ok(None)` when any element is still incomplete.
///
/// # Errors
///
/// [`RedisError::Protocol`] for a malformed count, a negative count other than
/// `-1`, or a count above `MAX_ARRAY_LEN`.
pub(super) fn decode(
    input: &[u8],
    line: &[u8],
    pos: usize,
    depth: usize,
) -> Result<Option<(RespValue, usize)>, RedisError> {
    let declared = parse_i64(line, "array length")?;
    if declared == -1 {
        return Ok(Some((RespValue::NullArray, pos)));
    }
    let count = checked_count(declared)?;
    let mut items = Vec::new();
    let mut cursor = pos;
    for _ in 0..count {
        // A partial element makes the whole array incomplete: report it and let
        // the caller retry from the original offset with more bytes.
        let Some((item, next)) = parse(input, cursor, depth + 1)? else {
            return Ok(None);
        };
        items.push(item);
        cursor = next;
    }
    Ok(Some((RespValue::Array(items), cursor)))
}

/// Validate a declared element count before it drives a decode loop.
fn checked_count(declared: i64) -> Result<usize, RedisError> {
    if declared < 0 {
        return Err(RedisError::Protocol(format!(
            "array length {declared} is negative; only -1 means null"
        )));
    }
    if declared as u64 > MAX_ARRAY_LEN as u64 {
        return Err(RedisError::Protocol(format!(
            "array length {declared} exceeds the {MAX_ARRAY_LEN}-element limit"
        )));
    }
    Ok(declared as usize)
}
