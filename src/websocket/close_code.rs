//! Which close status codes may appear on the wire (RFC 6455 §7.4).
//!
//! The permitted set is deliberately narrow:
//!
//! | Range | Status |
//! |---|---|
//! | `1000..=1003`, `1007..=1011`, `1012..=1014` | Registered, allowed |
//! | `1005`, `1006`, `1015` | **Never on the wire** |
//! | `1016..=2999` | Reserved for future RFCs, rejected |
//! | `3000..=4999` | Registered/private use, allowed |
//! | everything else (`0..=999`, `5000..`) | Rejected |
//!
//! 1005 (no status), 1006 (abnormal closure), and 1015 (TLS handshake failure)
//! are *locally generated designations*. A peer that sends one is claiming an
//! event that, by definition, cannot be reported by a frame that arrived — so
//! passing it through would let a remote forge a "the connection died
//! abnormally" signal to the application.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::close_code;
//!
//! assert!(close_code::check(1000).is_ok());
//! assert!(close_code::check(3000).is_ok());
//! assert!(close_code::check(1005).is_err());
//! assert!(close_code::check(1006).is_err());
//! assert!(close_code::check(1015).is_err());
//! assert!(close_code::check(999).is_err());
//! ```

use crate::websocket::error::ProtocolError;

/// Reject a close code that must not appear on the wire.
///
/// # Arguments
///
/// * `code` — The 16-bit status code read from a close body.
///
/// # Returns
///
/// `Ok(())` when the code is permitted for transmission.
///
/// # Errors
///
/// [`ProtocolError::ForbiddenCloseCode`] naming the rejected code.
pub fn check(code: u16) -> Result<(), ProtocolError> {
    let allowed = matches!(code, 1000..=1003 | 1007..=1014 | 3000..=4999);
    if allowed {
        return Ok(());
    }
    Err(ProtocolError::ForbiddenCloseCode { code })
}
