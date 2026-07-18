//! Minimal bounded HTTP request reader for browser action envelopes.

use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

pub(super) struct Request {
    pub(super) path: String,
    pub(super) body: String,
}

pub(super) fn read(stream: &mut TcpStream) -> Result<Request, String> {
    let mut reader = BufReader::new(stream);
    let mut start = String::new();
    reader
        .read_line(&mut start)
        .map_err(|error| format!("browser host: read request line failed: {}", error))?;
    let mut parts = start.split_whitespace();
    if parts.next() != Some("POST") {
        return Err("browser host: expected POST".into());
    }
    let path = parts
        .next()
        .ok_or_else(|| "browser host: missing request path".to_string())?
        .to_string();
    let length = super::request_headers::content_length(&mut reader)?;
    let mut body = vec![0; length];
    reader
        .read_exact(&mut body)
        .map_err(|error| format!("browser host: read body failed: {}", error))?;
    let body = String::from_utf8(body)
        .map_err(|_| "browser host: request body must be UTF-8".to_string())?;
    Ok(Request { path, body })
}
