//! Verification of a signed OAuth state received on the callback.
//!
//! The threat model is in [`super`]. This file is the checking half, split out so
//! minting and verification are independently readable.
//!
//! # Order of operations is load-bearing
//!
//! The signature is checked **before** the payload is interpreted. A verifier that parses
//! first and authenticates second runs its parser on attacker-chosen bytes on every
//! request; here the parser only ever sees bytes that already carry a valid MAC.
//!
//! # Distinct errors, distinct responses
//!
//! `bad signature` means forgery or the wrong secret and should be treated as an attack.
//! `expired state` means the user was simply slow, and the flow should restart.
//! `malformed state` means the value did not come from here at all. Collapsing these into
//! one message loses an operator's ability to tell an attack from a slow login.
//!
//! The expiry message carries the nonce, which is the only stable handle on a stateless
//! token; without it two concurrent expired logins are indistinguishable in a log. The
//! token is already dead by the time it is printed, so the nonce is no longer a secret.

use std::rc::Rc;

use super::super::super::hmac::{constant_time_eq, hmac_sha256};
use super::super::clock::now_secs;
use super::super::codec::decode::decode;
use super::super::return_to::validate;
use super::codec::parse;
use crate::value::Value;

/// Verify a state and recover its return path.
///
/// # Arguments
///
/// * `secret` — The same HMAC key used to mint the state.
/// * `state` — The value received on the callback, entirely untrusted.
///
/// # Returns
///
/// `Ok` of a `Value::Str` holding the validated relative return path.
///
/// # Errors
///
/// Returns a named `Err` for a malformed value, a bad signature, or an expired state. The
/// return path is revalidated on the way out by [`validate`], so even a state signed with
/// a leaked secret cannot smuggle an off-origin redirect.
pub(crate) fn verify(secret: &str, state: &str) -> Result<Value, String> {
    let (payload, signature) = split(state)?;
    let expected = hmac_sha256(secret.as_bytes(), payload.as_bytes());
    if !constant_time_eq(&expected, &decode("state signature", signature)?) {
        return Err("oauth_state_verify: bad signature".into());
    }
    let claims = parse(&text(payload)?)?;
    let now = now_secs();
    if now >= claims.expires_at {
        return Err(format!(
            "oauth_state_verify: expired state; nonce {} was minted {}s ago and was valid until {}",
            claims.nonce,
            now - claims.issued_at,
            claims.expires_at
        ));
    }
    Ok(Value::Str(Rc::new(validate(&text(&claims.return_to)?)?)))
}

/// Split a state into its payload and signature segments.
fn split(state: &str) -> Result<(&str, &str), String> {
    let mut parts = state.split('.');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(payload), Some(signature), None) if !payload.is_empty() && !signature.is_empty() => {
            Ok((payload, signature))
        }
        _ => Err(format!(
            "oauth_state_verify: malformed state; expected 2 base64url segments, got {}",
            state.split('.').count()
        )),
    }
}

/// Decode a base64url segment to UTF-8 text.
fn text(segment: &str) -> Result<String, String> {
    let raw = decode("state payload", segment)?;
    String::from_utf8(raw)
        .map_err(|_| "oauth_state_verify: malformed state; payload is not valid UTF-8".to_string())
}
