//! Continuation of [`ProtocolError`] rendering; see `error_text.rs`.
//!
//! The message table is split across two files purely to honor the 50-line file
//! limit. `error_text` handles the framing-layer violations; this one handles the
//! payload-, close-, and sequencing-layer violations.

use crate::websocket::error::ProtocolError;
use std::fmt;

/// Write the description for the payload/close/sequencing variants.
///
/// # Arguments
///
/// * `error` — The violation to describe.
/// * `f` — Formatter supplied by the [`fmt::Display`] impl.
///
/// # Returns
///
/// Whatever the underlying formatter returns.
pub(super) fn write(error: &ProtocolError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        ProtocolError::PayloadTooLarge { declared, max } => {
            write!(f, "websocket: payload {declared} bytes exceeds bound {max}")
        }
        ProtocolError::MessageTooLarge { total, max } => {
            write!(f, "websocket: message {total} bytes exceeds bound {max}")
        }
        ProtocolError::InvalidUtf8 { context } => {
            write!(f, "websocket: {context} is not valid UTF-8")
        }
        ProtocolError::TruncatedCloseCode => {
            write!(f, "websocket: close body of 1 byte cannot hold a code")
        }
        ProtocolError::ForbiddenCloseCode { code } => {
            write!(f, "websocket: close code {code} is not allowed on the wire")
        }
        ProtocolError::UnexpectedContinuation => {
            write!(f, "websocket: continuation with no message in progress")
        }
        ProtocolError::InterleavedDataFrame => {
            write!(
                f,
                "websocket: data frame interleaved into a fragmented message"
            )
        }
        // The framing variants are rendered by `error_text::write`, which only
        // delegates here for the cases above; this arm is therefore unreachable
        // in practice but is written out rather than using `unreachable!()` so a
        // future variant cannot introduce a panic path.
        other => write!(f, "websocket: protocol violation ({other:?})"),
    }
}
