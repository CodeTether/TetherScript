//! The three states of a key's time-to-live.
//!
//! `TTL` answers with an integer, but two of its values are sentinels rather than
//! durations: `-1` means *the key exists and never expires*, `-2` means *there is
//! no such key*. Returning the bare integer pushes that trivia onto every caller,
//! and the failure mode is silent — arithmetic on `-2` yields a plausible-looking
//! number, so a rate limiter would happily treat a missing bucket as one expiring
//! in the past. Modelling the three cases as variants makes the distinction
//! impossible to skip, and keeps *missing key* distinguishable from *no expiry*
//! for the same reason a missing value is distinguishable from an empty one.

/// The lifetime of a key as reported by `TTL`.
///
/// # Examples
///
/// ```rust,ignore
/// use tetherscript::redis::client::Ttl;
///
/// assert_eq!(Ttl::from_reply(30), Ttl::Seconds(30));
/// assert_eq!(Ttl::from_reply(-1), Ttl::Persistent);
/// assert_eq!(Ttl::from_reply(-2), Ttl::Missing);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ttl {
    /// The key expires in this many seconds.
    Seconds(u64),
    /// The key exists with no expiry set (`-1`).
    Persistent,
    /// No such key (`-2`).
    Missing,
}

impl Ttl {
    /// Interpret a raw `TTL` integer reply.
    ///
    /// # Arguments
    ///
    /// * `reply` — The integer the server sent.
    ///
    /// # Returns
    ///
    /// [`Ttl::Persistent`] for `-1`, [`Ttl::Missing`] for `-2` and any other
    /// negative value — an unknown negative is a *sentinel* by construction, and
    /// treating it as a duration is the bug this type exists to prevent — and
    /// [`Ttl::Seconds`] otherwise.
    pub fn from_reply(reply: i64) -> Self {
        match reply {
            -1 => Self::Persistent,
            negative if negative < 0 => Self::Missing,
            seconds => Self::Seconds(seconds as u64),
        }
    }
}
