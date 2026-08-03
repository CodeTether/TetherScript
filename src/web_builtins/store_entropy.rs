//! CSPRNG bytes for session ids.
//!
//! # Why this file exists rather than an import
//!
//! The audited entropy path in this tree is `random_source.rs` in the `random`
//! group, which reads a fixed-size buffer from `/dev/urandom` with a documented
//! time-and-PID fallback. It is declared `mod random_source;` — private to that
//! group — so it is not reachable from here, and this group may not edit files it
//! does not own. Re-including the same file under a second module name would trip
//! `clippy::duplicate_mod`.
//!
//! So this delegates to the same primitive by the same rules, matching the
//! established in-tree convention: `password_salt.rs`, `csrf_payload.rs`,
//! `uuid_gen.rs`, and `postgres/nonce.rs` each carry the same short read for the
//! same reason. **The single-copy fix is a one-line change the integrator can
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
/// * `len` — Number of bytes to produce. Session ids ask for 32.
///
/// # Returns
///
/// Exactly `len` bytes, from the OS CSPRNG when the device is readable.
///
/// # Examples
///
/// ```rust,ignore
/// let a = bytes(32);
/// assert_eq!(a.len(), 32);
/// assert_ne!(a, bytes(32));
/// ```
pub(super) fn bytes(len: usize) -> Vec<u8> {
    let mut out = vec![0u8; len];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        if file.read_exact(&mut out).is_ok() {
            return out;
        }
    }
    // Weaker last resort, not a design choice: mix the clock and PID through the
    // same LCG the in-tree nonce path uses so ids still differ per call.
    let mut seed = derived_seed();
    for byte in out.iter_mut() {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *byte = (seed >> 33) as u8;
    }
    out
}

/// Seed derived from wall-clock nanoseconds mixed with the process id.
fn derived_seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| (elapsed.as_nanos() as u64) ^ elapsed.as_secs())
        .unwrap_or(0);
    nanos ^ ((std::process::id() as u64) << 32)
}
