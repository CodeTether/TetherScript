//! MD5 for PostgreSQL's legacy `md5` authentication method.
//!
//! MD5 is cryptographically broken and is implemented only because existing
//! PostgreSQL deployments still request it. Never use it for anything else;
//! SCRAM-SHA-256 is the modern path and is preferred whenever offered.

use super::md5_block::compress;

/// Compute the MD5 digest of `input`.
pub(super) fn digest(input: &[u8]) -> [u8; 16] {
    let mut state: [u32; 4] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476];
    for chunk in pad(input).chunks(64) {
        state = compress(state, &words(chunk));
    }
    let mut out = [0u8; 16];
    for (index, word) in state.iter().enumerate() {
        out[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    out
}

/// Split a 64-byte block into sixteen little-endian words.
fn words(chunk: &[u8]) -> [u32; 16] {
    let mut m = [0u32; 16];
    for (index, word) in m.iter_mut().enumerate() {
        let base = index * 4;
        *word = u32::from_le_bytes([
            chunk[base],
            chunk[base + 1],
            chunk[base + 2],
            chunk[base + 3],
        ]);
    }
    m
}

/// Append the `0x80` marker, zero padding, and the little-endian bit length.
fn pad(input: &[u8]) -> Vec<u8> {
    let mut padded = input.to_vec();
    let bit_len = (input.len() as u64).wrapping_mul(8);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_le_bytes());
    padded
}
