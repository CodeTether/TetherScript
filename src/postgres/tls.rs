//! TLS negotiation for the PostgreSQL wire protocol.
//!
//! PostgreSQL does not use a separate TLS port. A client sends an `SSLRequest` in
//! cleartext and reads a single byte: `S` to proceed with a handshake on the same
//! socket, `N` to refuse. Only after `S` does any TLS record appear, which is why
//! the connector must wrap an already-connected stream.

use std::io::{Read, Write};
use std::net::TcpStream;

use super::encode::Builder;
use super::transport::Socket;
use crate::tls::TlsConnector;

/// The protocol version code that requests TLS instead of a normal startup.
const SSL_REQUEST_CODE: i32 = 80877103;

/// Ask the server for TLS and wrap the socket when it agrees.
///
/// # Arguments
///
/// * `tcp` — Freshly connected socket, before any startup message.
/// * `host` — Hostname used to validate the server certificate.
///
/// # Returns
///
/// A TLS-wrapped socket.
///
/// # Errors
///
/// Returns an error when the server refuses TLS, when the reply is malformed, or
/// when certificate or hostname validation fails. Refusal is an error rather than a
/// silent downgrade: a caller that asked for TLS must never get cleartext instead.
pub(super) fn negotiate(mut tcp: TcpStream, host: &str) -> Result<Socket, String> {
    let mut request = Builder::untagged();
    request.i32(SSL_REQUEST_CODE);
    tcp.write_all(&request.finish())
        .map_err(|error| format!("postgres: send SSLRequest: {error}"))?;

    let mut reply = [0u8; 1];
    tcp.read_exact(&mut reply)
        .map_err(|error| format!("postgres: read SSLRequest reply: {error}"))?;
    match reply[0] {
        b'S' => wrap(tcp, host),
        b'N' => Err(format!(
            "postgres: server at {host} refused TLS (replied `N`); \
             retry without sslmode=require only if the network is trusted"
        )),
        other => Err(format!(
            "postgres: unexpected SSLRequest reply byte {other:#04x}, expected `S` or `N`"
        )),
    }
}

/// Perform the handshake, validating the certificate against `host`.
fn wrap(tcp: TcpStream, host: &str) -> Result<Socket, String> {
    let connector =
        TlsConnector::new().map_err(|error| format!("postgres: TLS unavailable: {error}"))?;
    let stream = connector
        .connect_over(host, tcp)
        .map_err(|error| format!("postgres: TLS handshake with {host}: {error}"))?;
    Ok(Box::new(stream))
}
