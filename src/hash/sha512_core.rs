//! Shared SHA-512-family driver: pad, compress every block, serialize state.
//!
//! Both SHA-512 and SHA-384 use this driver; only the IV and the output length
//! differ. Keeping the driver here means the padding and big-endian
//! serialization exist in exactly one place, so the two variants cannot drift.

use crate::hash::pad_block::padded128;
use crate::hash::sha512_block::compress;

/// SHA-512 / SHA-384 block size in bytes.
pub(crate) const BLOCK: usize = 128;

/// Run the SHA-512 chain over `input` starting from `iv`.
///
/// # Arguments
///
/// * `iv` — Initial hash value (SHA-512's or SHA-384's).
/// * `input` — Message bytes.
///
/// # Returns
///
/// The full 64-byte state; SHA-384 truncates this to its leading 48 bytes.
pub(crate) fn digest(iv: [u64; 8], input: &[u8]) -> [u8; 64] {
    let mut h = iv;
    for chunk in padded128(input).chunks_exact(BLOCK) {
        compress(&mut h, chunk);
    }
    let mut out = [0u8; 64];
    for (bytes, word) in out.chunks_exact_mut(8).zip(h) {
        bytes.copy_from_slice(&word.to_be_bytes());
    }
    out
}
