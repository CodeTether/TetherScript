//! Owned nonblocking TCP streams.

mod connect;
mod io;

use std::net::{Shutdown, TcpStream};
use std::time::Duration;

use crate::value::Value;

use super::result;

pub(super) struct Handle {
    stream: TcpStream,
}

impl Handle {
    pub(super) fn connect(host: &str, port: u16, timeout: Duration) -> Result<Self, String> {
        connect::stream(host, port, timeout).map(|stream| Self { stream })
    }

    pub(super) fn accepted(stream: TcpStream) -> Result<Self, String> {
        stream
            .set_nonblocking(true)
            .map_err(|error| format!("tcp_stream.accept: set nonblocking: {error}"))?;
        Ok(Self { stream })
    }

    pub(super) fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        match (name, args) {
            ("read", [limit]) => Ok(result::value(io::read(&mut self.stream, limit))),
            ("write", [body]) => Ok(result::value(io::write(&mut self.stream, body))),
            ("peer_addr", []) => Ok(result::value(io::peer_addr(&self.stream))),
            ("shutdown", []) => Ok(result::nil(self.cancel())),
            _ => Err(format!(
                "tcp_stream: no method `{name}` accepting {} arguments",
                args.len()
            )),
        }
    }

    pub(super) fn cancel(&mut self) -> Result<(), String> {
        self.stream
            .shutdown(Shutdown::Both)
            .map_err(|error| format!("tcp_stream.cancel: {error}"))
    }
}
