//! PHC-style encoding and parsing of a stored hash.
//!
//! The stored form is self-describing:
//!
//! ```text
//! $pbkdf2-sha256$i=600000$<base64 salt>$<base64 hash>
//! ```
//!
//! Recording the algorithm and iteration count *inside* the string is what lets
//! the cost be raised later without invalidating existing credentials: an old
//! hash still verifies at its own recorded count, and
//! `password_needs_rehash` reports that it should be upgraded on next login.

use crate::system::{base64_decode_bytes, base64_encode_bytes};

/// Algorithm identifier for the only scheme this module implements.
pub(super) const ALGORITHM: &str = "pbkdf2-sha256";

/// A parsed stored hash.
pub(super) struct Stored {
    pub(super) iterations: u32,
    pub(super) salt: Vec<u8>,
    pub(super) hash: Vec<u8>,
}

/// Render a stored hash in PHC-style form.
pub(super) fn encode(iterations: u32, salt: &[u8], hash: &[u8]) -> String {
    format!(
        "${ALGORITHM}$i={iterations}${}${}",
        base64_encode_bytes(salt),
        base64_encode_bytes(hash)
    )
}

/// Parse a stored hash, naming whatever is wrong with it.
///
/// # Errors
///
/// Returns an error identifying the specific defect: a malformed field layout, an
/// unknown algorithm, a missing or non-numeric iteration count, or salt/hash text
/// that is not valid base64.
pub(super) fn parse(encoded: &str) -> Result<Stored, String> {
    // Leading `$` yields an empty first field, so five parts are expected.
    let parts: Vec<&str> = encoded.split('$').collect();
    if parts.len() != 5 || !parts[0].is_empty() {
        return Err(format!(
            "password: malformed encoding; expected `${ALGORITHM}$i=<n>$<salt>$<hash>`, got {} field(s)",
            parts.len()
        ));
    }
    if parts[1] != ALGORITHM {
        return Err(format!(
            "password: unknown algorithm `{}`; only `{ALGORITHM}` is supported",
            parts[1]
        ));
    }
    Ok(Stored {
        iterations: iterations(parts[2])?,
        salt: decode_field("salt", parts[3])?,
        hash: decode_field("hash", parts[4])?,
    })
}

/// Parse the `i=<n>` iteration field.
fn iterations(field: &str) -> Result<u32, String> {
    let digits = field.strip_prefix("i=").ok_or_else(|| {
        format!("password: missing iteration field; expected `i=<n>`, got `{field}`")
    })?;
    let parsed: u32 = digits
        .parse()
        .map_err(|_| format!("password: non-numeric iteration count `{digits}`"))?;
    if parsed == 0 {
        return Err("password: iteration count must be at least 1".into());
    }
    Ok(parsed)
}

/// Decode one base64 field, naming which field failed.
fn decode_field(label: &str, field: &str) -> Result<Vec<u8>, String> {
    base64_decode_bytes(field)
        .map_err(|error| format!("password: {label} is not valid base64: {error}"))
}
