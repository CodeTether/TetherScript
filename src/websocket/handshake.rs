//! Validating the WebSocket opening handshake request (RFC 6455 §4.2.1).
//!
//! The server agent hands this module the request headers it already parsed and
//! gets back the accept value to echo. Headers are looked up case-insensitively,
//! because HTTP field names are case-insensitive and a client that sends
//! `sec-websocket-key` is conforming.
//!
//! Validation is not a formality. Requiring `Upgrade: websocket`,
//! `Connection: Upgrade`, and `Sec-WebSocket-Version: 13` is what stops a plain
//! cross-origin `fetch` — which cannot set those headers — from being upgraded
//! into a socket. The key is additionally required to decode to exactly 16 bytes,
//! per §4.1, so a client sending a short or non-base64 nonce is refused rather
//! than being handed a well-formed accept value for a malformed request.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::handshake::validate_request;
//!
//! let headers = [
//!     ("Host", "example.com"),
//!     ("Upgrade", "websocket"),
//!     ("Connection", "keep-alive, Upgrade"),
//!     ("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ=="),
//!     ("Sec-WebSocket-Version", "13"),
//! ];
//! let accept = validate_request(|name| {
//!     headers
//!         .iter()
//!         .find(|(key, _)| key.eq_ignore_ascii_case(name))
//!         .map(|(_, value)| *value)
//! })
//! .unwrap();
//! assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
//! ```

use crate::websocket::handshake_error::HandshakeError;
use crate::websocket::handshake_headers::check_headers;

/// Validate an upgrade request and derive its accept value.
///
/// # Arguments
///
/// * `header` — Case-insensitive header lookup returning the field value, or
///   `None` when the field is absent. Taking a closure keeps this module free of
///   any dependency on the server's header representation.
///
/// # Returns
///
/// The `Sec-WebSocket-Accept` value the 101 response must carry.
///
/// # Errors
///
/// A [`HandshakeError`] naming the header that failed. The caller should answer
/// `400 Bad Request`, or `426 Upgrade Required` for
/// [`HandshakeError::UnsupportedVersion`].
pub fn validate_request<'a, F>(header: F) -> Result<String, HandshakeError>
where
    F: Fn(&str) -> Option<&'a str>,
{
    let key = check_headers(&header)?;
    Ok(crate::websocket::accept::accept_key(key))
}
