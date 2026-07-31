//! Per-password salt generation.
//!
//! A random salt per credential is what stops one precomputed table from
//! breaking every account, and stops two users with the same password from
//! sharing a hash. A fixed salt would defeat both, so there is deliberately no
//! way to supply one from a script.
//!
//! Entropy follows the approach already proven in `src/postgres/nonce.rs`: a
//! fixed-size read from `/dev/urandom`, with a time and PID derived path when the
//! OS source is unavailable. That keeps the core build dependency-free.

use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

/// Salt width in bytes. RFC 8018 requires at least 8; 16 is the modern floor.
pub(super) const SALT_LEN: usize = 16;

/// Produce a fresh salt.
///
/// # Returns
///
/// `SALT_LEN` bytes, drawn from the OS CSPRNG when available.
pub(super) fn generate() -> Vec<u8> {
    if let Some(bytes) = os_random(SALT_LEN) {
        return bytes;
    }
    derived_from_clock_and_pid(SALT_LEN)
}

/// Read exactly `len` bytes from the OS random device.
///
/// A fixed-size `read_exact` is required: `/dev/urandom` never reaches EOF, so a
/// whole-file read would block forever.
fn os_random(len: usize) -> Option<Vec<u8>> {
    let mut file = std::fs::File::open("/dev/urandom").ok()?;
    let mut bytes = vec![0u8; len];
    file.read_exact(&mut bytes).ok()?;
    Some(bytes)
}

/// Derive bytes from the clock and PID when no OS source is reachable.
///
/// This is weaker than the OS CSPRNG and is a last resort, not a design choice.
/// It still varies per process and per call, so salts do not repeat in practice.
fn derived_from_clock_and_pid(len: usize) -> Vec<u8> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| (elapsed.as_nanos() as u64) ^ elapsed.as_secs())
        .unwrap_or(0);
    let mut seed = nanos ^ ((std::process::id() as u64) << 32);
    let mut out = Vec::with_capacity(len);
    while out.len() < len {
        // Mix forward so each byte depends on all prior state.
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        out.push((seed >> 33) as u8);
    }
    out
}
