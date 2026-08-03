//! Wall-clock seconds for OAuth state expiry.
//!
//! # Why a local copy
//!
//! `datetime_args::now_secs`, `csrf_payload::now_secs`, and
//! `session_ttl::now_secs` are all this exact function, and all three are
//! `pub(super)` to groups this task must not edit. Rather than widen someone
//! else's visibility, the four-line body is repeated. Folding all four into one
//! `pub(crate)` clock is follow-up work for whoever owns them.
//!
//! # Note on trust
//!
//! State expiry is checked against the *server's* clock, and the expiry itself sits
//! inside the HMAC-authenticated payload, so a client cannot extend a state's life.
//! A server whose clock jumps backwards will accept an expired state for as long as
//! the skew lasts; that is a host configuration problem this module cannot detect.

use std::time::{SystemTime, UNIX_EPOCH};

/// Current Unix time in whole seconds.
///
/// # Returns
///
/// Seconds since the Unix epoch, saturating at [`i64::MAX`], and `0` when the clock
/// is set before the epoch. Neither degenerate case can extend a token's life
/// without the signing secret.
pub(crate) fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs().min(i64::MAX as u64) as i64)
        .unwrap_or(0)
}
