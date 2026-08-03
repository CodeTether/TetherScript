//! `SET`, with optional `EX` and `NX`.

use super::commands_set_args::build;
use super::connection::Connection;
use super::error::RedisError;
use super::options::SetOptions;
use super::value::RespValue;

impl Connection {
    /// Write `value` at `key`, honouring `options`.
    ///
    /// # Arguments
    ///
    /// * `key` — Key bytes.
    /// * `value` — Value bytes. Binary-safe: CRLF, NUL, and non-UTF-8 all survive,
    ///   because arguments are length-prefixed rather than delimited.
    /// * `options` — `EX` expiry and `NX` guard; see [`SetOptions`].
    ///
    /// # Returns
    ///
    /// `true` when the value was stored. `false` only in the `NX` case where the
    /// key already existed: Redis answers that with the null bulk string rather
    /// than `+OK`, which is exactly why the null bulk string must be represented
    /// distinctly from an empty one. Without `NX`, the result is always `true`.
    ///
    /// # Errors
    ///
    /// * [`RedisError::Server`] when the server rejects the command, for instance
    ///   `EX` on a server too old to support it.
    /// * [`RedisError::UnexpectedType`] when the reply is neither `+OK` nor the
    ///   null bulk string.
    /// * [`RedisError::Transport`] or [`RedisError::Protocol`] on socket or framing
    ///   failure.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tetherscript::redis::{Config, Connection, SetOptions};
    ///
    /// # fn main() -> Result<(), tetherscript::redis::RedisError> {
    /// let mut connection = Connection::connect(&Config::default())?;
    ///
    /// // A session that expires in an hour.
    /// connection.set(b"session:42", b"token", &SetOptions::expiring(3600))?;
    ///
    /// // A lock: `false` means someone else holds it.
    /// let _acquired = connection.set(
    ///     b"lock:render",
    ///     b"owner",
    ///     &SetOptions { expire_seconds: Some(30), if_not_exists: true },
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set(
        &mut self,
        key: &[u8],
        value: &[u8],
        options: &SetOptions,
    ) -> Result<bool, RedisError> {
        let mut seconds = String::new();
        let args = build(key, value, options, &mut seconds);
        match self.command(&args)? {
            RespValue::Simple(_) => Ok(true),
            RespValue::NullBulk => Ok(false),
            other => Err(RedisError::UnexpectedType(format!(
                "SET: expected a status reply or null, got {}",
                other.type_name()
            ))),
        }
    }
}
