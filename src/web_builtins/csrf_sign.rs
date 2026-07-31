//! Signing, verification, and unverified inspection.
//!
//! A token is `<base64url(payload)>.<base64url(hmac)>`. The signature covers the
//! encoded payload exactly as transmitted, so verification never re-renders the
//! payload and cannot disagree with the signer about spacing or field order.

use std::rc::Rc;

use crate::value::Value;

// The HMAC group re-exports both primitives `pub(crate)`, so this group reuses
// them rather than repeating the construction. `csrf` and `hmac` are siblings
// under `web_builtins`, hence the two `super`s.
use super::super::hmac::{constant_time_eq, hmac_sha256};
use super::csrf_base64url::encode;
use super::csrf_base64url_decode::decode;
use super::csrf_claims::claims_map;
use super::csrf_parse::parse;
use super::csrf_payload::{build, now_secs, render};

/// Mint a signed token valid for `ttl_seconds`.
///
/// # Errors
///
/// Returns an error when `ttl_seconds` is not positive; a token that is already
/// expired at issue is a caller mistake, not a token.
pub(super) fn token(secret: &str, ttl_seconds: i64) -> Result<Value, String> {
    if ttl_seconds <= 0 {
        return Err(format!(
            "csrf_token: bad ttl `{ttl_seconds}`; must be a positive number of seconds"
        ));
    }
    let payload = encode(render(&build(ttl_seconds)).as_bytes());
    let signature = encode(&hmac_sha256(secret.as_bytes(), payload.as_bytes()));
    Ok(Value::Str(Rc::new(format!("{payload}.{signature}"))))
}

/// Verify a token's signature, then its expiry.
///
/// # Returns
///
/// `false` for a correctly signed token that has expired. Expiry is an expected
/// outcome, not a failure, so callers can distinguish "try again" from "tampered".
///
/// # Errors
///
/// Returns an error for a malformed token, bad base64url, a bad signature, or an
/// unparseable payload.
pub(super) fn verify(token: &str, secret: &str) -> Result<Value, String> {
    let (payload, signature) = split(token)?;
    let expected = hmac_sha256(secret.as_bytes(), payload.as_bytes());
    let presented = decode("signature", signature)?;
    // Constant-time: never early-exit on the first differing byte.
    if !constant_time_eq(&expected, &presented) {
        return Err("csrf_verify: bad signature".into());
    }
    let claims = parse(&payload_text(payload)?)?;
    Ok(Value::Bool(now_secs() < claims.expires_at))
}

/// Decode a token's claims **without** verifying its signature.
///
/// # Returns
///
/// A map of `nonce`, `iat`, and `exp`. The values are untrusted: anyone can author
/// them, which is why the builtin is named `csrf_claims` and never `csrf_check`.
///
/// # Errors
///
/// Returns an error for a malformed token, bad base64url, or an unparseable
/// payload. The signature is deliberately not examined.
pub(super) fn claims(token: &str) -> Result<Value, String> {
    let (payload, _) = split(token)?;
    claims_map(&payload_text(payload)?)
}

/// Split a token into its payload and signature segments.
fn split(token: &str) -> Result<(&str, &str), String> {
    let mut parts = token.split('.');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(payload), Some(signature), None) if !payload.is_empty() && !signature.is_empty() => {
            Ok((payload, signature))
        }
        _ => Err(format!(
            "csrf: malformed token; expected 2 base64url segments, got {}",
            token.split('.').count()
        )),
    }
}

fn payload_text(payload: &str) -> Result<String, String> {
    let bytes = decode("payload", payload)?;
    String::from_utf8(bytes).map_err(|_| "csrf: malformed payload; not valid UTF-8".to_string())
}
