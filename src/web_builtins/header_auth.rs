//! `Authorization` scheme parsing.
//!
//! Only the `Bearer` scheme is recognized. A bare token with no scheme is
//! rejected on purpose: accepting it would let a caller present an opaque string
//! that some other layer might interpret as a different credential type, and it
//! silently diverges from RFC 6750, which requires the scheme.

use std::collections::HashMap;

use super::header_lookup::find;
use crate::value::Value;

/// Extract the token from an `Authorization: Bearer <token>` header.
///
/// # Arguments
///
/// * `headers` — Header map.
///
/// # Returns
///
/// The token with surrounding whitespace removed.
///
/// # Errors
///
/// Returns an error naming the problem when the header is absent, carries a
/// scheme other than `Bearer`, has no scheme at all, or has an empty token.
pub(super) fn bearer(headers: &HashMap<String, Value>) -> Result<Value, String> {
    let header =
        find(headers, "authorization").ok_or("bearer_token: no Authorization header is present")?;

    // Split on the first run of whitespace: `Bearer <token>` per RFC 6750.
    let (scheme, token) = header
        .split_once(char::is_whitespace)
        .ok_or("bearer_token: Authorization header has no scheme; expected `Bearer <token>`")?;

    if !scheme.eq_ignore_ascii_case("bearer") {
        return Err(format!(
            "bearer_token: unsupported Authorization scheme `{scheme}`; expected Bearer"
        ));
    }

    let token = token.trim();
    if token.is_empty() {
        return Err("bearer_token: Bearer scheme carries an empty token".into());
    }
    Ok(Value::Str(std::rc::Rc::new(token.to_string())))
}
