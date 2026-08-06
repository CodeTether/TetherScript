//! Datagram send and receive.

use std::net::UdpSocket;

use crate::value::Value;

use super::super::args;
use super::pressure::pressure;

/// Largest datagram this module will attempt to receive.
const MAX_DATAGRAM_BYTES: usize = 65_507;

/// Receive one datagram, returning a map with `bytes` and `from`.
pub(super) fn recv_from(socket: &UdpSocket, limit: &Value) -> Result<Value, String> {
    let limit = args::usize(limit, "udp_socket.recv_from limit")?;
    if limit > MAX_DATAGRAM_BYTES {
        return Err(format!(
            "udp_socket.recv_from limit {limit} exceeds maximum {MAX_DATAGRAM_BYTES}"
        ));
    }
    let mut buffer = vec![0; limit];
    let (count, from) = socket
        .recv_from(&mut buffer)
        .map_err(|error| pressure("recv_from", error))?;
    buffer.truncate(count);
    Ok(super::datagram::value(buffer, &from.to_string()))
}

/// Send `body` to `host:port`, returning the byte count written.
pub(super) fn send_to(
    socket: &UdpSocket,
    body: &Value,
    host: &str,
    port: u16,
) -> Result<Value, String> {
    let bytes = args::bytes(body, "udp_socket.send_to body")?;
    socket
        .send_to(&bytes, (host, port))
        .map(|count| Value::Int(count as i64))
        .map_err(|error| pressure("send_to", error))
}
