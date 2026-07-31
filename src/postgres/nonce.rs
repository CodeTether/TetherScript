//! Client nonce generation for SCRAM.
//!
//! SCRAM requires a fresh, unpredictable client nonce per exchange: reuse lets a
//! replayed server-first message match an old proof. There is no RNG dependency
//! in the core build, so entropy is drawn from the OS where available and mixed
//! with process and time values.

use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

/// Base64 alphabet minus `,`, which is the SCRAM attribute separator.
const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

/// Produce a 24-character printable client nonce.
pub(super) fn client_nonce() -> String {
    let mut seed = seed_bytes();
    let mut out = String::with_capacity(24);
    for index in 0..24 {
        // Mix forward so each output character depends on all prior state.
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407 + index as u64);
        let pick = (seed >> 33) as usize % ALPHABET.len();
        out.push(ALPHABET[pick] as char);
    }
    out
}

/// Gather a seed, preferring OS entropy and falling back to time and PID.
fn seed_bytes() -> u64 {
    // Read a fixed 8 bytes: `/dev/urandom` never reaches EOF, so a whole-file
    // read would block forever.
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        let mut value = [0u8; 8];
        if file.read_exact(&mut value).is_ok() {
            return u64::from_le_bytes(value);
        }
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.subsec_nanos() as u64 ^ elapsed.as_secs())
        .unwrap_or(0);
    nanos ^ ((std::process::id() as u64) << 32)
}
