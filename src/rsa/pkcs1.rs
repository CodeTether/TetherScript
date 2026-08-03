//! # Byte-exact EMSA-PKCS1-v1_5 encoding check
//!
//! One responsibility: given a recovered encoded message, decide whether it is
//! *exactly* the RFC 8017 section 9.2 encoding of the expected digest under the
//! claimed algorithm. The padding head is walked by
//! `super::pkcs1_padding`; this file owns the `DigestInfo` and digest halves,
//! and the total-length arithmetic that makes the check exhaustive.
//!
//! ## Why the check is total, not "starts with"
//!
//! RFC 8017 section 8.2.2 specifies verification as *re-encoding and comparing*,
//! precisely so that no octet of `EM` is left unexamined. This implementation
//! walks the structure instead of re-encoding, so it must account for every
//! octet itself. The remaining two rejections do that:
//!
//! - **`DigestInfo` region not exactly `tLen` octets.** Anything appended after
//!   the digest — even one octet — is trailing garbage. A verifier that stops
//!   after reading the digest lets an attacker append attacker-chosen octets,
//!   which is enough slack for small-exponent forgery constructions. So the
//!   region between the separator and the end of `EM` must have exactly the
//!   length the algorithm dictates.
//! - **`DigestInfo` prefix for a different hash.** If SHA-256 is requested but
//!   the recovered prefix names SHA-1, accepting it means a SHA-1 signature is
//!   honoured as a SHA-256 one, and the caller's algorithm choice is decided by
//!   the attacker. That is algorithm confusion, the RS256/HS256 attack's cousin.
//!
//! The digest itself is compared with [`ct_eq`](crate::rsa::ct_eq); see that
//! module for why
//! a short-circuiting compare leaks.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::rsa::{check_encoding, DigestAlgorithm, RsaError};
//!
//! let digest = [0x11u8; 32];
//! let alg = DigestAlgorithm::Sha256;
//! let mut em = vec![0x00, 0x01];
//! em.extend(vec![0xff; 256 - 3 - alg.encoded_len()]);
//! em.push(0x00);
//! em.extend_from_slice(alg.der_prefix());
//! em.extend_from_slice(&digest);
//! assert_eq!(em.len(), 256);
//! assert_eq!(check_encoding(&em, &digest, alg), Ok(()));
//!
//! // Flip the last octet of the digest inside EM: refused.
//! let mut tampered = em.clone();
//! *tampered.last_mut().unwrap() ^= 0x01;
//! assert_eq!(
//!     check_encoding(&tampered, &digest, alg),
//!     Err(RsaError::DigestMismatch)
//! );
//! ```

use super::ct::ct_eq;
use super::digestinfo::DigestAlgorithm;
use super::error::RsaError;
use super::pkcs1_padding::walk;

/// Verify that `em` is the byte-exact PKCS#1 v1.5 encoding of `digest`.
///
/// # Arguments
///
/// * `em` — the recovered encoded message, at least 2 octets long. Callers get
///   this from [`verify`](crate::rsa::verify), which guarantees the length.
/// * `digest` — the expected message digest.
/// * `alg` — the digest algorithm the caller claims was used.
///
/// # Returns
///
/// `Ok(())` only when every octet of `em` matches the required structure.
///
/// # Errors
///
/// [`RsaError::DigestLength`] when `digest` is not `alg.digest_len()` octets;
/// [`RsaError::EncodingTooShort`] when `em` cannot hold the structure at all;
/// [`RsaError::LeadingBytes`], [`RsaError::PaddingRunTooShort`], and
/// [`RsaError::MissingSeparator`] from the padding walk;
/// [`RsaError::DigestInfoLength`] for a wrong-sized tail;
/// [`RsaError::DigestInfoMismatch`] for the wrong hash's DER prefix; and
/// [`RsaError::DigestMismatch`] when the digests differ.
pub fn check_encoding(em: &[u8], digest: &[u8], alg: DigestAlgorithm) -> Result<(), RsaError> {
    length_guards(em, digest, alg)?;
    let start = walk(em)?;
    let tail = &em[start..];
    let expected = alg.encoded_len();
    if tail.len() != expected {
        return Err(RsaError::DigestInfoLength {
            expected,
            found: tail.len(),
        });
    }
    let prefix = alg.der_prefix();
    if !ct_eq(&tail[..prefix.len()], prefix) {
        return Err(RsaError::DigestInfoMismatch);
    }
    if !ct_eq(&tail[prefix.len()..], digest) {
        return Err(RsaError::DigestMismatch);
    }
    Ok(())
}

/// Reject inputs that cannot possibly hold the structure, before indexing `em`.
fn length_guards(em: &[u8], digest: &[u8], alg: DigestAlgorithm) -> Result<(), RsaError> {
    let expected = alg.digest_len();
    if digest.len() != expected {
        return Err(RsaError::DigestLength {
            expected,
            found: digest.len(),
        });
    }
    // 0x00 0x01 + at least 8 padding octets + 0x00 separator + DigestInfo.
    let needed = 3 + super::pkcs1_padding::MIN_PADDING_RUN + alg.encoded_len();
    if em.len() < needed {
        return Err(RsaError::EncodingTooShort {
            modulus_bytes: em.len(),
            needed,
        });
    }
    Ok(())
}
