//! PBKDF2-HMAC-SHA-256 key derivation.
//!
//! Unlike the single-block helper in [`crate::postgres`], this generalizes to any
//! output length: blocks are derived at index 1, 2, 3, … and concatenated per
//! RFC 8018 section 5.2, then truncated. That matters because a caller may raise
//! the derived-key length without needing a second implementation.
//!
//! The HMAC underneath is the one the `hmac` group re-exports, so there is a
//! single HMAC construction in this module tree rather than a private copy.

use super::super::hmac::hmac_sha256;

/// SHA-256 output width, and therefore the PBKDF2 block width.
pub(super) const HASH_LEN: usize = 32;

/// Derive `length` bytes from `password` and `salt`.
///
/// # Arguments
///
/// * `password` — Secret input, used as the HMAC key.
/// * `salt` — Per-password random value; must be unique per credential.
/// * `iterations` — Round count. Higher is slower to attack; must be >= 1.
/// * `length` — Desired output length in bytes, which may exceed one 32-byte block.
///
/// # Returns
///
/// Exactly `length` derived bytes.
pub(super) fn derive(password: &[u8], salt: &[u8], iterations: u32, length: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(length);
    let mut index: u32 = 1;
    while out.len() < length {
        out.extend_from_slice(&block(password, salt, iterations, index));
        index += 1;
    }
    out.truncate(length);
    out
}

/// Compute one PBKDF2 block: `U1 ^ U2 ^ … ^ Uc` for block `index`.
fn block(password: &[u8], salt: &[u8], iterations: u32, index: u32) -> [u8; HASH_LEN] {
    // U1 = HMAC(password, salt || INT_BE32(index))
    let mut seed = Vec::with_capacity(salt.len() + 4);
    seed.extend_from_slice(salt);
    seed.extend_from_slice(&index.to_be_bytes());

    let mut previous = hmac_sha256(password, &seed);
    let mut result = previous;
    for _ in 1..iterations {
        previous = hmac_sha256(password, &previous);
        for (acc, byte) in result.iter_mut().zip(previous.iter()) {
            *acc ^= byte;
        }
    }
    result
}
