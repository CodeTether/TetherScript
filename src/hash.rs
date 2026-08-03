//! SHA-1, SHA-384, and SHA-512, with HMAC over each.
//!
//! Siblings of the SHA-256 already in [`crate::system`], added because the protocols the
//! port needs demand them: SHA-1 for the RFC 6455 WebSocket handshake, SHA-512 for JWT's
//! HS512.
//!
//! # Layers
//!
//! | Concern | Modules |
//! |---|---|
//! | SHA-1 | `sha1`, `sha1_block` |
//! | SHA-512 / SHA-384 | `sha512`, `sha512_*`, `sha384` |
//! | Padding | `pad`, `pad_block` |
//! | HMAC | `hmac_sha1`, `hmac_sha512` |
//!
//! # SHA-1 is broken for signatures
//!
//! SHA-1 is collision-broken and must not be used for new signatures or content integrity.
//! It is here only for protocol compatibility, where the specification fixes it as a
//! non-secret transformation — the WebSocket accept value proves nothing about identity.
//!
//! # Padding is where implementations go wrong
//!
//! A message whose length is exactly 56 mod 64 (SHA-1) or 112 mod 128 (SHA-512) needs a
//! whole extra block. The length counter is in **bits**, big-endian, and SHA-512's field is
//! 128 bits wide. Each of those cases is pinned by a test rather than assumed.

#[path = "hash/hmac_sha1.rs"]
pub mod hmac_sha1;
#[path = "hash/hmac_sha512.rs"]
pub mod hmac_sha512;
#[path = "hash/pad.rs"]
mod pad;
#[path = "hash/pad_block.rs"]
mod pad_block;
#[path = "hash/sha1.rs"]
pub mod sha1;
#[path = "hash/sha1_block.rs"]
mod sha1_block;
#[path = "hash/sha384.rs"]
pub mod sha384;
#[path = "hash/sha512.rs"]
pub mod sha512;
#[path = "hash/sha512_block.rs"]
mod sha512_block;
#[path = "hash/sha512_consts.rs"]
mod sha512_consts;
#[path = "hash/sha512_core.rs"]
mod sha512_core;
#[path = "hash/sha512_iv.rs"]
mod sha512_iv;
