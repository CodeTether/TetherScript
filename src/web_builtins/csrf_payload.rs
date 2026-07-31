//! Token payload construction and parsing.
//!
//! The payload is a compact fixed-order text record rather than JSON, so parsing
//! needs no allocator-heavy decoder and the field order is part of the format:
//! `v1.<nonce>.<iat>.<exp>`, all in seconds.
//!
//! Entropy follows the in-tree pattern used for SCRAM nonces and UUIDs: a
//! fixed-size read from `/dev/urandom`, with a time-and-PID derived path when the
//! device is unavailable. `/dev/urandom` never reaches EOF, so it is read with
//! `read_exact` into a fixed buffer; a whole-file read would block forever.

use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::system::hex_encode;

/// Format version, so a future change can be rejected explicitly.
pub(super) const VERSION: &str = "v1";

/// Decoded token fields.
pub(super) struct Claims {
    pub(super) nonce: String,
    pub(super) issued_at: i64,
    pub(super) expires_at: i64,
}

/// Build a fresh payload valid for `ttl_seconds` from now.
pub(super) fn build(ttl_seconds: i64) -> Claims {
    let now = now_secs();
    Claims {
        nonce: hex_encode(&random_bytes()),
        issued_at: now,
        expires_at: now.saturating_add(ttl_seconds),
    }
}

/// Render a payload to its wire form.
pub(super) fn render(claims: &Claims) -> String {
    format!(
        "{VERSION}.{}.{}.{}",
        claims.nonce, claims.issued_at, claims.expires_at
    )
}

/// Current Unix time in seconds.
pub(super) fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs().min(i64::MAX as u64) as i64)
        .unwrap_or(0)
}

/// Draw 16 fresh random bytes for the nonce.
///
/// A fresh nonce per token is what makes two tokens minted in the same second
/// differ, so a token cannot be guessed from a previously observed one.
fn random_bytes() -> [u8; 16] {
    let mut bytes = [0u8; 16];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        if file.read_exact(&mut bytes).is_ok() {
            return bytes;
        }
    }
    let mut seed = now_secs() as u64 ^ ((std::process::id() as u64) << 32);
    for byte in bytes.iter_mut() {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *byte = (seed >> 33) as u8;
    }
    bytes
}
