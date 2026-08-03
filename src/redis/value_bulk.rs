//! Typed extraction from a [`RespValue`], for the command layer.
//!
//! These are the conversions the command layer needs: bytes out of a bulk
//! string, an integer out of an integer reply, and an optional bulk for the
//! `GET`-shaped commands where absence is a legitimate answer.

use super::error::RedisError;
use super::value::RespValue;

impl RespValue {
    /// Borrow the payload of a bulk string.
    ///
    /// # Arguments
    ///
    /// * `context` — Command name, used in the error message so a mismatch names
    ///   the caller rather than just the type.
    ///
    /// # Returns
    ///
    /// The raw bytes. An empty bulk string yields an empty slice.
    ///
    /// # Errors
    ///
    /// [`RedisError::UnexpectedType`] for any other variant, including
    /// [`RespValue::NullBulk`]; use [`RespValue::optional_bulk`] when absence is
    /// expected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::redis::RespValue;
    ///
    /// let value = RespValue::Bulk(b"hello".to_vec());
    /// assert_eq!(value.bulk("GET").unwrap(), b"hello".as_slice());
    /// assert!(RespValue::NullBulk.bulk("GET").is_err());
    /// ```
    pub fn bulk(&self, context: &str) -> Result<&[u8], RedisError> {
        match self {
            Self::Bulk(bytes) => Ok(bytes),
            other => Err(RedisError::UnexpectedType(format!(
                "{context}: expected a bulk string, got {}",
                other.type_name()
            ))),
        }
    }

    /// Borrow a bulk payload, mapping the null bulk string to `None`.
    ///
    /// # Arguments
    ///
    /// * `context` — Command name for the error message.
    ///
    /// # Returns
    ///
    /// `None` only for [`RespValue::NullBulk`], so `Some(&[])` still faithfully
    /// reports a key holding the empty string.
    ///
    /// # Errors
    ///
    /// [`RedisError::UnexpectedType`] when the reply is neither a bulk string nor
    /// the null bulk string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::redis::RespValue;
    ///
    /// assert_eq!(RespValue::NullBulk.optional_bulk("GET").unwrap(), None);
    /// let empty = RespValue::Bulk(Vec::new());
    /// assert_eq!(empty.optional_bulk("GET").unwrap(), Some(b"".as_slice()));
    /// ```
    pub fn optional_bulk(&self, context: &str) -> Result<Option<&[u8]>, RedisError> {
        match self {
            Self::NullBulk => Ok(None),
            Self::Bulk(bytes) => Ok(Some(bytes.as_slice())),
            other => Err(RedisError::UnexpectedType(format!(
                "{context}: expected a bulk string or null, got {}",
                other.type_name()
            ))),
        }
    }
}
