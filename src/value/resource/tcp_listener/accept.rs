//! Listener accept operations and backpressure.

use std::io;
use std::net::TcpListener;

use crate::value::Value;

use super::super::{tcp_stream, OwnedResource};

pub(super) fn stream(listener: &TcpListener) -> Result<Value, String> {
    let (stream, _) = listener.accept().map_err(accept_error)?;
    let resource = OwnedResource::new(super::super::payload::Payload::TcpStream(
        tcp_stream::Handle::accepted(stream)?,
    ));
    Ok(Value::Resource(std::rc::Rc::new(std::cell::RefCell::new(
        resource,
    ))))
}

pub(super) fn address(listener: &TcpListener) -> Result<Value, String> {
    listener
        .local_addr()
        .map(|address| Value::Str(std::rc::Rc::new(address.to_string())))
        .map_err(|error| format!("tcp_listener.local_addr: {error}"))
}

pub(super) fn port(listener: &TcpListener) -> Result<Value, String> {
    listener
        .local_addr()
        .map(|address| Value::Int(address.port() as i64))
        .map_err(|error| format!("tcp_listener.port: {error}"))
}

fn accept_error(error: io::Error) -> String {
    if error.kind() == io::ErrorKind::WouldBlock {
        "tcp_listener.accept: backpressure: no connection is ready".into()
    } else {
        format!("tcp_listener.accept: {error}")
    }
}
