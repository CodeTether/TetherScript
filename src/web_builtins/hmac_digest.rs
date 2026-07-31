//! HMAC-SHA-256 over the in-tree SHA-256.
//!
//! # Why this is not `crate::postgres::hmac::hmac_sha256`
//!
//! That function is verified against RFC 4231 and would be the right thing to
//! call, but it is `pub(super)` inside a private `mod hmac`, so it is not
//! reachable from this module. Exposing it would mean editing `src/postgres.rs`,
//! which the owner of this file is not permitted to touch. The construction below
//! is therefore deliberately identical to it — same block size, same ipad/opad
//! values, same oversized-key rule — and both are pinned to the same published
//! RFC 4231 vectors so they cannot silently diverge.
//!
//! Collapsing the two into one shared helper is follow-up work for whoever owns
//! `src/postgres.rs`.

use crate::system::sha256;

/// SHA-256 block size in bytes, per RFC 2104.
const BLOCK: usize = 64;

/// HMAC-SHA-256 per RFC 2104.
///
/// # Arguments
///
/// * `key` — Secret key. Keys longer than the 64-byte block are hashed first;
///   shorter keys are zero-padded.
/// * `message` — Bytes to authenticate.
///
/// # Returns
///
/// The 32-byte message authentication code.
pub(crate) fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut block = [0u8; BLOCK];
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
