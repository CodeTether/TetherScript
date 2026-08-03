//! Algorithm-name policy for the RSA JWS family.
//!
//! # Security
//!
//! This file is the whole reason the group refuses to be helpful. It answers one
//! question — "is this name in the allow-list?" — and it never answers the
//! question a forger wants answered, which is "which verifier should I run for
//! this token?"
//!
//! Dispatching on a token's own `alg` is the classic JWT forgery. Two variants:
//!
//! * `{"alg":"none"}` — the unsecured JWS. A verifier that switches on `alg`
//!   selects the no-op verifier and accepts an unsigned token.
//! * `{"alg":"HS256"}` against an RS256 deployment — the verifier reaches for
//!   HMAC, uses the *public* key as the shared secret, and since the public key
//!   is published, the attacker can compute that MAC too.
//!
//! Therefore: the caller decides it is doing RSA verification, and this
//! allow-list only confirms the token does not *contradict* that decision. The
//! returned name selects the digest inside an already-chosen RSA verifier; it
//! never selects the signature scheme.

/// RSASSA-PKCS1-v1_5 names this group is willing to see in a header.
const RSA_ALGS: [&str; 3] = ["RS256", "RS384", "RS512"];

/// Require that `alg` names a supported RSA algorithm.
///
/// # Arguments
///
/// * `alg` — The `alg` value read from an unverified header.
/// * `label` — Qualified name used in error text.
///
/// # Returns
///
/// The accepted algorithm name, echoed back so a caller can pick a digest.
///
/// # Errors
///
/// Returns a named error for `none`, for any HMAC or ECDSA name, and for any
/// unregistered string. `none` gets its own message because it is an attack, not
/// a configuration mistake.
///
/// # Examples
///
/// ```tether
/// let forged = "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.e30.AA"
/// println(jwt_rs256_parts(forged).err())   // ...alg `none` is never accepted...
/// ```
pub(super) fn require_rsa_alg(alg: &str, label: &str) -> Result<String, String> {
    if alg == "none" {
        return Err(format!(
            "{label}: alg `none` is never accepted; an unsigned token is not a token"
        ));
    }
    if RSA_ALGS.contains(&alg) {
        return Ok(alg.to_string());
    }
    Err(format!(
        "{label}: unsupported alg `{alg}`; expected one of {}",
        RSA_ALGS.join(", ")
    ))
}
