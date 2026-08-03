//! Whether a JWK is permitted to verify signatures.
//!
//! One responsibility: apply the `use` and `key_ops` rules from RFC 7517 §4.2–4.3.
//! No key material is looked at here.
//!
//! # Security: encryption keys are not signing keys
//!
//! A realm may publish an RSA key with `use: "enc"` for JWE key wrapping alongside
//! its `use: "sig"` signing key. Verifying a token against the encryption key is a
//! real attack, not a hygiene issue: the two keys have different lifetimes,
//! different protection, and different exposure, and an encryption key is often
//! reachable by more parties. Treating them as interchangeable means a signature
//! "verified" by a key nobody intended to sign with.
//!
//! The same reasoning applies to `key_ops`. When present it is the *exhaustive*
//! list of permitted operations, so a key whose `key_ops` is `["encrypt"]` or
//! `["sign"]` — note: `sign`, the private-key operation, not `verify` — must not
//! be used to verify.
//!
//! # Absent means unrestricted, present means exhaustive
//!
//! Both members are optional. Absent `use` and absent `key_ops` mean the issuer
//! placed no restriction, so the key is usable; that is what RFC 7517 says and what
//! Keycloak, Auth0, and Google all rely on. Present means exhaustive. This module
//! never *infers* a restriction from silence, and never ignores one that was
//! stated.

/// Reject a key whose declared purpose excludes signature verification.
///
/// # Arguments
///
/// * `use_member` — The JWK `use` member, if present.
/// * `key_ops` — The JWK `key_ops` member, if present.
/// * `label` — Locating name used in error text.
///
/// # Returns
///
/// `Ok(())` when the key may verify signatures.
///
/// # Errors
///
/// Returns a named error when `use` is present and is not `sig`, or when `key_ops`
/// is present and does not contain `verify`.
///
/// # Panics
///
/// Does not panic.
///
/// # Examples
///
/// ```rust
/// use tetherscript::jwks::usage::check;
///
/// let verify = vec!["verify".to_string()];
/// let sign = vec!["sign".to_string()];
/// let empty: Vec<String> = Vec::new();
///
/// assert!(check(None, None, "k").is_ok());
/// assert!(check(Some("sig"), Some(verify.as_slice()), "k").is_ok());
/// assert!(check(Some("enc"), None, "k").unwrap_err().contains("enc"));
/// // `sign` is the private-key operation and does not imply `verify`.
/// assert!(check(None, Some(sign.as_slice()), "k").is_err());
/// // An explicitly empty `key_ops` permits nothing.
/// assert!(check(None, Some(empty.as_slice()), "k").is_err());
/// ```
pub fn check(
    use_member: Option<&str>,
    key_ops: Option<&[String]>,
    label: &str,
) -> Result<(), String> {
    if let Some(purpose) = use_member.filter(|purpose| *purpose != "sig") {
        return Err(format!(
            "{label}: `use` is `{purpose}`, not `sig`, so this key must not verify signatures"
        ));
    }
    let unverifiable = key_ops.filter(|ops| !ops.iter().any(|op| op == "verify"));
    if let Some(ops) = unverifiable {
        return Err(format!(
            "{label}: `key_ops` is [{}] and does not include `verify`",
            ops.join(", ")
        ));
    }
    Ok(())
}
