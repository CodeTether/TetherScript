//! Concrete padded-message builders for the 64-byte and 128-byte block sizes.

use crate::hash::pad::bit_len;

/// Pad `input` for a 64-byte-block hash with a 64-bit length field (SHA-1).
///
/// # Arguments
///
/// * `input` — Message bytes.
///
/// # Returns
///
/// A buffer whose length is a multiple of 64.
///
/// # Panics
///
/// Panics if the bit length exceeds SHA-1's 2^64-bit message limit.
pub(crate) fn padded64(input: &[u8]) -> Vec<u8> {
    let bits = u64::try_from(bit_len(input.len())).expect("hash: message exceeds 2^64 bits");
    let mut bytes = Vec::with_capacity(input.len() + 72);
    bytes.extend_from_slice(input);
    bytes.push(0x80);
    while bytes.len() % 64 != 56 {
        bytes.push(0);
    }
    bytes.extend_from_slice(&bits.to_be_bytes());
    bytes
}

/// Pad `input` for a 128-byte-block hash with a 128-bit length field (SHA-512).
///
/// # Arguments
///
/// * `input` — Message bytes.
///
/// # Returns
///
/// A buffer whose length is a multiple of 128.
pub(crate) fn padded128(input: &[u8]) -> Vec<u8> {
    let bits = bit_len(input.len());
    let mut bytes = Vec::with_capacity(input.len() + 144);
    bytes.extend_from_slice(input);
    bytes.push(0x80);
    while bytes.len() % 128 != 112 {
        bytes.push(0);
    }
    bytes.extend_from_slice(&bits.to_be_bytes());
    bytes
}
