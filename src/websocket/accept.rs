//! The RFC 6455 §4.2.2 `Sec-WebSocket-Accept` derivation.
//!
//! The server proves it understood the upgrade by echoing a value derived from
//! the client's nonce: append the fixed GUID, SHA-1, base64. It is a *handshake
//! confirmation*, not authentication — the GUID is a published constant and the
//! key is not secret, so nothing about this value authorizes anything. It exists
//! so a cached HTTP response or a confused proxy cannot be mistaken for a live
//! WebSocket peer.
//!
//! Base64 is reused from [`crate::system::base64_encode_bytes`]. SHA-1 lives in
//! [`crate::websocket::sha1`]; see that module for why the copy in
//! `src/rpc_cap.rs` could not be reused.
//!
//! # Examples
//!
//! The example key and accept value from RFC 6455 §1.3:
//!
//! ```rust
//! use tetherscript::websocket::accept::accept_key;
//!
//! assert_eq!(
//!     accept_key("dGhlIHNhbXBsZSBub25jZQ=="),
//!     "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=",
//! );
//! ```

/// The GUID RFC 6455 §1.3 fixes for the accept derivation.
pub const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Derive the `Sec-WebSocket-Accept` header value from a client key.
///
/// # Arguments
///
/// * `key` — The client's `Sec-WebSocket-Key` header value, verbatim and already
///   trimmed of surrounding whitespace.
///
/// # Returns
///
/// The base64 `Sec-WebSocket-Accept` value, always 28 characters.
///
/// # Panics
///
/// Never. Both the digest and the base64 encoder operate on owned buffers with no
/// input-derived indexing.
pub fn accept_key(key: &str) -> String {
    let combined = format!("{key}{WS_GUID}");
    let digest = crate::websocket::sha1::sha1(combined.as_bytes());
    crate::system::base64_encode_bytes(&digest)
}
