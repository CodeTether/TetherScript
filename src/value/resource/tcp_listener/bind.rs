//! Listener binding policy.

use std::net::TcpListener;

pub(super) fn listener(host: &str, port: u16) -> Result<TcpListener, String> {
    let listener = TcpListener::bind((host, port))
        .map_err(|error| format!("resource.tcp_listen {host}:{port}: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("resource.tcp_listen {host}:{port}: set nonblocking: {error}"))?;
    Ok(listener)
}
