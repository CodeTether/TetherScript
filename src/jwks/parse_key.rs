//! Validation of one JWK into an [`RsaPublicKey`].
//!
//! One responsibility: wire the field readers, the base64url decoder, and the
//! acceptance rules together for a single `keys` array element. No policy is
//! decided here; every refusal comes from a neighbouring module.
//!
//! # What is refused here versus at selection time
//!
//! This file applies the **absolute** rules — the ones that hold regardless of
//! what the caller later asks for: `kty` must be `RSA`, `n` and `e` must be
//! present and well-formed, and `use`/`key_ops` must permit verification. A key
//! failing any of these can never be used to verify any token, so it is dropped at
//! parse time and can never reach a verifier by any path.
//!
//! The one **relative** rule — a JWK `alg` contradicting the *requested*
//! algorithm — cannot be decided here, because the request is not known yet. It
//! lives in [`crate::jwks::select`].

use crate::jwks::alg::SigAlg;
use crate::jwks::base64url::decode;
use crate::jwks::fields::{opt_str, req_str};
use crate::jwks::key::RsaPublicKey;
use crate::jwks::key_ops::opt_key_ops;
use crate::jwks::{exponent, modulus, usage};
use crate::value::Value;

/// Validate one JWK.
///
/// # Arguments
///
/// * `jwk` — One element of the `keys` array, already JSON-decoded.
/// * `label` — Locating name used in error text, such as `jwks: keys[0]`.
///
/// # Returns
///
/// The validated key. Its `modulus` and `exponent` are minimal big-endian bytes.
///
/// # Errors
///
/// Returns a named error, which the caller records as a skip reason rather than a
/// document failure, when the entry is not an object, declares a `kty` other than
/// `RSA` (including `oct`, which must never be read as RSA), is missing `n` or
/// `e`, encodes either in anything but strict unpadded base64url, or fails the
/// modulus, exponent, or usage rules.
///
/// # Panics
///
/// Does not panic.
pub(crate) fn parse_key(jwk: &Value, label: &str) -> Result<RsaPublicKey, String> {
    let kty = req_str(jwk, "kty", label)?;
    if kty != "RSA" {
        return Err(format!(
            "{label}: unsupported kty `{kty}`; only RSA public keys are implemented"
        ));
    }
    let key_ops = opt_key_ops(jwk, label)?;
    let use_member = opt_str(jwk, "use", label)?;
    usage::check(use_member.as_deref(), key_ops.as_deref(), label)?;
    let modulus_bytes = decode(&format!("{label}.n"), &req_str(jwk, "n", label)?)?;
    let exponent_bytes = decode(&format!("{label}.e"), &req_str(jwk, "e", label)?)?;
    let modulus_bits = modulus::check(&modulus_bytes, label)?;
    exponent::check(&exponent_bytes, label)?;
    Ok(RsaPublicKey {
        kid: opt_str(jwk, "kid", label)?,
        modulus: modulus_bytes,
        exponent: exponent_bytes,
        modulus_bits,
        alg: declared_alg(jwk, label)?,
        key_ops,
    })
}

/// Read the optional `alg`, refusing one this module cannot honour.
fn declared_alg(jwk: &Value, label: &str) -> Result<Option<SigAlg>, String> {
    match opt_str(jwk, "alg", label)? {
        None => Ok(None),
        Some(name) => SigAlg::parse(&name)
            .map(Some)
            .map_err(|error| format!("{label}: {error}")),
    }
}
