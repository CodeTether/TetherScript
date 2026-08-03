//! Minting of a signed, expiring state value.
//!
//! The threat model — why state must be signed and why it must expire — is in
//! [`super`]. This file is only the construction.

use std::rc::Rc;

use super::super::super::hmac::hmac_sha256;
use super::super::clock::now_secs;
use super::super::codec::encode;
use super::super::entropy::bytes;
use super::super::return_to::validate;
use super::codec::{nonce_hex, render};
use super::Claims;
use crate::value::Value;

/// Mint a signed, expiring state carrying `return_to`.
///
/// # Arguments
///
/// * `secret` — HMAC key. Must be the same value verification is given.
/// * `ttl_seconds` — Lifetime in seconds; must be positive.
/// * `return_to` — Relative destination path, validated by [`validate`].
///
/// # Returns
///
/// `Ok` of a `Value::Str` holding `payload.signature`, both unpadded base64url, so
/// the value is safe unescaped in a URL query parameter.
///
/// # Errors
///
/// Returns `Err` when `ttl_seconds` is not positive, or when `return_to` is not a
/// safe relative path — an absolute URL, a scheme-relative `//host`, a backslash
/// form, or a control character.
pub(crate) fn token(secret: &str, ttl_seconds: i64, return_to: &str) -> Result<Value, String> {
    if ttl_seconds <= 0 {
        return Err(format!(
            "oauth_state_token: bad ttl_secs `{ttl_seconds}`; must be a positive number of seconds"
        ));
    }
    let now = now_secs();
    let claims = Claims {
        nonce: nonce_hex(&bytes(16)),
        issued_at: now,
        expires_at: now.saturating_add(ttl_seconds),
        return_to: validate(return_to)?,
    };
    let payload = encode(render(&claims).as_bytes());
    let signature = encode(&hmac_sha256(secret.as_bytes(), payload.as_bytes()));
    Ok(Value::Str(Rc::new(format!("{payload}.{signature}"))))
}
