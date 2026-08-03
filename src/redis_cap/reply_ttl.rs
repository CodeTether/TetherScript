//! Mapping [`Ttl`] into a script-facing [`Value`].
//!
//! One concern: `TTL`'s three-way answer. Redis encodes it as one integer with
//! negative sentinels, and [`Ttl`] already names the cases, so the only decision
//! here is how a script sees them.
//!
//! | [`Ttl`] | Wire | `Value` | Meaning |
//! |---|---|---|---|
//! | [`Ttl::Seconds`] | `:42` | [`Value::Int`] `42` | expires in 42s |
//! | [`Ttl::Persistent`] | `:-1` | [`Value::Bool`] `false` | exists, never expires |
//! | [`Ttl::Missing`] | `:-2` | [`Value::Nil`] | no such key |
//!
//! The sentinels are *not* passed through as `-1` and `-2`. Handing a script a
//! negative duration invites `if ttl < 60 { refresh() }`, which fires for a key
//! that does not exist and for one that never expires — the two cases where
//! refreshing is wrong. Distinct types make that comparison impossible instead of
//! merely inadvisable.
//!
//! [`Value::Nil`] for missing is the same spelling [`super::reply`] uses for a
//! cache miss, so absence reads identically across the capability.

use crate::redis::Ttl;
use crate::value::Value;

/// Convert a `TTL` outcome for a script.
///
/// # Arguments
///
/// * `ttl` — The outcome decoded by [`Ttl::from_reply`].
///
/// # Returns
///
/// [`Value::Int`] with the seconds remaining, [`Value::Bool`] `false` for a
/// persistent key, or [`Value::Nil`] for a missing one.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis::Ttl;
/// use tetherscript::redis_cap::reply_ttl;
/// use tetherscript::value::Value;
///
/// assert!(matches!(reply_ttl::value(Ttl::Seconds(42)), Value::Int(42)));
/// assert!(matches!(reply_ttl::value(Ttl::Persistent), Value::Bool(false)));
/// assert!(matches!(reply_ttl::value(Ttl::Missing), Value::Nil));
///
/// // A key that expires this second is still a present key, not an absent one.
/// assert!(matches!(reply_ttl::value(Ttl::Seconds(0)), Value::Int(0)));
/// ```
pub fn value(ttl: Ttl) -> Value {
    match ttl {
        Ttl::Seconds(seconds) => Value::Int(seconds as i64),
        Ttl::Persistent => Value::Bool(false),
        Ttl::Missing => Value::Nil,
    }
}
