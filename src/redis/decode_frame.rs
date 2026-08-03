//! Type-byte dispatch for one RESP frame.
//!
//! Every reply starts with a single byte naming its type: `+` simple, `-` error,
//! `:` integer, `$` bulk, `*` array. Dispatch is recursive only through arrays,
//! and depth is bounded by `MAX_DEPTH` so a reply of nothing but `*1\r\n`
//! repeated cannot overflow the stack.

use super::decode_array;
use super::decode_bulk;
use super::decode_int::{parse_i64, split_error};
use super::decode_line::read_line;
use super::error::RedisError;
use super::limits::MAX_DEPTH;
use super::value::RespValue;

/// Decode the frame beginning at `pos`.
///
/// # Returns
///
/// `Ok(Some((value, next)))` with the offset just past the frame, or `Ok(None)`
/// when the frame is incomplete. `None` propagates outward unchanged so a partial
/// element inside a partial array still reports *incomplete* overall.
///
/// # Errors
///
/// [`RedisError::Protocol`] for an unknown type byte, a malformed number, an
/// over-limit length, or nesting past `MAX_DEPTH`.
pub(super) fn parse(
    input: &[u8],
    pos: usize,
    depth: usize,
) -> Result<Option<(RespValue, usize)>, RedisError> {
    if depth > MAX_DEPTH {
        return Err(RedisError::Protocol(format!(
            "reply nests deeper than {MAX_DEPTH} levels"
        )));
    }
    // The type byte itself may not have arrived yet; that is incomplete, not bad.
    let Some(&marker) = input.get(pos) else {
        return Ok(None);
    };
    let Some((line, next)) = read_line(input, pos + 1)? else {
        return Ok(None);
    };
    match marker {
        b'+' => Ok(Some((RespValue::Simple(text(line, "simple string")?), next))),
        b'-' => {
            let (kind, message) = split_error(&text(line, "error")?);
            Ok(Some((RespValue::Error { kind, message }, next)))
        }
        b':' => Ok(Some((
            RespValue::Integer(parse_i64(line, "integer reply")?),
            next,
        ))),
        b'$' => decode_bulk::decode(input, line, next),
        b'*' => decode_array::decode(input, line, next, depth),
        other => Err(RedisError::Protocol(format!(
            "unknown reply type byte 0x{other:02x}"
        ))),
    }
}

/// Decode a control line as UTF-8. Status and error lines are always text.
fn text(line: &[u8], context: &str) -> Result<String, RedisError> {
    std::str::from_utf8(line)
        .map(str::to_string)
        .map_err(|_| RedisError::Protocol(format!("{context} is not valid UTF-8")))
}
