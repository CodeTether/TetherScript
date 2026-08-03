//! `exp` and `nbf` enforcement with symmetric clock skew.
//!
//! One responsibility: compare two numeric-date claims against a caller-supplied
//! `now`. No clock is read in this module, or anywhere else in `crate::jwtrs`.
//!
//! # Why `now` is an argument
//!
//! Three reasons, in order of importance:
//!
//! 1. **Testability.** Expiry is the one claim whose behaviour changes with the
//!    wall clock. With an injected `now`, "expired by one second" and "expired but
//!    inside skew" are two ordinary assertions. With an ambient clock they become
//!    `sleep`-based tests that are slow, flaky, and untestable for the past.
//! 2. **Consistency within one request.** A middleware validating several tokens,
//!    or re-checking one, gets a single instant for the whole decision. Reading the
//!    clock per check lets `nbf` and `exp` be evaluated against different instants,
//!    so a token can be simultaneously not-yet-valid and expired.
//! 3. **Honesty about the dependency.** Time is an input to this decision. Hiding
//!    it inside the function makes it look like a pure claim check when it is not,
//!    and callers cannot see that a wrong system clock breaks their auth.
//!
//! # Skew is symmetric, and 60 seconds by default
//!
//! `exp` is compared against `now - skew`, and `nbf` against `now + skew`. One
//! number therefore describes the whole uncertainty about clock offset, in both
//! directions, because nobody knows *which way* two clocks disagree. The default and
//! its justification live in [`DEFAULT_SKEW_SECS`](crate::jwtrs::limits::DEFAULT_SKEW_SECS).
//!
//! # `exp` is required; `nbf` and `iat` are not
//!
//! A token with no `exp` never expires. A leaked bearer token that never expires is
//! a permanent credential, and revocation lists are exactly what a stateless
//! verifier does not have — so an absent `exp` is
//! [`ClaimError::Missing`], not "no limit". `nbf` is genuinely optional (RFC 7519
//! §4.1.5) and enforced only when present; `iat` is informational and never gates
//! acceptance.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::base64url::encode;
//! use tetherscript::jwtrs::segment::decode_object;
//! use tetherscript::jwtrs::time_window::check;
//!
//! let members = decode_object("payload", &encode(br#"{"exp":1000,"nbf":900}"#)).unwrap();
//! assert!(check(&members, 950, 60).is_ok());          // inside the window
//! assert!(check(&members, 1030, 60).is_ok());         // expired, but within skew
//! assert!(check(&members, 1100, 60).is_err());        // expired beyond skew
//! assert!(check(&members, 800, 60).is_err());         // before nbf beyond skew
//!
//! // No `exp` at all is refused rather than treated as eternal.
//! let eternal = decode_object("payload", &encode(br#"{"sub":"u"}"#)).unwrap();
//! assert!(check(&eternal, 950, 60).is_err());
//! ```

use std::collections::HashMap;

use crate::jwtrs::error_claims::ClaimError;
use crate::jwtrs::payload_fields::{optional_secs, required_secs};
use crate::value::Value;

/// Enforce the token's validity window.
///
/// # Arguments
///
/// * `members` — The authenticated payload object.
/// * `now` — Seconds since the Unix epoch, supplied by the caller.
/// * `skew` — Tolerance in seconds, applied symmetrically. Callers pass
///   `config.skew_secs`, which is already clamped non-negative.
///
/// # Returns
///
/// `Ok(())` when `now` is inside `[nbf - skew, exp + skew)`.
///
/// # Errors
///
/// [`ClaimError::Missing`] for an absent `exp`, [`ClaimError::NotNumber`] for a
/// non-numeric `exp` or `nbf`, [`ClaimError::Expired`], and
/// [`ClaimError::NotYetValid`]. Each carries `exp`/`nbf`, `now`, and `skew`, so a
/// misconfigured clock is diagnosable from the message alone.
///
/// # Panics
///
/// Does not panic. Comparisons use saturating arithmetic, so an `exp` of
/// `i64::MAX` cannot overflow when the skew is added.
pub fn check(members: &HashMap<String, Value>, now: i64, skew: i64) -> Result<(), ClaimError> {
    let exp = required_secs(members, "exp")?;
    if now.saturating_sub(skew) >= exp {
        return Err(ClaimError::Expired { exp, now, skew });
    }
    if let Some(nbf) = optional_secs(members, "nbf")? {
        if now.saturating_add(skew) < nbf {
            return Err(ClaimError::NotYetValid { nbf, now, skew });
        }
    }
    Ok(())
}
