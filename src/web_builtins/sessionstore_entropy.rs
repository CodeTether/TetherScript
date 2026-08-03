//! CSPRNG byte source for Redis-backed session ids.
//!
//! # Why this file exists rather than an import
//!
//! The audited entropy path in this tree is `web_builtins/random_source.rs`. It is
//! declared `mod random_source;` — private to the `random` group — and it is not
//! re-exported, so it cannot be named from here, and this group may not edit a file
//! it does not own. Including the same file under a second module name would trip
//! `clippy::duplicate_mod`.
//!
//! Nothing below implements a CSPRNG: it *reads* the operating system's, with the
//! same fixed-size `read_exact` and the same documented degraded fallback as
//! `random_source.rs`, `postgres/nonce.rs`, `uuid_gen.rs`, `password_salt.rs`, and
//! `csrf_payload.rs`. **The single-copy fix is a one-line change the integrator can
//! make:** promote `random_source` to `pub(crate) mod` in `random.rs`, import
//! `bytes` here, and delete this file. That edit is offered rather than taken,
//! because `random.rs` belongs to another group.
//!
//! `read_exact` into a fixed buffer is required: `/dev/urandom` never reaches EOF,
//! so a read-to-end would block forever.

use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

/// Fill `len` bytes with fresh entropy.
///
/// # Arguments
///
/// * `len` — Number of bytes wanted. Session ids ask for 32.
///
/// # Returns
///
/// Exactly `len` bytes, from the OS CSPRNG whenever the device is readable.
///
/// # Errors
///
/// Infallible: an unreadable entropy device falls back to the degraded path below
/// rather than failing a request. The fallback is documented as weaker, not equal.
///
/// # Examples
///
/// ```rust,ignore
/// let first = bytes(32);
/// assert_eq!(first.len(), 32);
/// assert_ne!(first, bytes(32));
/// ```
pub(super) fn bytes(len: usize) -> Vec<u8> {
    let mut out = vec![0u8; len];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        if file.read_exact(&mut out).is_ok() {
            return out;
        }
    }
    fill_degraded(&mut out);
    out
}

/// Last-resort filler for platforms with no entropy device.
///
/// Mixes the clock and the process id through the same LCG the in-tree SCRAM nonce
/// path uses, so ids still differ per call. This is **not** a substitute for OS
/// entropy and is not a designed CSPRNG; it exists so a missing device degrades
/// instead of panicking inside a request handler.
fn fill_degraded(out: &mut [u8]) {
    let mut seed = derived_seed();
    for byte in out.iter_mut() {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *byte = (seed >> 33) as u8;
    }
}

/// Seed derived from wall-clock nanoseconds mixed with the process id.
fn derived_seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| (elapsed.as_nanos() as u64) ^ elapsed.as_secs())
        .unwrap_or(0);
    nanos ^ ((std::process::id() as u64) << 32)
}
