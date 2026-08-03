//! `Sec-WebSocket-Key` shape validation (RFC 6455 §4.1, item 7).
//!
//! The key must be the base64 encoding of a 16-byte nonce. Both halves matter:
//! an unbounded string would be hashed as-is (SHA-1 accepts anything), so without
//! the length check a client could send megabytes of "key" and make the server do
//! the hashing. Requiring exactly 16 decoded bytes bounds that work to one SHA-1
//! block-pair and rejects a request that was never a real handshake.
//!
//! The base64 decoder is reused from [`crate::system::base64_decode_bytes`].
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::handshake_key::check_key;
//!
//! assert!(check_key("dGhlIHNhbXBsZSBub25jZQ==").is_ok());
//! // Decodes to 4 bytes, not 16.
//! assert!(check_key("AAAAAA==").is_err());
//! // Not base64 at all.
//! assert!(check_key("!!!!").is_err());
//! ```

use crate::websocket::handshake_error::HandshakeError;

/// The nonce length RFC 6455 fixes, in bytes.
const NONCE_LEN: usize = 16;

/// The base64 length of a 16-byte nonce, including padding.
const KEY_LEN: usize = 24;

/// Verify that `key` is base64 for exactly 16 bytes.
///
/// # Arguments
///
/// * `key` — The trimmed `Sec-WebSocket-Key` value.
///
/// # Returns
///
/// `Ok(())` when the key has the required shape.
///
/// # Errors
///
/// [`HandshakeError::BadKey`] describing whether the length or the encoding was
/// wrong.
pub fn check_key(key: &str) -> Result<(), HandshakeError> {
    if key.len() != KEY_LEN {
        return Err(HandshakeError::BadKey {
            reason: format!("is {} chars, want {KEY_LEN}", key.len()),
        });
    }
    let bytes = crate::system::base64_decode_bytes(key).map_err(|error| HandshakeError::BadKey {
        reason: format!("is not base64: {error}"),
    })?;
    if bytes.len() != NONCE_LEN {
        return Err(HandshakeError::BadKey {
            reason: format!("decodes to {} bytes, want {NONCE_LEN}", bytes.len()),
        });
    }
    Ok(())
}
