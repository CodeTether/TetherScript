//! SHA-1 (FIPS 180-4), needed by the RFC 6455 opening handshake.
//!
//! # Why this is not reused from elsewhere in the tree
//!
//! `src/rpc_cap.rs` already contains a correct SHA-1, but it is declared as a
//! private `fn sha1` inside that module, so it is not reachable from
//! `crate::websocket` without editing `rpc_cap.rs` — and this task is forbidden
//! from modifying existing files. Base64 *is* reused: `crate::system` exports
//! `base64_encode_bytes` as `pub(crate)`. The recommended follow-up is to lift
//! one SHA-1 into a shared module and have both callers use it; see the report.
//!
//! SHA-1 is cryptographically broken for signatures. It is used here only
//! because RFC 6455 §4.2.2 mandates it as a fixed, non-secret handshake
//! transformation — the accept value proves nothing about identity.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::sha1::sha1;
//!
//! // FIPS 180-2: SHA-1("abc") = a9993e36 4706816a ba3e2571 7850c26c 9cd0d89d
//! let digest = sha1(b"abc");
//! assert_eq!(digest[0], 0xa9);
//! assert_eq!(digest[19], 0x9d);
//! ```

use crate::websocket::sha1_block::compress;

/// Compute the SHA-1 digest of `message`.
///
/// # Arguments
///
/// * `message` — Arbitrary bytes. Any length is accepted.
///
/// # Returns
///
/// The 20-byte big-endian digest.
///
/// # Panics
///
/// Never. Padding is computed so the buffer length is always a multiple of 64,
/// and `chunks_exact(64)` therefore yields only full blocks; no index is derived
/// from input data.
pub fn sha1(message: &[u8]) -> [u8; 20] {
    let mut state: [u32; 5] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];
    let mut data = message.to_vec();
    let bits = (message.len() as u64).wrapping_mul(8);
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0x00);
    }
    data.extend_from_slice(&bits.to_be_bytes());
    for block in data.chunks_exact(64) {
        state = compress(state, block);
    }
    let mut out = [0u8; 20];
    for (word, slot) in state.iter().zip(out.chunks_exact_mut(4)) {
        slot.copy_from_slice(&word.to_be_bytes());
    }
    out
}
