//! # RSASSA-PKCS1-v1_5 signature verification
//!
//! One responsibility: turn a signature octet string into a recovered encoded
//! message and hand it to `check_encoding`. This is RFC 8017 section 8.2.2 with
//! the "EMSA-PKCS1-v1_5 encoding, then compare" step restructured as a
//! byte-exact structural walk.
//!
//! ## The three steps
//!
//! 1. **Length check.** `s` must be exactly `k` octets, where `k` is the modulus
//!    octet length. RFC 8017 section 8.2.2 step 1 says so, and it matters:
//!    accepting a shorter string would let one integer be presented at several
//!    lengths, and accepting a longer one would let leading octets be ignored.
//!    Both make signatures non-canonical.
//! 2. **Range check.** `OS2IP(s)` must be strictly less than `n`. `s == n`
//!    reduces to 0 and `s > n` reduces to `s - n`, so without this check a valid
//!    signature gains accepted aliases `s + i * n`, which breaks any downstream
//!    code treating a signature as a unique token — replay caches, revocation
//!    lists, audit logs.
//! 3. **Exponentiation and structural check.** `m = s^e mod n`, encoded back to
//!    exactly `k` octets, then validated octet by octet.
//!
//! ## Verification only
//!
//! There is no signing, no key generation, and no private-key operation anywhere
//! in `src/rsa/`. See `super::key` for why that is deliberate.
//!
//! ## Integration note for the module root
//!
//! `src/rsa/` deliberately contains no `mod.rs`; the integrator writes it. The
//! doc examples in this directory assume the root declares every sibling file as
//! a private `mod` and re-exports exactly `ct_eq`, `DigestAlgorithm`, `RsaError`,
//! `RsaPublicKey`, `MIN_MODULUS_BYTES`, `check_encoding`, and `verify`.
//!
//! # Examples
//!
//! ```rust,no_run
//! use tetherscript::rsa::{verify, DigestAlgorithm, RsaPublicKey};
//!
//! // Real 2048-bit key material and a real signature are needed to succeed, so
//! // this example is compiled but not run. `n` and `e` come from a JWKS entry;
//! // `digest` is SHA-256 over the JWS signing input.
//! let n = std::fs::read("modulus.bin").unwrap();
//! let e = std::fs::read("exponent.bin").unwrap();
//! let signature = std::fs::read("signature.bin").unwrap();
//! let digest = std::fs::read("digest.bin").unwrap();
//!
//! let key = RsaPublicKey::from_be_bytes(&n, &e).unwrap();
//! verify(&signature, &digest, DigestAlgorithm::Sha256, &key).unwrap();
//! ```

use std::cmp::Ordering;

use crate::bigint::BigUint;

use super::digestinfo::DigestAlgorithm;
use super::error::RsaError;
use super::key::RsaPublicKey;
use super::pkcs1::check_encoding;

/// Verify an RSASSA-PKCS1-v1_5 signature over an already-computed digest.
///
/// # Arguments
///
/// * `signature` — the signature octet string, exactly `key.modulus_bytes()`
///   long, most significant octet first.
/// * `digest` — the digest of the signing input, produced by `algorithm`. For
///   JWS `RS256` this is `SHA-256(header_b64 || "." || payload_b64)`.
/// * `algorithm` — the hash the signer claims to have used. Supplied by the
///   caller from its own policy, never read out of the signature.
/// * `key` — a validated public key; see [`RsaPublicKey::new`].
///
/// # Returns
///
/// `Ok(())` only when the signature is byte-exactly the PKCS#1 v1.5 signature of
/// `digest` under `key`.
///
/// # Errors
///
/// [`RsaError::SignatureLength`], [`RsaError::SignatureOutOfRange`],
/// [`RsaError::BigInt`], and every encoding error from
/// [`check_encoding`].
pub fn verify(
    signature: &[u8],
    digest: &[u8],
    algorithm: DigestAlgorithm,
    key: &RsaPublicKey,
) -> Result<(), RsaError> {
    let width = key.modulus_bytes();
    if signature.len() != width {
        return Err(RsaError::SignatureLength {
            got: signature.len(),
            expected: width,
        });
    }
    let value = BigUint::from_be_bytes(signature);
    if value.compare(key.modulus()) != Ordering::Less {
        return Err(RsaError::SignatureOutOfRange);
    }
    let recovered = value.modpow(key.exponent(), key.modulus())?;
    // `recovered < n` so it always fits in `width` octets; `to_be_bytes` pads.
    let em = recovered.to_be_bytes(width)?;
    check_encoding(&em, digest, algorithm)
}
