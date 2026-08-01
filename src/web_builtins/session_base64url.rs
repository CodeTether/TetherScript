//! Unpadded base64url for session cookie values.
//!
//! `crate::system::base64_encode_bytes` uses the standard alphabet with `=`
//! padding, and all three of `+`, `/`, and `=` need escaping inside a cookie
//! value, so the URL-safe alphabet is applied here instead.
//!
//! The `jwt` and `csrf` groups each carry an equivalent codec, but both scope it
//! `pub(super)` to their own group, so neither is reachable from `session`.
//! Consolidating the three needs whoever owns all of those files.

/// Encode bytes as unpadded base64url.
///
/// # Arguments
///
/// * `bytes` — Input bytes.
///
/// # Returns
///
/// A base64url string using `-` and `_`, with no `=` padding.
pub(super) fn encode(bytes: &[u8]) -> String {
    let standard = crate::system::base64_encode_bytes(bytes);
    standard
        .trim_end_matches('=')
        .chars()
        .map(|character| match character {
            '+' => '-',
            '/' => '_',
            other => other,
        })
        .collect()
}

/// Decode unpadded base64url.
///
/// # Arguments
///
/// * `input` — base64url text, without padding.
///
/// # Returns
///
/// The decoded bytes.
///
/// # Errors
///
/// Returns an error when `input` contains a character outside the base64url
/// alphabet, including the standard-alphabet `+`, `/`, and `=`. A value carrying
/// those was not produced by [`encode`], so accepting it would blur which
/// encoder wrote the cookie.
pub(super) fn decode(label: &str, input: &str) -> Result<Vec<u8>, String> {
    if let Some(bad) = input
        .chars()
        .find(|character| matches!(character, '+' | '/' | '='))
    {
        return Err(format!(
            "session: {label} must be unpadded base64url; found `{bad}`"
        ));
    }
    let mut standard: String = input
        .chars()
        .map(|character| match character {
            '-' => '+',
            '_' => '/',
            other => other,
        })
        .collect();
    while !standard.len().is_multiple_of(4) {
        standard.push('=');
    }
    crate::system::base64_decode_bytes(&standard)
        .map_err(|error| format!("session: {label} is not valid base64url: {error}"))
}
