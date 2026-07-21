//! HTTP/1.1 request parsing shared by plain and TLS listeners.

use std::io::{BufRead, Read};

use crate::value::Value;

use super::{http_server_headers, http_server_reader, http_server_request_map};

const MAX_START_LINE_BYTES: usize = 8 * 1024;

pub(super) fn parse<R: BufRead>(reader: &mut R) -> Result<Value, String> {
    let line = http_server_reader::line(reader, MAX_START_LINE_BYTES, "start-line")?
        .ok_or_else(|| "empty request".to_string())?;
    let start = line.trim_end_matches(['\r', '\n']);
    let mut parts = start.splitn(3, ' ');
    let method = parts.next().ok_or("missing method")?.to_string();
    let target = parts.next().ok_or("missing target")?.to_string();
    let _version = parts.next().unwrap_or("HTTP/1.1");
    let (headers, content_length) = http_server_headers::read(reader)?;
    let body = read_body(reader, content_length)?;
    Ok(http_server_request_map::build(
        method, target, headers, body,
    ))
}

fn read_body<R: Read>(reader: &mut R, content_length: usize) -> Result<String, String> {
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader
            .read_exact(&mut body)
            .map_err(|error| format!("read body: {error}"))?;
    }
    Ok(String::from_utf8_lossy(&body).into_owned())
}
