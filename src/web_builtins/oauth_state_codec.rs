//! Render and parse the `v1` state payload.
//!
//! Separated from [`super::mint`] and [`super::verify`] so signing and framing are
//! independent concerns: the parser here trusts nothing and never checks a signature,
//! and the signer never looks at field layout. The format itself is documented in
//! [`super`].

use super::super::codec::encode;
use super::{Claims, FIELDS, VERSION};
use crate::system::hex_encode;

/// Serialise claims to the `.`-separated payload text.
///
/// `return_to` is base64url-encoded so a path containing `.` cannot shift field
/// boundaries.
///
/// # Arguments
///
/// * `claims` — The claims to serialise; `return_to` must already be validated.
///
/// # Returns
///
/// The five-field payload text, ready to be base64url-encoded and signed.
pub(crate) fn render(claims: &Claims) -> String {
    format!(
        "{VERSION}.{}.{}.{}.{}",
        claims.nonce,
        claims.issued_at,
        claims.expires_at,
        encode(claims.return_to.as_bytes())
    )
}

/// Format a nonce as lowercase hex.
///
/// Hex rather than base64url because the payload separator must not appear inside a
/// field, and hex is trivially separator-free.
///
/// # Arguments
///
/// * `raw` — Random bytes, 16 in practice.
///
/// # Returns
///
/// `2 * raw.len()` lowercase hex characters.
pub(crate) fn nonce_hex(raw: &[u8]) -> String {
    hex_encode(raw)
}

/// Parse payload text into claims, validating shape but **not** authenticity.
///
/// # Arguments
///
/// * `text` — Decoded payload text, already proven to carry a valid MAC.
///
/// # Returns
///
/// The parsed [`Claims`], with `return_to` still base64url-encoded; the caller
/// decodes and revalidates it.
///
/// # Errors
///
/// Returns `Err` when the field count is wrong, the version tag is unknown, the nonce
/// is empty, or a timestamp is not an integer. Each message names the field.
pub(crate) fn parse(text: &str) -> Result<Claims, String> {
    let fields: Vec<&str> = text.split('.').collect();
    if fields.len() != FIELDS {
        return Err(format!(
            "oauth_state_verify: malformed state; expected {FIELDS} payload fields, got {}",
            fields.len()
        ));
    }
    if fields[0] != VERSION {
        return Err(format!(
            "oauth_state_verify: malformed state; unsupported version `{}`",
            fields[0]
        ));
    }
    if fields[1].is_empty() {
        return Err("oauth_state_verify: malformed state; nonce is empty".into());
    }
    Ok(Claims {
        nonce: fields[1].to_string(),
        issued_at: number("iat", fields[2])?,
        expires_at: number("exp", fields[3])?,
        return_to: fields[4].to_string(),
    })
}

/// Parse one integer field, naming it on failure.
fn number(field: &str, text: &str) -> Result<i64, String> {
    text.parse().map_err(|_| {
        format!("oauth_state_verify: malformed state; {field} `{text}` is not a number")
    })
}
