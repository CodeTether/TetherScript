//! Uniform integer selection without modulo bias.
//!
//! # Why not `%`
//!
//! The obvious `random_u64() % range` is **not** uniform unless `range` divides
//! 2^64 evenly. Because 2^64 is not a multiple of most ranges, the first
//! `2^64 % range` values are reachable by one extra draw each, so small results
//! come up slightly more often. For a shuffle that is a subtle statistical flaw;
//! for a token or a nonce it is exploitable bias.
//!
//! Rejection sampling removes it: discard any draw landing in the short final
//! partial block and try again. Each retry has probability under 1/2, so the
//! expected number of draws is below two and the loop always terminates.

use super::random_source::bytes;

/// Draw a uniformly distributed `u64`.
fn next_u64() -> u64 {
    let raw = bytes(8);
    let mut value = [0u8; 8];
    value.copy_from_slice(&raw);
    u64::from_le_bytes(value)
}

/// Pick a uniform value in `0..span`, with no modulo bias.
///
/// # Arguments
///
/// * `span` — Exclusive upper bound. Must be non-zero; callers guarantee this.
///
/// # Returns
///
/// A uniformly distributed value in `[0, span)`.
pub(super) fn below(span: u64) -> u64 {
    // `zone` is the largest exact multiple of `span` at or below u64::MAX, so the
    // accepted window [0, zone) contains exactly the same number of draws for
    // every residue. Anything at or above it is the truncated tail and is
    // rejected instead of being folded in, which is what would reintroduce bias.
    let zone = (u64::MAX / span) * span;
    loop {
        let candidate = next_u64();
        if candidate < zone {
            return candidate % span;
        }
    }
}
