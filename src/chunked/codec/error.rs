//! The single error type of the chunked codec.
//!
//! The distinction between the two variants is load-bearing. [`ChunkedError::Incomplete`]
//! means "these bytes are a valid *prefix* of a chunked body; read more and call again,
//! having consumed nothing". [`ChunkedError::Malformed`] means "no continuation of these
//! bytes can ever be valid; fail the message and close the connection".
//!
//! Collapsing the two would be a security bug in either direction: treating a prefix as
//! malformed breaks every peer whose writes happen to split a chunk, and treating garbage
//! as a prefix lets an attacker hold a connection open forever.
//!
//! A bound violation is always `Malformed`, never `Incomplete` — an over-long claim is
//! decidable from the bytes already in hand.

use std::fmt;

/// Why a chunked decode did not produce a body.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::{decode, ChunkedError};
///
/// // A valid body missing its final byte is a prefix, not garbage.
/// assert!(matches!(decode(b"5\r\nhello\r\n0\r\n"), Err(ChunkedError::Incomplete)));
///
/// // A signed size can never become valid.
/// assert!(matches!(decode(b"+5\r\nhello\r\n"), Err(ChunkedError::Malformed(_))));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkedError {
    /// The input is a valid prefix; nothing was consumed. Read more bytes and retry.
    Incomplete,
    /// The input can never become valid. The payload names the offending construct.
    Malformed(String),
}

impl ChunkedError {
    /// Build a [`ChunkedError::Malformed`] from any displayable reason.
    ///
    /// # Arguments
    ///
    /// * `reason` — Human-readable description naming the offending bytes or bound.
    ///
    /// # Returns
    ///
    /// The `Malformed` variant carrying `reason`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::chunked::codec::ChunkedError;
    ///
    /// let error = ChunkedError::malformed("chunk size line is empty");
    /// assert_eq!(error.to_string(), "malformed chunked body: chunk size line is empty");
    /// ```
    pub fn malformed(reason: impl fmt::Display) -> Self {
        ChunkedError::Malformed(reason.to_string())
    }
}

impl fmt::Display for ChunkedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChunkedError::Incomplete => write!(f, "incomplete chunked body: need more bytes"),
            ChunkedError::Malformed(reason) => write!(f, "malformed chunked body: {reason}"),
        }
    }
}

impl std::error::Error for ChunkedError {}
