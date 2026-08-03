//! Payload masking (RFC 6455 §5.3).
//!
//! Masking is an XOR with a repeating four-byte key. It is not encryption — the
//! key travels in the clear immediately before the payload. Its only purpose is
//! to stop a scripted client from steering a proxy into treating the payload as
//! a second HTTP request (cache poisoning). That is why the *direction* matters
//! and is enforced in [`crate::websocket::header`]: clients must mask, servers
//! must not.
//!
//! Because XOR is its own inverse, masking and unmasking are the same operation.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::websocket::mask;
//!
//! let key = [0x37, 0xfa, 0x21, 0x3d];
//! let mut bytes = b"Hello".to_vec();
//! mask::apply(&mut bytes, key);
//! assert_eq!(bytes, vec![0x7f, 0x9f, 0x4d, 0x51, 0x58]);
//! mask::apply(&mut bytes, key);
//! assert_eq!(&bytes, b"Hello");
//! ```

/// XOR `bytes` in place with the repeating four-byte `key`.
///
/// # Arguments
///
/// * `bytes` — Payload to mask or unmask. Any length, including empty.
/// * `key` — The four-byte masking key read from the frame header.
///
/// # Returns
///
/// Nothing; `bytes` is transformed in place.
///
/// # Panics
///
/// Never. The key index is `i & 3`, which is arithmetically confined to `0..=3`
/// and therefore always in range for a `[u8; 4]`, no matter how long `bytes` is.
pub fn apply(bytes: &mut [u8], key: [u8; 4]) {
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte ^= key[i & 3];
    }
}
