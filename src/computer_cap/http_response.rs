//! HTTP response reader for computer bridge calls.

use std::io::{BufReader, Read};
use std::net::TcpStream;

pub(crate) fn read_response(stream: TcpStream) -> Result<String, String> {
    let mut reader = BufReader::new(stream);
    let mut text = String::new();
    reader
        .read_to_string(&mut text)
        .map_err(|e| format!("computer bridge: read response failed: {}", e))?;
    let split = text
        .find("\r\n\r\n")
        .ok_or("computer bridge: malformed HTTP response")?;
    let status = text.lines().next().unwrap_or("");
    if !status.contains(" 2") {
        return Err(format!(
            "computer bridge returned {}: {}",
            status,
            &text[split + 4..]
        ));
    }
    Ok(text[split + 4..].to_string())
}
