//! The backend seam: what a Redis, SQL, or in-memory store must provide.
//!
//! # Design of the seam
//!
//! Seven operations, each returning `Result<_, String>` so a *transport* failure
//! (Redis unreachable, SQL deadlock) is reportable and is never confused with a
//! *logical* miss, which is `Ok(None)` / `Ok(false)` / `Ok(0)`. Conflating the two
//! is how a store outage silently logs every user out instead of erroring.
//!
//! No expiry logic lives behind this trait. A backend stores [`Record`] fields
//! verbatim and the policy in [`super::store_expiry`] decides, so two backends can
//! never disagree about when a session ended. A backend with native TTL support
//! may *additionally* expire keys early as a space optimisation; that is safe,
//! because an early eviction is indistinguishable from a miss.
//!
//! `create` is separate from `save` even though a key/value backend would use one
//! command for both. A create must not overwrite an existing id: a collision is a
//! CSPRNG failure and must surface loudly rather than silently hand one user
//! another user's session. Redis expresses this as `SET NX`, SQL as an `INSERT`
//! that violates the primary key.
//!
//! Nothing here mentions a connection, a runtime, or a serialisation format, so
//! implementing it needs no file from this group.

use super::store_record::Record;

/// Storage a session store can sit on top of.
pub(super) trait SessionBackend {
    /// Insert a record under a *new* id.
    ///
    /// # Errors
    ///
    /// Returns an error when the id already exists, or when the transport fails.
    fn create(&mut self, record: Record) -> Result<(), String>;

    /// Fetch a record by id.
    ///
    /// # Returns
    ///
    /// `Ok(None)` when no such id is stored — an ordinary miss, not an error.
    ///
    /// # Errors
    ///
    /// Returns an error only when the transport fails.
    fn load(&self, id: &str) -> Result<Option<Record>, String>;

    /// Overwrite an existing record in place, keyed by `record.id`.
    ///
    /// # Errors
    ///
    /// Returns an error when the id is absent, or when the transport fails.
    fn save(&mut self, record: Record) -> Result<(), String>;

    /// Delete one record.
    ///
    /// # Returns
    ///
    /// True when a record was present and is now gone.
    ///
    /// # Errors
    ///
    /// Returns an error only when the transport fails.
    fn delete(&mut self, id: &str) -> Result<bool, String>;

    /// Delete every record belonging to one subject.
    ///
    /// # Returns
    ///
    /// How many records were removed; `0` is a normal answer.
    ///
    /// # Errors
    ///
    /// Returns an error only when the transport fails.
    fn delete_subject(&mut self, subject: &str) -> Result<usize, String>;

    /// Count stored records, expired-but-unswept ones included.
    ///
    /// # Errors
    ///
    /// Returns an error only when the transport fails.
    fn count(&self) -> Result<usize, String>;

    /// Drop records that either clock has expired as of `now_ms`.
    ///
    /// Reclaims space only. Expiry is already enforced on every read, so sweeping
    /// changes no answer a script can observe. A backend with native key TTL may
    /// return `Ok(0)`, since the server has already reaped on its behalf.
    ///
    /// # Returns
    ///
    /// How many records were dropped.
    ///
    /// # Errors
    ///
    /// Returns an error only when the transport fails.
    fn sweep(&mut self, now_ms: i64) -> Result<usize, String>;
}
