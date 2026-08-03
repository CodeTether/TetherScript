//! The two token-facing built-ins: unverified header, and RS256 signature parts.
//!
//! One responsibility: shape what a caller needs *before* verification. Splitting
//! and decoding live in `super::jwks_token`; algorithm policy lives in
//! `super::jwks_alg`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

use super::jwks_alg::require_rsa_alg;
use super::jwks_base64url::decode;
use super::jwks_field::req_str;
use super::jwks_token::{header_object, split};

/// Decode a token's header **without verifying anything**.
///
/// # Arguments
///
/// * `token` — Compact serialization to inspect.
///
/// # Returns
///
/// The decoded header map, exactly as the token author wrote it.
///
/// # Errors
///
/// Returns a named error for a token that is not three non-empty segments, whose
/// header is not strict base64url, is not UTF-8, is not JSON, or is not a JSON
/// object.
///
/// # Security
///
/// **The result is unverified and must never be trusted for an authorization
/// decision.** No signature is checked, so every member — `alg`, `kid`, and any
/// custom claim someone put in the header — is attacker-controlled. The single
/// legitimate use is key selection: read `kid`, pass it to `jwks_find`, and let
/// an actual verifier decide whether the token is genuine. In particular, do not
/// read `alg` here and use it to choose a verification algorithm; that is the
/// forgery `super::jwks_alg` exists to prevent.
///
/// # Examples
///
/// ```tether
/// let header = jwt_header(token).unwrap()   // UNVERIFIED
/// let key = jwks_find(jwks_parse(certs_json).unwrap(), header.kid).unwrap()
/// ```
pub(super) fn header(token: &str) -> Result<Value, String> {
    header_object("jwt_header", token)
}

/// Extract the bytes an RSA verifier needs from a token.
///
/// # Arguments
///
/// * `token` — Compact serialization to prepare for verification.
///
/// # Returns
///
/// A map with `signing_input` (the ASCII bytes of `header.payload`, which is what
/// RFC 7515 signs), `signature` (decoded signature bytes), `alg` (the confirmed
/// RSA algorithm name), and `kid` (nil when the header omits it).
///
/// # Errors
///
/// Returns a named error for a malformed token, an unreadable header, `alg`
/// missing, `alg` equal to `none`, or any `alg` outside RS256/RS384/RS512.
///
/// # Security
///
/// The returned `alg` selects a *digest* inside an RSA verification the caller
/// already decided to perform. It must never select the signature scheme itself:
/// honouring a token's own choice lets an attacker pick `none`, or pick HS256 and
/// forge a MAC with the published public key as the shared secret.
///
/// # Examples
///
/// ```tether
/// let parts = jwt_rs256_parts(token).unwrap()
/// println(parts.alg)                          // RS256
/// println(str(parts.signature.len()))
/// ```
pub(super) fn rs256_parts(token: &str) -> Result<Value, String> {
    let label = "jwt_rs256_parts";
    let (header_segment, payload_segment, signature_segment) = split(label, token)?;
    let decoded = header_object(label, token)?;
    let alg = require_rsa_alg(&req_str(&decoded, "alg", "jwt_rs256_parts: header")?, label)?;
    let signature = decode(&format!("{label}: signature"), signature_segment)?;
    let signing_input = format!("{header_segment}.{payload_segment}");

    let mut out = HashMap::new();
    out.insert("alg".into(), Value::Str(Rc::new(alg)));
    out.insert("kid".into(), kid_of(&decoded));
    out.insert(
        "signing_input".into(),
        Value::Bytes(Rc::new(RefCell::new(signing_input.into_bytes()))),
    );
    out.insert(
        "signature".into(),
        Value::Bytes(Rc::new(RefCell::new(signature))),
    );
    Ok(Value::Map(Rc::new(RefCell::new(out))))
}

/// Read `kid` if the header has one, otherwise `nil`.
///
/// A missing `kid` is not an error: single-key issuers omit it.
fn kid_of(header: &Value) -> Value {
    let Value::Map(fields) = header else {
        return Value::Nil;
    };
    match fields.borrow().get("kid") {
        Some(Value::Str(text)) => Value::Str(text.clone()),
        _ => Value::Nil,
    }
}
