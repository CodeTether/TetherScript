//! # Why decoding fails
//!
//! Exactly two things can go wrong while decoding, and a client that confuses
//! them is broken in a way that is hard to see in testing:
//!
//! - [`DecodeError::Incomplete`] means *read more and try again*. The bytes seen
//!   so far are a valid prefix of some reply. Nothing was consumed, so the caller
//!   must keep its buffer exactly as it is.
//! - [`DecodeError::Malformed`] means *the stream is not RESP*. Resynchronising is
//!   not possible, because framing depends on the very bytes that turned out to be
//!   wrong, so the only correct response is to drop the connection.
//!
//! A `-ERR ...` reply from the server is **neither** of these. It is a
//! successfully decoded [`Reply::Error`](super::reply::Reply::Error); see that
//! variant for the reasoning.

use std::error::Error;
use std::fmt;

/// The reason a [`decode`](super::decode) call did not produce a reply.
///
/// # Examples
///
/// ```rust
/// use tetherscript::resp::codec::{decode, DecodeError};
///
/// // A truncated integer reply: valid so far, just not finished.
/// assert_eq!(decode(b":12").unwrap_err(), DecodeError::Incomplete);
///
/// // An unknown type byte can never become valid, however much arrives next.
/// assert!(matches!(decode(b"^nope\r\n"), Err(DecodeError::Malformed(_))));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// The buffer holds a valid prefix of a reply. Read more bytes and retry; the
    /// buffer must not be advanced.
    Incomplete,
    /// The buffer violates the protocol or a bound in [`super::limits`]. The
    /// string names what was wrong, so the failure is diagnosable from a log line
    /// alone.
    Malformed(String),
}

impl DecodeError {
    /// Build a [`DecodeError::Malformed`] from any message-like value.
    pub(super) fn malformed<S: Into<String>>(message: S) -> Self {
        DecodeError::Malformed(message.into())
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Incomplete => write!(f, "resp: reply is incomplete"),
            DecodeError::Malformed(message) => write!(f, "resp: {message}"),
        }
    }
}

impl Error for DecodeError {}
