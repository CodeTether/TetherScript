//! # The EMSA-PKCS1-v1_5 padding walk
//!
//! One responsibility: validate the fixed-pattern head of an encoded message —
//! `0x00 0x01`, the `0xFF` run, the `0x00` separator — and report where the DER
//! `DigestInfo` begins. The `DigestInfo` and digest themselves are checked in
//! `super::pkcs1`.
//!
//! ## RFC 8017 section 9.2 step 2
//!
//! The encoded message is exactly
//!
//! ```text
//! EM = 0x00 || 0x01 || PS || 0x00 || T
//! ```
//!
//! where `PS` is `0xFF` repeated `k - tLen - 3` times, which the same section
//! requires to be at least 8 octets, and `T` is the DER `DigestInfo`.
//!
//! ## Each rejection, and the forgery it blocks
//!
//! - **Leading octets not `0x00 0x01`.** The leading `0x00` is what keeps the
//!   encoded integer below `n`; the `0x01` is the block type. Accepting other
//!   values (notably block type `0x02`, or a missing leading zero) admits whole
//!   families of small-exponent forgeries.
//! - **`0xFF` run shorter than 8.** Short padding is *the* Bleichenbacher-style
//!   forgery lever: the shorter `PS` is allowed to be, the more room is left for
//!   attacker-chosen octets, and with a small `e` a cube-root construction
//!   produces an integer whose encoding passes a lax check with no private key.
//! - **Run not terminated by `0x00`.** Without a mandatory separator the
//!   boundary between padding and `DigestInfo` is attacker-movable, so an
//!   attacker can slide the digest to a position that happens to match.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::rsa::{check_encoding, DigestAlgorithm, RsaError};
//!
//! // Eight 0xff octets is the documented minimum, so seven must be refused
//! // even though the block is otherwise the right size.
//! let mut em = vec![0x00, 0x01];
//! em.extend(vec![0xff; 7]);
//! em.resize(256, 0x00);
//! assert_eq!(
//!     check_encoding(&em, &[0u8; 32], DigestAlgorithm::Sha256),
//!     Err(RsaError::PaddingRunTooShort { len: 7 })
//! );
//! ```

use super::error::RsaError;

/// Minimum number of `0xFF` padding octets required by RFC 8017 section 9.2.
pub(super) const MIN_PADDING_RUN: usize = 8;

/// Validate the padding head and locate the DER `DigestInfo`.
///
/// # Arguments
///
/// * `em` — the full encoded message, at least two octets long.
///
/// # Returns
///
/// The index of the first `DigestInfo` octet, i.e. one past the `0x00`
/// separator.
///
/// # Errors
///
/// [`RsaError::LeadingBytes`], [`RsaError::PaddingRunTooShort`], or
/// [`RsaError::MissingSeparator`]; see the [module docs](self).
pub(super) fn walk(em: &[u8]) -> Result<usize, RsaError> {
    let (first, second) = (em[0], em[1]);
    if first != 0x00 || second != 0x01 {
        return Err(RsaError::LeadingBytes { first, second });
    }
    let mut index = 2;
    while index < em.len() && em[index] == 0xff {
        index += 1;
    }
    let run = index - 2;
    if run < MIN_PADDING_RUN {
        return Err(RsaError::PaddingRunTooShort { len: run });
    }
    // A run that reached the end never met a separator; a run that stopped on a
    // non-0x00 octet met the wrong one. Both are the same refusal.
    if em.get(index) != Some(&0x00) {
        return Err(RsaError::MissingSeparator);
    }
    Ok(index + 1)
}
