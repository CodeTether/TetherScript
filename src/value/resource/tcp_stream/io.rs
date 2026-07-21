//! Nonblocking stream I/O and backpressure diagnostics.

use std::io::{self, Read, Write};
use std::net::TcpStream;

use crate::value::Value;

use super::super::args;

const MAX_READ_BYTES: usize = 16 * 1024 * 1024;

pub(super) fn read(stream: &mut TcpStream, limit: &Value) -> Result<Value, String> {
    let limit = args::usize(limit, "tcp_stream.read limit")?;
    if limit > MAX_READ_BYTES {
        return Err(format!(
            "tcp_stream.read limit {limit} exceeds maximum {MAX_READ_BYTES}"
        ));
    }
    let mut bytes = vec![0; limit];
    let count = stream
        .read(&mut bytes)
        .map_err(|error| pressure("read", error))?;
    bytes.truncate(count);
    Ok(Value::Bytes(std::rc::Rc::new(std::cell::RefCell::new(
        bytes,
    ))))
}

pub(super) fn write(stream: &mut TcpStream, body: &Value) -> Result<Value, String> {
    let bytes = args::bytes(body, "tcp_stream.write body")?;
    stream
        .write(&bytes)
        .map(|count| Value::Int(count as i64))
        .map_err(|error| pressure("write", error))
}

pub(super) fn peer_addr(stream: &TcpStream) -> Result<Value, String> {
    stream
        .peer_addr()
        .map(|address| Value::Str(std::rc::Rc::new(address.to_string())))
        .map_err(|error| format!("tcp_stream.peer_addr: {error}"))
}

fn pressure(operation: &str, error: io::Error) -> String {
    if error.kind() == io::ErrorKind::WouldBlock {
        format!("tcp_stream.{operation}: backpressure: operation would block")
    } else {
        format!("tcp_stream.{operation}: {error}")
    }
}
