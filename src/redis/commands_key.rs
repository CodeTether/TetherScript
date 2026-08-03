//! `DEL` and `EXISTS`.
//!
//! Both are variadic and both answer with a *count*, not a boolean. Preserving the
//! count matters: `EXISTS k k` legitimately answers `2`, and `DEL` over a batch
//! reports how many of the keys were actually present, which is the difference
//! between an idempotent no-op and a real deletion.

use super::connection::Connection;
use super::error::RedisError;

impl Connection {
    /// Delete one or more keys.
    ///
    /// # Arguments
    ///
    /// * `keys` — Keys to remove. Must be non-empty.
    ///
    /// # Returns
    ///
    /// How many keys existed and were removed; `0` when none did, which is not an
    /// error.
    ///
    /// # Errors
    ///
    /// [`RedisError::Protocol`] when `keys` is empty, since Redis would reject the
    /// arity itself. Otherwise [`RedisError::Server`], [`RedisError::Transport`],
    /// or [`RedisError::Protocol`].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tetherscript::redis::{Config, Connection};
    ///
    /// # fn main() -> Result<(), tetherscript::redis::RedisError> {
    /// let mut connection = Connection::connect(&Config::default())?;
    /// let removed = connection.del(&[&b"session:42"[..], &b"session:43"[..]])?;
    /// println!("removed {removed}");
    /// # Ok(())
    /// # }
    /// ```
    pub fn del(&mut self, keys: &[&[u8]]) -> Result<i64, RedisError> {
        self.keyed_count("DEL", keys)
    }

    /// Count how many of `keys` exist.
    ///
    /// # Arguments
    ///
    /// * `keys` — Keys to test. Must be non-empty. A repeated key is counted each
    ///   time it appears, matching Redis' own semantics.
    ///
    /// # Returns
    ///
    /// The number of existing keys.
    ///
    /// # Errors
    ///
    /// As [`Connection::del`].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use tetherscript::redis::{Config, Connection};
    /// # fn run(connection: &mut Connection) -> Result<(), tetherscript::redis::RedisError> {
    /// if connection.exists(&[&b"session:42"[..]])? > 0 {
    ///     println!("still logged in");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn exists(&mut self, keys: &[&[u8]]) -> Result<i64, RedisError> {
        self.keyed_count("EXISTS", keys)
    }
}
