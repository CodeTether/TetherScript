//! Vault HTTP response reader.

use std::io::Read;

use super::body;

pub(super) fn read(stream: &mut dyn Read) -> Result<String, String> {
    let mut bytes = Vec::new();
    stream
        .take(2 * 1024 * 1024)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("vault: read response failed: {error}"))?;
    let head_end = header_end(&bytes).ok_or("vault: malformed HTTP response")?;
    let head = std::str::from_utf8(&bytes[..head_end])
        .map_err(|error| format!("vault: response headers were not UTF-8: {error}"))?;
    let status = status_code(head)?;
    let body = body::decode(head, &bytes[head_end + 4..])?;
    let body = String::from_utf8(body)
        .map_err(|error| format!("vault: response body was not UTF-8: {error}"))?;
    if !(200..300).contains(&status) {
        return Err(format!("vault: HTTP {status}: {}", body.trim()));
    }
    Ok(body)
}

fn header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn status_code(head: &str) -> Result<u16, String> {
    let line = head
        .lines()
        .next()
        .ok_or_else(|| "vault: missing status line".to_string())?;
    let mut parts = line.split_whitespace();
    match (parts.next(), parts.next()) {
        (Some(version), Some(code)) if version.starts_with("HTTP/") => code
            .parse()
            .map_err(|_| format!("vault: invalid status code in {line:?}")),
        _ => Err(format!("vault: invalid status line {line:?}")),
    }
}
