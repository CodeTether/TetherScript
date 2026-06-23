//! Vault HTTP response body decoding.

pub(super) fn decode(head: &str, body: &[u8]) -> Result<Vec<u8>, String> {
    if has_token(head, "transfer-encoding", "chunked") {
        return chunked(body);
    }
    Ok(body.to_vec())
}

fn has_token(head: &str, name: &str, token: &str) -> bool {
    head.lines().any(|line| {
        let Some((key, value)) = line.split_once(':') else {
            return false;
        };
        key.trim().eq_ignore_ascii_case(name)
            && value
                .split(',')
                .any(|part| part.trim().eq_ignore_ascii_case(token))
    })
}

fn chunked(mut body: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    loop {
        let line_end = crlf(body).ok_or("vault: malformed chunked response")?;
        let line = std::str::from_utf8(&body[..line_end])
            .map_err(|error| format!("vault: invalid chunk size text: {error}"))?;
        let size_text = line.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_text, 16)
            .map_err(|_| format!("vault: invalid chunk size {size_text:?}"))?;
        body = &body[line_end + 2..];
        if size == 0 {
            return Ok(out);
        }
        if body.len() < size + 2 {
            return Err("vault: truncated chunked response".into());
        }
        out.extend_from_slice(&body[..size]);
        if &body[size..size + 2] != b"\r\n" {
            return Err("vault: missing chunk terminator".into());
        }
        body = &body[size + 2..];
    }
}

fn crlf(bytes: &[u8]) -> Option<usize> {
    bytes.windows(2).position(|window| window == b"\r\n")
}
