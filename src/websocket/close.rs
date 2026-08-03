//! Close-frame bodies: a 2-byte big-endian code plus an optional UTF-8 reason.
//!
//! Three lengths are legal and one is not. An empty body means "no status
//! given". Two or more bytes means a code and possibly a reason. **One** byte is
//! a protocol error, because half a code is not a code — accepting it would force
//! a guess about the missing octet.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::close::{self, CloseFrame};
//!
//! // 1000 "bye"
//! let body = [0x03, 0xe8, b'b', b'y', b'e'];
//! let parsed = close::validate(&body).unwrap().unwrap();
//! assert_eq!(parsed.code, 1000);
//! assert_eq!(parsed.reason, "bye");
//!
//! // An empty body is "no status given".
//! assert_eq!(close::validate(&[]).unwrap(), None);
//! // 1006 is generated locally and must never be received.
//! assert!(close::validate(&[0x03, 0xee]).is_err());
//! // A one-byte body cannot hold a code.
//! assert!(close::validate(&[0x03]).is_err());
//!
//! let encoded = CloseFrame { code: 1000, reason: "bye".into() }.to_payload();
//! assert_eq!(encoded, body.to_vec());
//! ```

use crate::websocket::close_code;
use crate::websocket::error::ProtocolError;
use crate::websocket::validate;

/// A parsed close-frame body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseFrame {
    /// The status code, already checked against the permitted ranges.
    pub code: u16,
    /// The optional reason, guaranteed valid UTF-8. Empty when absent.
    pub reason: String,
}

impl CloseFrame {
    /// Encode this close frame as a payload body.
    ///
    /// # Returns
    ///
    /// The code in big-endian order followed by the UTF-8 reason.
    pub fn to_payload(&self) -> Vec<u8> {
        let mut out = self.code.to_be_bytes().to_vec();
        out.extend_from_slice(self.reason.as_bytes());
        out
    }
}

/// Validate and parse a close-frame body.
///
/// # Arguments
///
/// * `payload` — The unmasked close payload; may be empty.
///
/// # Returns
///
/// `Ok(None)` for an empty body, otherwise `Ok(Some(close))`.
///
/// # Errors
///
/// [`ProtocolError::TruncatedCloseCode`] for a one-byte body,
/// [`ProtocolError::ForbiddenCloseCode`] for a disallowed code, and
/// [`ProtocolError::InvalidUtf8`] for a non-UTF-8 reason.
///
/// # Panics
///
/// Never. The code is read from a slice obtained with `get(0..2)` and the reason
/// from `get(2..)`, so a short body returns an error instead of indexing.
pub fn validate(payload: &[u8]) -> Result<Option<CloseFrame>, ProtocolError> {
    if payload.is_empty() {
        return Ok(None);
    }
    let Some(head) = payload.get(0..2) else {
        return Err(ProtocolError::TruncatedCloseCode);
    };
    let code = u16::from_be_bytes([head[0], head[1]]);
    close_code::check(code)?;
    let tail = payload.get(2..).unwrap_or(&[]);
    validate::utf8(tail, "close reason")?;
    Ok(Some(CloseFrame {
        code,
        reason: String::from_utf8_lossy(tail).into_owned(),
    }))
}
