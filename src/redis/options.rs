//! Options for `SET`.
//!
//! `SET` grows modifiers rather than variants, so the options are a struct instead
//! of a family of methods. Two are supported here, chosen because they are what a
//! session store and a rate limiter actually need:
//!
//! - `EX <seconds>` — set the value and its expiry atomically. Doing it as `SET`
//!   then `EXPIRE` leaves a window where a crash in between leaks a key that never
//!   expires, which is how session stores fill up.
//! - `NX` — set only if the key does not already exist. This is the lock/rate-limit
//!   primitive: the reply distinguishes *I set it* from *someone else already had
//!   it*, which two separate commands cannot do without a race.
//!
//! The order matters on the wire, so the `SET` encoder emits `NX` before `EX`,
//! matching the documented grammar.

/// Modifiers for a `SET` call.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis::SetOptions;
///
/// // Plain overwrite.
/// let plain = SetOptions::default();
/// assert!(plain.expire_seconds.is_none());
///
/// // A session that expires in an hour, only if absent.
/// let guarded = SetOptions { expire_seconds: Some(3600), if_not_exists: true };
/// assert!(guarded.if_not_exists);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SetOptions {
    /// Time-to-live in seconds, sent as `EX <seconds>`. `None` leaves the key
    /// persistent; note that `SET` without `KEEPTTL` also *clears* an existing
    /// key's expiry, so passing `None` over a key that had a TTL makes it
    /// permanent.
    pub expire_seconds: Option<u64>,
    /// Send `NX`, so the write happens only when the key is absent.
    pub if_not_exists: bool,
}

impl SetOptions {
    /// Options that only set an expiry.
    ///
    /// # Arguments
    ///
    /// * `seconds` — Time-to-live.
    ///
    /// # Returns
    ///
    /// Options with `EX` set and `NX` clear.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::redis::SetOptions;
    ///
    /// assert_eq!(SetOptions::expiring(60).expire_seconds, Some(60));
    /// ```
    pub fn expiring(seconds: u64) -> Self {
        Self {
            expire_seconds: Some(seconds),
            if_not_exists: false,
        }
    }
}
