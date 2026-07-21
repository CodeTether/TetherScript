//! Owned nonblocking TCP listeners.

mod accept;
mod bind;

use std::net::TcpListener;

use crate::value::Value;

use super::result;

pub(super) struct Handle {
    listener: TcpListener,
}

impl Handle {
    pub(super) fn bind(host: &str, port: u16) -> Result<Self, String> {
        bind::listener(host, port).map(|listener| Self { listener })
    }

    pub(super) fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        match (name, args) {
            ("accept", []) => Ok(result::value(accept::stream(&self.listener))),
            ("local_addr", []) => Ok(result::value(accept::address(&self.listener))),
            ("port", []) => Ok(result::value(accept::port(&self.listener))),
            _ => Err(format!(
                "tcp_listener: no method `{name}` accepting {} arguments",
                args.len()
            )),
        }
    }
}
