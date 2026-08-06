//! Local address introspection for a bound UDP socket.

use std::net::UdpSocket;
use std::rc::Rc;

use crate::value::Value;

/// Report the bound local address as `host:port`.
pub(super) fn local_addr(socket: &UdpSocket) -> Result<Value, String> {
    socket
        .local_addr()
        .map(|address| Value::Str(Rc::new(address.to_string())))
        .map_err(|error| format!("udp_socket.local_addr: {error}"))
}

/// Report the bound local port, which is what `udp_bind(host, 0)` assigned.
pub(super) fn port(socket: &UdpSocket) -> Result<Value, String> {
    socket
        .local_addr()
        .map(|address| Value::Int(i64::from(address.port())))
        .map_err(|error| format!("udp_socket.port: {error}"))
}
