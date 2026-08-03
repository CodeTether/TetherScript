//! The `101 Switching Protocols` response for a validated handshake.
//!
//! Emitting the response is separated from validating the request so the server
//! agent can decide *when* to write it — after its own routing and capability
//! checks, not as a side effect of parsing.
//!
//! No `Sec-WebSocket-Protocol` or `Sec-WebSocket-Extensions` header is emitted.
//! That is intentional: this codec negotiates no extension, and echoing an
//! extension the codec does not implement (`permessage-deflate`, say) would make
//! every subsequent frame uninterpretable while looking like agreement.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::response::switching_protocols;
//!
//! let response = switching_protocols("s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
//! assert!(response.starts_with("HTTP/1.1 101 Switching Protocols\r\n"));
//! assert!(response.contains("Upgrade: websocket\r\n"));
//! assert!(response.contains("Connection: Upgrade\r\n"));
//! assert!(response.ends_with("\r\n\r\n"));
//! ```

/// Build the `101` response that completes the handshake.
///
/// # Arguments
///
/// * `accept` — The value from
///   [`validate_request`](crate::websocket::handshake::validate_request).
///
/// # Returns
///
/// The full response head, terminated by a blank line. Frame bytes follow it
/// immediately on the same connection.
///
/// # Panics
///
/// Never; this only concatenates strings.
pub fn switching_protocols(accept: &str) -> String {
    format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {accept}\r\n\
         \r\n"
    )
}
