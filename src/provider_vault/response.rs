//! Vault HTTP response reader.

use std::io::Read;

pub(super) fn read(stream: &mut dyn Read) -> Result<String, String> {
    let mut bytes = Vec::new();
    stream
        .take(2 * 1024 * 1024)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("vault: read response failed: {error}"))?;
    let text = String::from_utf8(bytes)
        .map_err(|error| format!("vault: response was not UTF-8: {error}"))?;
    let (head, body) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| "vault: malformed HTTP response".to_string())?;
    let status = status_code(head)?;
    if !(200..300).contains(&status) {
        return Err(format!("vault: HTTP {status}: {}", body.trim()));
    }
    Ok(body.to_string())
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
