//! The three outcomes of `TTL`.
//!
//! Redis encodes them in one integer reply using negative sentinels; this type
//! names them so a caller cannot accidentally treat "no expiry" and "no key" as the
//! same thing.

/// A key's remaining lifetime.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis::Ttl;
///
/// assert_eq!(Ttl::from_reply(42), Ttl::Seconds(42));
/// assert_eq!(Ttl::from_reply(-1), Ttl::Persistent);
/// assert_eq!(Ttl::from_reply(-2), Ttl::Missing);
///
/// match Ttl::from_reply(0) {
///     Ttl::Seconds(left) => assert_eq!(left, 0),
///     Ttl::Persistent => unreachable!(),
///     Ttl::Missing => unreachable!(),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ttl {
    /// Seconds remaining before the key expires. May be `0`.
    Seconds(u64),
    /// The key exists and will never expire (`-1`).
    Persistent,
    /// The key does not exist (`-2`).
    Missing,
}

impl Ttl {
    /// Interpret a raw `TTL` integer reply.
    ///
    /// # Arguments
    ///
    /// * `reply` — The integer Redis sent.
    ///
    /// # Returns
    ///
    /// The named outcome. Any negative value other than `-1` maps to
    /// [`Ttl::Missing`], since `-2` is the only other one Redis defines and
    /// treating an unknown negative as a lifetime would be worse.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::redis::Ttl;
    ///
    /// assert_eq!(Ttl::from_reply(-1), Ttl::Persistent);
    /// ```
    pub fn from_reply(reply: i64) -> Self {
        match reply {
            -1 => Self::Persistent,
            value if value < 0 => Self::Missing,
            value => Self::Seconds(value as u64),
        }
    }
}
