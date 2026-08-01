//! RFC 7578 body splitting.
//!
//! # The delimiter rule that matters
//!
//! The delimiter is CRLF + `--` + boundary. That leading CRLF belongs to the
//! *delimiter*, not to the part before it, so a part body must never keep it. Off
//! by those two bytes and every uploaded file is silently corrupted — a JPEG that
//! still opens, a checksum that never matches.
//!
//! The final delimiter carries a trailing `--`. Its absence means the body was
//! truncated in transit, which is reported rather than guessed at.

use super::multipart_headers::{self, Headers};

/// One decoded part: its headers and its exact body bytes.
pub(super) struct Part {
    pub(super) headers: Headers,
    pub(super) body: String,
}

/// Split a multipart body into parts.
///
/// # Arguments
///
/// * `body` — The raw request body.
/// * `boundary` — The boundary from the `Content-Type` header, without `--`.
///
/// # Returns
///
/// The parts in transmission order. A body whose only content is the final
/// delimiter yields an empty list.
///
/// # Errors
///
/// Returns an error when the opening delimiter is missing, when a part has no
/// CRLF CRLF header separator, or when the closing `--` delimiter never arrives.
pub(super) fn split(body: &str, boundary: &str) -> Result<Vec<Part>, String> {
    if boundary.is_empty() {
        return Err("multipart_parse: boundary must not be empty".to_string());
    }
    let dash = format!("--{boundary}");
    // The preamble before the first delimiter is discarded per RFC 2046, but a
    // body that never opens a part is malformed rather than empty.
    let start = body
        .find(&dash)
        .ok_or_else(|| format!("multipart_parse: body has no `{dash}` delimiter"))?;

    let delimiter = format!("\r\n{dash}");
    let mut rest = &body[start + dash.len()..];
    let mut parts = Vec::new();
    loop {
        // A `--` suffix marks the closing delimiter. Any epilogue after it is
        // ignored per RFC 2046 rather than treated as another part.
        if rest.starts_with("--") {
            return Ok(parts);
        }
        let rest_body = rest
            .strip_prefix("\r\n")
            .ok_or_else(|| "multipart_parse: delimiter is not followed by CRLF".to_string())?;
        let end = rest_body.find(&delimiter).ok_or_else(|| {
            "multipart_parse: body ends without the closing `--` delimiter".to_string()
        })?;
        parts.push(part(&rest_body[..end])?);
        rest = &rest_body[end + delimiter.len()..];
    }
}

/// Split one part into its header block and its exact body.
fn part(section: &str) -> Result<Part, String> {
    let (block, body) = section
        .split_once("\r\n\r\n")
        .ok_or_else(|| "multipart_parse: part has no CRLF CRLF header separator".to_string())?;
    Ok(Part {
        headers: multipart_headers::parse(block),
        // `body` is already exact: the trailing CRLF was consumed as part of the
        // delimiter search, never included here.
        body: body.to_string(),
    })
}
