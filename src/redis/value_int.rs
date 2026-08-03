//! Integer and status extraction from a [`RespValue`].
//!
//! `INCR`, `DEL`, `EXISTS`, `EXPIRE`, and `TTL` all answer with `:<n>\r\n`, and
//! `SET`/`PING` answer with `+OK`/`+PONG`. Kept separate from the bulk-string
//! accessors so each file has one responsibility.

use super::error::RedisError;
use super::value::RespValue;

impl RespValue {
    /// Read an integer reply.
    ///
    /// # Arguments
    ///
    /// * `context` — Command name, included in the error message.
    ///
    /// # Returns
    ///
    /// The signed 64-bit value. `TTL` uses `-1` for *no expiry* and `-2` for *no
    /// such key*, so negatives are meaningful and are not normalised away here.
    ///
    /// # Errors
    ///
    /// [`RedisError::UnexpectedType`] when the reply is not an integer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::redis::RespValue;
    ///
    /// assert_eq!(RespValue::Integer(-2).integer("TTL").unwrap(), -2);
    /// assert!(RespValue::NullBulk.integer("TTL").is_err());
    /// ```
    pub fn integer(&self, context: &str) -> Result<i64, RedisError> {
        match self {
            Self::Integer(value) => Ok(*value),
            other => Err(RedisError::UnexpectedType(format!(
                "{context}: expected an integer, got {}",
                other.type_name()
            ))),
        }
    }

    /// Read a simple status reply such as `OK` or `PONG`.
    ///
    /// # Arguments
    ///
    /// * `context` — Command name, included in the error message.
    ///
    /// # Returns
    ///
    /// The status text without its `+` prefix or trailing CRLF.
    ///
    /// # Errors
    ///
    /// [`RedisError::UnexpectedType`] when the reply is not a simple string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::redis::RespValue;
    ///
    /// let ok = RespValue::Simple("OK".into());
    /// assert_eq!(ok.simple("SET").unwrap(), "OK");
    /// ```
    pub fn simple(&self, context: &str) -> Result<&str, RedisError> {
        match self {
            Self::Simple(text) => Ok(text),
            other => Err(RedisError::UnexpectedType(format!(
                "{context}: expected a status reply, got {}",
                other.type_name()
            ))),
        }
    }
}
