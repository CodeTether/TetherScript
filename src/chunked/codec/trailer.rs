//! Parsing the trailer section that follows the zero chunk.
//!
//! Layout is zero or more `name: value` lines, then an empty line. The section is bounded
//! three ways — per-line length, total bytes, and field count — because trailers arrive
//! *after* the body, when a server has usually already committed to the request; an
//! unbounded trailer section is a memory-exhaustion hole hiding behind an
//! already-accepted message.
//!
//! Names are lowercased so a caller can compare them without re-normalising, matching how
//! `src/http_response.rs` keys its header map. Values are trimmed of optional whitespace
//! per RFC 9110. A line without a colon is [`ChunkedError::Malformed`]: an obs-fold
//! continuation line is exactly the ambiguity that lets two parsers read different fields.
//!
//! # Panics
//!
//! None. Line framing is delegated to [`crlf_line`], which never panics; the split is done
//! with `iter().position` plus `get`, and `at + 1` cannot exceed the line length because
//! `position` returned an index inside it.

use super::error::ChunkedError;
use super::limits::{MAX_TRAILERS, MAX_TRAILER_BYTES, MAX_TRAILER_LINE_BYTES};
use super::line::crlf_line;

/// Parse trailers starting at `from`, up to and including the terminating empty line.
///
/// # Arguments
///
/// * `input` — Whole buffer being decoded.
/// * `from` — Offset just past the zero chunk's CRLF.
///
/// # Returns
///
/// The lowercased-name/trimmed-value pairs and the offset just past the empty line.
///
/// # Errors
///
/// [`ChunkedError::Incomplete`] until the empty line arrives. [`ChunkedError::Malformed`]
/// for a line with no colon, an empty name, non-UTF-8 bytes, or any bound exceeded.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::decode_trailers;
///
/// let (fields, next) = decode_trailers(b"X-Sum: 9\r\n\r\nrest", 0).unwrap();
/// assert_eq!(fields, vec![("x-sum".to_string(), "9".to_string())]);
/// assert_eq!(next, 12);
///
/// assert_eq!(decode_trailers(b"\r\n", 0).unwrap(), (Vec::new(), 2));
/// ```
pub fn decode_trailers(
    input: &[u8],
    from: usize,
) -> Result<(Vec<(String, String)>, usize), ChunkedError> {
    let mut at = from;
    let mut fields = Vec::new();
    loop {
        let (line, next) = crlf_line(input, at, MAX_TRAILER_LINE_BYTES, "trailer")?;
        at = next;
        if line.is_empty() {
            return Ok((fields, at));
        }
        if at - from > MAX_TRAILER_BYTES {
            return Err(ChunkedError::malformed(format!(
                "trailer section exceeds {MAX_TRAILER_BYTES} bytes"
            )));
        }
        if fields.len() == MAX_TRAILERS {
            return Err(ChunkedError::malformed(format!(
                "more than {MAX_TRAILERS} trailer fields"
            )));
        }
        fields.push(super::trailer_split::split_field(line)?);
    }
}
