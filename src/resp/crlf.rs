//! # Finding the CRLF terminator
//!
//! RESP delimits every line with CRLF, so "where does this line end?" is asked at
//! every step of decoding. It lives in its own module rather than inside
//! [`super::cursor`] so that the cursor stays purely about position, and so the
//! one place that decides between "no terminator yet" and "no terminator ever"
//! can be read on its own.
//!
//! That decision is the important one. A missing CRLF is normally
//! [`DecodeError::Incomplete`] — the rest is still in flight — but a peer that
//! never sends one would keep the caller waiting while its buffer grew without
//! limit. Past [`MAX_LINE_LEN`] the absence is treated as
//! [`DecodeError::Malformed`] instead.

use super::error::DecodeError;
use super::limits::MAX_LINE_LEN;

/// Offset of the first CRLF in `bytes`, i.e. the length of the line before it.
///
/// # Returns
///
/// The number of bytes preceding the first `\r\n`. Zero for a buffer starting
/// with CRLF, which is a legal empty line such as the payload of `+\r\n`.
///
/// # Errors
///
/// [`DecodeError::Incomplete`] when no CRLF is present and the run so far is
/// within [`MAX_LINE_LEN`]; [`DecodeError::Malformed`] once it is longer, naming
/// the bound that was passed.
pub(super) fn find(bytes: &[u8]) -> Result<usize, DecodeError> {
    match bytes.windows(2).position(|pair| pair == b"\r\n") {
        Some(offset) => Ok(offset),
        None if bytes.len() > MAX_LINE_LEN => Err(DecodeError::malformed(format!(
            "line exceeds {MAX_LINE_LEN} bytes with no CRLF"
        ))),
        None => Err(DecodeError::Incomplete),
    }
}
