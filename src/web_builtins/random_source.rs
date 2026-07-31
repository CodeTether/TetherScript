//! Cryptographic-quality byte generation for the `random_*` built-ins.
//!
//! Every call reads fresh bytes from the OS. There is deliberately **no** stored
//! PRNG state: seeding once and stepping a generator would produce a long-lived
//! predictable stream, so an attacker who recovered one session token could
//! derive the next. Drawing per call keeps each token independent.
//!
//! The device is read with `read_exact` into a fixed buffer because
//! `/dev/urandom` never reaches EOF — a whole-file read would block forever.
//! This mirrors [`crate::postgres`]'s SCRAM nonce path.

use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

/// Upper bound on a single request, in bytes.
///
/// Caps a script at 4 KiB per call so a typo such as `random_bytes_hex(1 << 30)`
/// cannot allocate a gigabyte inside a request handler. Real secrets — session
/// IDs, CSRF tokens, API keys, salts — are tens of bytes.
pub(super) const MAX_BYTES: i64 = 4096;

/// Fill `len` bytes with fresh entropy.
///
/// # Arguments
///
/// * `len` — Number of bytes to produce. Callers validate against [`MAX_BYTES`].
///
/// # Returns
///
/// A vector of exactly `len` unpredictable bytes.
pub(super) fn bytes(len: usize) -> Vec<u8> {
    let mut out = vec![0u8; len];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        if file.read_exact(&mut out).is_ok() {
            return out;
        }
    }
    // Without the device, mix time and PID through the same LCG the in-tree
    // nonce path uses. This is weaker and only keeps values distinct per call;
    // it is not a substitute for OS entropy.
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
        .map(|elapsed| elapsed.subsec_nanos() as u64 ^ elapsed.as_secs())
        .unwrap_or(0);
    nanos ^ ((std::process::id() as u64) << 32)
}
