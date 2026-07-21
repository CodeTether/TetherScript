//! Deadline-bounded TCP connection establishment.

use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub(super) fn stream(host: &str, port: u16, timeout: Duration) -> Result<TcpStream, String> {
    let addresses = (host, port)
        .to_socket_addrs()
        .map_err(|error| format!("resource.tcp_connect {host}:{port}: resolve failed: {error}"))?;
    let mut last_error = None;
    for address in addresses {
        match TcpStream::connect_timeout(&address, timeout) {
            Ok(stream) => {
                stream.set_nonblocking(true).map_err(|error| {
                    format!("resource.tcp_connect {address}: set nonblocking: {error}")
                })?;
                return Ok(stream);
            }
            Err(error) => last_error = Some(error),
        }
    }
    Err(format!(
        "resource.tcp_connect {host}:{port}: {}",
        last_error.map_or_else(|| "no addresses resolved".into(), |error| error.to_string())
    ))
}
