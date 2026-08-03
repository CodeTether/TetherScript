//! CRLF line scanning for the RESP decoder.
//!
//! Only *control* lines are scanned this way: the type/length lines, status
//! strings, error strings, and integers. A bulk **body** is never scanned for
//! CRLF, because it is length-prefixed and may legitimately contain CRLF —
//! splitting a reply on CRLF is the classic bug this module is shaped to avoid.
//! See the `decode_bulk` module.

use super::error::RedisError;
use super::limits::MAX_LINE_LEN;

/// Find the next CRLF-terminated line starting at `pos`.
///
/// # Arguments
///
/// * `input` — Bytes received so far, not necessarily a whole reply.
/// * `pos` — Offset to start scanning from.
///
/// # Returns
///
/// * `Ok(Some((line, next)))` — `line` excludes the terminator and `next` is the
///   offset just past it.
/// * `Ok(None)` — no terminator yet: the caller needs to read more bytes. This is
///   the ordinary short-read case and is not an error.
///
/// # Errors
///
/// [`RedisError::Protocol`] once the unterminated run exceeds `MAX_LINE_LEN`,
/// so a peer that never sends CRLF cannot make the buffer grow without bound
/// while the decoder keeps politely asking for more.
pub(super) fn read_line(input: &[u8], pos: usize) -> Result<Option<(&[u8], usize)>, RedisError> {
    let tail = input.get(pos..).unwrap_or(&[]);
    // Stop at len-1 so `window[1]` is always in range.
    for index in 0..tail.len().saturating_sub(1) {
        if tail[index] == b'\r' && tail[index + 1] == b'\n' {
            return Ok(Some((&tail[..index], pos + index + 2)));
        }
    }
    if tail.len() > MAX_LINE_LEN {
        return Err(RedisError::Protocol(format!(
            "unterminated line exceeds {MAX_LINE_LEN} bytes"
        )));
    }
    Ok(None)
}
