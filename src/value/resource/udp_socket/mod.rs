//! Owned nonblocking UDP sockets.
//!
//! Move-only like the other owned resources: a datagram socket is an OS handle,
//! so duplicating it would duplicate authority over a port.

mod address;
mod bind;
mod datagram;
mod io;
mod pressure;

use std::net::UdpSocket;

use crate::value::Value;

use super::{args, factory, result};

pub(super) struct Handle {
    socket: UdpSocket,
}

impl Handle {
    pub(super) fn bind(host: &str, port: u16) -> Result<Self, String> {
        bind::socket(host, port).map(|socket| Self { socket })
    }

    pub(super) fn call(&mut self, name: &str, arguments: &[Value]) -> Result<Value, String> {
        match (name, arguments) {
            ("recv_from", [limit]) => Ok(result::value(io::recv_from(&self.socket, limit))),
            ("send_to", [body, host, port]) => Ok(result::value(self.send_to(body, host, port))),
            ("local_addr", []) => Ok(result::value(address::local_addr(&self.socket))),
            ("port", []) => Ok(result::value(address::port(&self.socket))),
            _ => Err(format!(
                "udp_socket: no method `{name}` accepting {} arguments",
                arguments.len()
            )),
        }
    }

    /// Authorize the destination before sending, so a granted socket cannot be
    /// reused to reach an address outside the grant.
    fn send_to(&self, body: &Value, host: &Value, port: &Value) -> Result<Value, String> {
        let host = args::string(host, "udp_socket.send_to host")?;
        let port = factory::port(port, "udp_socket.send_to port")?;
        crate::socket_cap::require("udp_socket.send_to", &host, port)?;
        io::send_to(&self.socket, body, &host, port)
    }
}
