//! Generation of a fresh request id when none arrives, or the arriving one is unsafe.
//!
//! # On reuse
//!
//! The in-tree generators were preferred and are **not reachable from here**.
//! `web_builtins/uuid_gen.rs::v4`, `web_builtins/random_source.rs::bytes`, and
//! `postgres/nonce.rs::client_nonce` are each `pub(super)` inside a sibling module
//! whose own `mod` declaration is private, so they are visible only to that
//! module's children. Widening any of them would mean editing a file this task may
//! not touch. What *is* reachable is [`crate::system::hex_encode`], which is
//! `pub(crate)`, so the hex formatting is genuinely shared and only the 16-byte
//! entropy read is local — following the same fixed-size `read_exact` pattern the
//! rest of the tree uses, for the same reason: `/dev/urandom` never reaches EOF,
//! so a whole-file read would block forever.
//!
//! # Why the format is a UUIDv4
//!
//! An id in canonical 8-4-4-4-12 lowercase hex is what the reference application's
//! tracing already emits, it is trivially recognisable in a log, and every
//! character it contains is inside the charset
//! [`super::identity_request_id::is_safe`] allows — so a generated id always
//! survives the same validation an incoming one must pass. A generator that could
//! emit a value its own validator rejects would be a latent bug.
//!
//! Unpredictability is not a security property here: a request id is a
//! correlation handle, never a credential. Uniqueness is what matters, and 122
//! random bits gives it.

use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::system::hex_encode;

/// Produce a fresh request id.
///
/// # Returns
///
/// A 36-character canonical version-4 UUID such as
/// `f47ac10b-58cc-4372-a567-0e02b2c3d479`. Two calls return distinct values.
pub(super) fn fresh() -> String {
    let mut bytes = entropy();
    // RFC 4122: version 4 in the high nibble of byte 6, variant `10` in byte 8.
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex = hex_encode(&bytes);
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// Fill 16 bytes, preferring OS entropy.
fn entropy() -> [u8; 16] {
    let mut bytes = [0u8; 16];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        if file.read_exact(&mut bytes).is_ok() {
            return bytes;
        }
    }
    // Without the device, mix time and PID through the LCG the in-tree nonce path
    // uses. Weaker, but a request id needs distinctness, not secrecy.
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|gap| gap.subsec_nanos() as u64 ^ gap.as_secs())
        .unwrap_or(0)
        ^ ((std::process::id() as u64) << 32);
    for byte in bytes.iter_mut() {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *byte = (seed >> 33) as u8;
    }
    bytes
}
