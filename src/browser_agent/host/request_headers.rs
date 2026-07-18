//! Bounded HTTP header extraction for browser host requests.

use std::io::{BufRead, BufReader};
use std::net::TcpStream;

const MAX_BODY: usize = 1024 * 1024;

pub(super) fn content_length(reader: &mut BufReader<&mut TcpStream>) -> Result<usize, String> {
    let mut length = None;
    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|error| error.to_string())?;
        if line == "\r\n" || line.is_empty() {
            break;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = Some(value.trim().parse::<usize>().map_err(|_| "bad length")?);
        }
    }
    match length {
        Some(value) if value <= MAX_BODY => Ok(value),
        Some(_) => Err("browser host: request body too large".into()),
        None => Err("browser host: missing content-length".into()),
    }
}
