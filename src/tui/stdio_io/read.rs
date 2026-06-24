//! Read one JSON message from stdin.

use std::io::{self, BufRead};

use crate::value::Value;

use super::{decode, header};

pub(super) fn value() -> Result<Value, String> {
    let mut input = io::stdin().lock();
    let mut first = String::new();
    let bytes = input
        .read_line(&mut first)
        .map_err(|error| format!("stdio_read: {error}"))?;
    if bytes == 0 {
        return Ok(Value::Nil);
    }
    if let Some(length) = header::content_length(&first)? {
        return frame(&mut input, length);
    }
    decode::line(&first)
}

fn frame<R: BufRead>(input: &mut R, mut length: usize) -> Result<Value, String> {
    loop {
        let mut line = String::new();
        let bytes = input
            .read_line(&mut line)
            .map_err(|error| format!("stdio_read: {error}"))?;
        if bytes == 0 {
            return Err("stdio_read: unexpected EOF in headers".into());
        }
        if header::is_blank(&line) {
            break;
        }
        if let Some(next) = header::content_length(&line)? {
            length = next;
        }
    }
    let mut body = vec![0; length];
    input
        .read_exact(&mut body)
        .map_err(|error| format!("stdio_read: unexpected EOF in body: {error}"))?;
    decode::body(&body)
}
