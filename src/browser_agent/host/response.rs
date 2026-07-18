//! HTTP JSON responses for native browser action calls.

use std::io::Write;
use std::net::TcpStream;

use crate::value::Value;

pub(super) fn write(stream: &mut TcpStream, result: Result<Value, String>) -> Result<(), String> {
    let body = crate::json::encode_to_string(&envelope(result))?;
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
    .map_err(|error| format!("browser host: write response failed: {}", error))
}

fn envelope(result: Result<Value, String>) -> Value {
    match result {
        Ok(value) => super::value::map(vec![("ok", Value::Bool(true)), ("value", value)]),
        Err(error) => super::value::map(vec![
            ("ok", Value::Bool(false)),
            ("error", super::value::string(error)),
        ]),
    }
}
