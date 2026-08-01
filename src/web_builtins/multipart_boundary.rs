//! `Content-Type` boundary extraction and part-header parsing.
//!
//! The sibling `mime` group parses `Content-Type` correctly, including quoted
//! parameters, but its `parse` is `pub(super)` to that group and so unreachable
//! from here. Rather than widen another owner's visibility, this module reuses the
//! same rule: split on `;` only outside quotes, because an RFC 2046 boundary may
//! legally contain a semicolon and splitting first would truncate it.

/// Extract the `boundary` parameter from a `Content-Type` header.
///
/// # Arguments
///
/// * `header` — Header value such as `multipart/form-data; boundary="ab;cd"`.
///
/// # Returns
///
/// The boundary with surrounding quotes removed.
///
/// # Errors
///
/// Returns an error when no `boundary` parameter is present, or when it is
/// present but empty. A blank boundary would make every delimiter match, so it is
/// rejected rather than allowed to silently shred the body.
pub(super) fn boundary(header: &str) -> Result<String, String> {
    for part in split_unquoted(header).into_iter().skip(1) {
        let Some((name, value)) = part.split_once('=') else {
            continue;
        };
        if !name.trim().eq_ignore_ascii_case("boundary") {
            continue;
        }
        let value = unquote(value.trim());
        if value.is_empty() {
            return Err("multipart_boundary: boundary parameter is empty".to_string());
        }
        return Ok(value);
    }
    Err(format!(
        "multipart_boundary: header declares no boundary parameter: `{header}`"
    ))
}

/// Split on `;` while treating a double-quoted run as opaque.
pub(super) fn split_unquoted(header: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for ch in header.chars() {
        match ch {
            '"' => {
                quoted = !quoted;
                current.push(ch);
            }
            ';' if !quoted => parts.push(std::mem::take(&mut current)),
            _ => current.push(ch),
        }
    }
    parts.push(current);
    parts
}

/// Strip surrounding double quotes and unescape `\"` inside a quoted string.
pub(super) fn unquote(value: &str) -> String {
    let inner = value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(value);
    inner.replace("\\\"", "\"")
}
