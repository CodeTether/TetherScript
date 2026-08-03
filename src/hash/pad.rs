//! Merkle–Damgård length padding shared by SHA-1, SHA-512, and SHA-384.
//!
//! Every hash in this directory appends `0x80`, then zero bytes, then a
//! big-endian **bit** count. The subtle part is the block boundary: once the
//! `0x80` byte is written, a message whose length is exactly 56 mod 64 (SHA-1)
//! or 112 mod 128 (SHA-512) can no longer fit its length field in the current
//! block, so padding must roll over into a whole extra block. That rollover is
//! the classic SHA implementation bug; `tests/hash_sha1.rs` and
//! `tests/hash_sha512.rs` pin those exact lengths.
//!
//! # Overflow
//!
//! The SHA-256 in `src/system.rs` converts bytes to bits with
//! `wrapping_mul(8)`, which silently produces a wrong length field for inputs at
//! or above 2^61 bytes. This module computes in `u128` with `checked_mul` and
//! panics loudly instead, because a wrong length field yields a
//! wrong-but-plausible digest — the worst failure mode a hash can have. The
//! `u128` intermediate also supplies SHA-512's 128-bit length field directly.

/// Message length in bits, computed without wrapping.
///
/// # Arguments
///
/// * `byte_len` — Message length in bytes.
///
/// # Returns
///
/// `byte_len * 8` as a `u128`; for example `bit_len(7) == 56`.
///
/// # Panics
///
/// Panics if the bit length overflows `u128`. That is unreachable on any real
/// machine, but it is checked rather than assumed.
pub(crate) fn bit_len(byte_len: usize) -> u128 {
    (byte_len as u128)
        .checked_mul(8)
        .expect("hash: message bit length overflows u128")
}
