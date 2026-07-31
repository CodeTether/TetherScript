//! Version-4 UUID generation.
//!
//! Entropy follows the in-tree pattern used for SCRAM nonces: a fixed-size read
//! from `/dev/urandom`, with a time-and-PID derived path when that device is
//! unavailable. `/dev/urandom` never reaches EOF, so it is read with
//! `read_exact` into a fixed buffer — a whole-file read would block forever.

use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::system::hex_encode;

/// Generate a random version-4 UUID in canonical 8-4-4-4-12 lowercase hex form.
///
/// # Returns
///
/// A 36-character string such as `f47ac10b-58cc-4372-a567-0e02b2c3d479`. The
/// version nibble is always `4` and the variant bits are always binary `10`, as
/// RFC 4122 requires; databases reject values that omit them.
pub(super) fn v4() -> String {
    let mut bytes = random_bytes();
    // Version 4 in the high nibble of byte 6.
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    // RFC 4122 variant: top two bits of byte 8 set to binary 10.
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
fn random_bytes() -> [u8; 16] {
    let mut bytes = [0u8; 16];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        if file.read_exact(&mut bytes).is_ok() {
            return bytes;
        }
    }
    // Without the device, mix time and PID through the same LCG the nonce path
    // uses, so a host still yields distinct values per call.
    let mut seed = derived_seed();
    for byte in bytes.iter_mut() {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *byte = (seed >> 33) as u8;
    }
    bytes
}

/// Seed derived from wall-clock nanoseconds mixed with the process id.
fn derived_seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.subsec_nanos() as u64 ^ elapsed.as_secs())
        .unwrap_or(0);
    nanos ^ ((std::process::id() as u64) << 32)
}
