//! The public RESP decoder entry point.
//!
//! # Incremental safety
//!
//! A TCP read returns whatever bytes have arrived, which may be half a reply, one
//! and a half replies, or a single byte. The decoder therefore never assumes it
//! has a whole frame: it reports [`Decoded::Incomplete`] and leaves the caller's
//! buffer untouched, so the caller reads more and calls again. It never panics on
//! a short read, never slices past the end of the input, and never partially
//! consumes a frame it could not finish.
//!
//! Distinguish the three outcomes carefully:
//!
//! | Outcome | Meaning | Connection still usable |
//! |---|---|---|
//! | [`Decoded::Frame`] | A complete reply, and how many bytes it used | yes |
//! | [`Decoded::Incomplete`] | Need more bytes; nothing consumed | yes |
//! | `Err(`[`RedisError::Protocol`]`)` | The bytes are not RESP | no — framing lost |
//!
//! An error *reply* (`-ERR ...`) is none of the failure cases above: it decodes
//! successfully into [`RespValue::Error`], because the protocol worked.

use super::decode_frame::parse;
use super::error::RedisError;
use super::value::RespValue;

/// The result of attempting to decode one reply from a buffer.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis::{decode, Decoded};
///
/// // A truncated bulk string asks for more bytes instead of panicking.
/// assert_eq!(decode(b"$5\r\nhel").unwrap(), Decoded::Incomplete);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decoded {
    /// One complete reply, plus the number of input bytes it consumed so the
    /// caller can drain exactly that much and keep any pipelined remainder.
    Frame {
        /// The decoded reply.
        value: RespValue,
        /// Bytes consumed from the front of the input.
        consumed: usize,
    },
    /// The input holds a valid but incomplete prefix. Read more and retry.
    Incomplete,
}

/// Decode the first reply in `input`.
///
/// # Arguments
///
/// * `input` — Bytes received so far. May contain a partial reply, or several
///   replies when commands were pipelined.
///
/// # Returns
///
/// [`Decoded::Frame`] with the value and its byte length, or
/// [`Decoded::Incomplete`] when more bytes are needed.
///
/// # Errors
///
/// [`RedisError::Protocol`] for an unknown type byte, a malformed length, a
/// declared length past the limits in the `limits` module, or nesting deeper than
/// `MAX_DEPTH`.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis::{decode, Decoded, RespValue};
///
/// let Decoded::Frame { value, consumed } = decode(b":42\r\n:7\r\n").unwrap() else {
///     panic!("expected a complete frame");
/// };
/// assert_eq!(value, RespValue::Integer(42));
/// assert_eq!(consumed, 5); // the pipelined `:7\r\n` is left for the next call
/// ```
pub fn decode(input: &[u8]) -> Result<Decoded, RedisError> {
    match parse(input, 0, 0)? {
        Some((value, next)) => Ok(Decoded::Frame {
            value,
            consumed: next,
        }),
        None => Ok(Decoded::Incomplete),
    }
}
