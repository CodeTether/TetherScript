//! UDP socket binding policy.

use std::net::UdpSocket;

/// Bind a nonblocking UDP socket to `host:port`.
///
/// # Errors
///
/// Returns a host-and-port-qualified bind or configuration error.
pub(super) fn socket(host: &str, port: u16) -> Result<UdpSocket, String> {
    let socket = UdpSocket::bind((host, port))
        .map_err(|error| format!("resource.udp_bind {host}:{port}: {error}"))?;
    socket
        .set_nonblocking(true)
        .map_err(|error| format!("resource.udp_bind {host}:{port}: set nonblocking: {error}"))?;
    Ok(socket)
}
