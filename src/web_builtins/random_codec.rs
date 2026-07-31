//! Token encoding and argument validation for the `random_*` built-ins.

use crate::value::Value;

use super::random_source::MAX_BYTES;

/// URL-safe base64 alphabet (RFC 4648 §5), with `-` and `_` for `+` and `/`.
const URL_SAFE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encode bytes as unpadded URL-safe base64.
///
/// Padding is omitted so a token is safe in a path segment, a query value, or a
/// cookie without further escaping: `=` would otherwise need encoding.
///
/// # Arguments
///
/// * `bytes` — Raw bytes to encode.
///
/// # Returns
///
/// A string containing only `A-Z`, `a-z`, `0-9`, `-`, and `_`.
pub(super) fn base64_url(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        out.push(URL_SAFE[(b0 >> 2) as usize] as char);
        out.push(URL_SAFE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(URL_SAFE[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(URL_SAFE[(b2 & 0x3f) as usize] as char);
        }
    }
    out
}

/// Validate a requested byte count.
///
/// # Arguments
///
/// * `value` — Script-supplied argument.
/// * `label` — Built-in name, used in the error message.
///
/// # Returns
///
/// The count as a `usize`.
///
/// # Errors
///
/// Returns a named error when the value is not an int, is zero or negative, or
/// exceeds the documented cap.
pub(super) fn byte_count(value: &Value, label: &str) -> Result<usize, String> {
    let Value::Int(count) = value else {
        return Err(format!("{label}: n must be int, got {}", value.type_name()));
    };
    if *count <= 0 {
        return Err(format!("{label}: n must be positive, got {count}"));
    }
    if *count > MAX_BYTES {
        return Err(format!(
            "{label}: n must be at most {MAX_BYTES}, got {count}"
        ));
    }
    Ok(*count as usize)
}
