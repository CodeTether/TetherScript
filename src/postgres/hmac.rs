//! HMAC-SHA-256 and PBKDF2-HMAC-SHA-256.
//!
//! SCRAM-SHA-256 authentication needs both. They are built on the in-tree
//! SHA-256 in [`crate::system`] so the core build stays dependency-free.

use crate::system::sha256;

const BLOCK: usize = 64;

/// HMAC-SHA-256 per RFC 2104.
pub(super) fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut block = [0u8; BLOCK];
    // Keys longer than the block size are hashed first; shorter keys zero-pad.
    if key.len() > BLOCK {
        block[..32].copy_from_slice(&sha256(key));
    } else {
        block[..key.len()].copy_from_slice(key);
    }

    let mut inner = Vec::with_capacity(BLOCK + message.len());
    let mut outer = Vec::with_capacity(BLOCK + 32);
    for byte in block.iter() {
        inner.push(byte ^ 0x36);
        outer.push(byte ^ 0x5c);
    }
    inner.extend_from_slice(message);
    outer.extend_from_slice(&sha256(&inner));
    sha256(&outer)
}

/// PBKDF2-HMAC-SHA-256 for a single 32-byte output block.
///
/// SCRAM's `SaltedPassword` is exactly one block wide, so the block index is
/// fixed at 1 rather than generalized.
pub(super) fn pbkdf2_sha256(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    let mut salted = Vec::with_capacity(salt.len() + 4);
    salted.extend_from_slice(salt);
    salted.extend_from_slice(&1u32.to_be_bytes());

    let mut previous = hmac_sha256(password, &salted);
    let mut result = previous;
    for _ in 1..iterations {
        previous = hmac_sha256(password, &previous);
        for (acc, byte) in result.iter_mut().zip(previous.iter()) {
            *acc ^= byte;
        }
    }
    result
}
