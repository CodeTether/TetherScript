//! String-keyed conveniences over the byte-oriented commands.
//!
//! The byte APIs are the real ones, because Redis keys and values are binary. These
//! wrappers exist so the common case — a UTF-8 key and a UTF-8 value — reads
//! cleanly, and they are lossy in exactly one documented direction: reading a value
//! that is not valid UTF-8 is an error rather than a silent replacement-character
//! substitution.

use super::connection::Connection;
use super::error::RedisError;
use super::options::SetOptions;

impl Connection {
    /// [`Connection::get`] with a `&str` key, decoding the value as UTF-8.
    ///
    /// # Arguments
    ///
    /// * `key` — Key as text.
    ///
    /// # Returns
    ///
    /// `Some(text)` when the key exists, `None` when it does not. As with
    /// [`Connection::get`], an empty stored string returns `Some(String::new())`,
    /// which is distinct from `None`.
    ///
    /// # Errors
    ///
    /// [`RedisError::UnexpectedType`] when the stored bytes are not valid UTF-8, so
    /// binary data is never silently mangled; use [`Connection::get`] for that.
    /// Server, transport, and protocol errors otherwise.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use tetherscript::redis::Connection;
    /// # fn run(connection: &mut Connection) -> Result<(), tetherscript::redis::RedisError> {
    /// let _token = connection.get_str("session:42")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_str(&mut self, key: &str) -> Result<Option<String>, RedisError> {
        match self.get(key.as_bytes())? {
            None => Ok(None),
            Some(bytes) => String::from_utf8(bytes).map(Some).map_err(|_| {
                RedisError::UnexpectedType(format!("GET {key}: value is not valid UTF-8"))
            }),
        }
    }

    /// [`Connection::set`] with `&str` key and value.
    ///
    /// # Arguments
    ///
    /// * `key` — Key as text.
    /// * `value` — Value as text.
    /// * `options` — `EX` and `NX` modifiers.
    ///
    /// # Returns
    ///
    /// `false` only when `NX` was requested and the key already existed.
    ///
    /// # Errors
    ///
    /// As [`Connection::set`].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use tetherscript::redis::{Connection, SetOptions};
    /// # fn run(connection: &mut Connection) -> Result<(), tetherscript::redis::RedisError> {
    /// connection.set_str("session:42", "token", &SetOptions::expiring(3600))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_str(
        &mut self,
        key: &str,
        value: &str,
        options: &SetOptions,
    ) -> Result<bool, RedisError> {
        self.set(key.as_bytes(), value.as_bytes(), options)
    }
}
