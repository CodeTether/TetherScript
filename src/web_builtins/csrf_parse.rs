//! Payload parsing, split from construction so each file owns one direction.

use super::csrf_payload::{Claims, VERSION};

/// Parse the decoded payload text into typed claims.
///
/// # Arguments
///
/// * `text` — Decoded payload, expected as `v1.<nonce>.<iat>.<exp>`.
///
/// # Returns
///
/// The parsed claims.
///
/// # Errors
///
/// Returns an error naming the specific defect: a field count other than four, an
/// unknown version, an empty nonce, or a non-numeric timestamp.
pub(super) fn parse(text: &str) -> Result<Claims, String> {
    let fields: Vec<&str> = text.split('.').collect();
    if fields.len() != 4 {
        return Err(format!(
            "csrf: malformed payload; expected 4 fields, got {}",
            fields.len()
        ));
    }
    if fields[0] != VERSION {
        return Err(format!(
            "csrf: malformed payload; unsupported version `{}`",
            fields[0]
        ));
    }
    if fields[1].is_empty() {
        return Err("csrf: malformed payload; nonce is empty".into());
    }
    Ok(Claims {
        nonce: fields[1].to_string(),
        issued_at: number("iat", fields[2])?,
        expires_at: number("exp", fields[3])?,
    })
}

fn number(field: &str, text: &str) -> Result<i64, String> {
    text.parse()
        .map_err(|_| format!("csrf: malformed payload; {field} `{text}` is not a number"))
}
