//! Fresh OS entropy for PKCE verifiers and state nonces.
//!
//! # Why this file exists rather than a call into the `random` group
//!
//! `random_source::bytes` is exactly this function, but it is declared
//! `mod random_source;` (private) inside `web_builtins/random.rs` and is not
//! re-exported, so it is unreachable from here. Making it reachable means editing
//! another sub-agent's file, which this task forbids. The same reasoning already
//! produced local copies in `csrf_payload.rs`, `password_salt.rs`, `uuid_gen.rs`,
//! and `postgres/nonce.rs`; this is the fifth, and consolidating all five behind
//! one `pub(crate)` entropy source is follow-up work for whoever owns them.
//!
//! # Security
//!
//! A PKCE verifier an attacker can predict defeats the whole exchange: the
//! challenge in the authorization request is public, so a guessable verifier lets
//! whoever intercepts a code redeem it. There is no seeding and no cached PRNG
//! state, so recovering one verifier reveals nothing about the next.
//!
//! The non-device path is a *degraded* fallback for platforms without
//! `/dev/urandom`, not a designed CSPRNG. It matches the existing in-tree precedent
//! so the core build keeps zero dependencies.

use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

/// Fill `len` bytes with unpredictable data.
///
/// # Arguments
///
/// * `len` — Number of bytes wanted.
///
/// # Returns
///
/// A vector of exactly `len` bytes, from `/dev/urandom` where available.
pub(crate) fn bytes(len: usize) -> Vec<u8> {
    let mut out = vec![0u8; len];
    // A fixed-size `read_exact` is required: `/dev/urandom` never reaches EOF, so a
    // read-to-end would block forever.
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        if file.read_exact(&mut out).is_ok() {
            return out;
        }
    }
    let mut seed = derived_seed();
    for byte in out.iter_mut() {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *byte = (seed >> 33) as u8;
    }
    out
}

/// Mix wall-clock nanoseconds with the PID when no entropy device is present.
fn derived_seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.subsec_nanos() as u64 ^ elapsed.as_secs())
        .unwrap_or(0);
    nanos ^ ((std::process::id() as u64) << 32)
}
