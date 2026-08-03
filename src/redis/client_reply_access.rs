//! Reading typed data out of a [`Reply`].
//!
//! Two kinds of accessor, one concern: classification helpers that cannot fail,
//! and the typed extractions the command layer needs. Every failure here is
//! [`ClientError::UnexpectedType`], which by design does *not* discard the
//! connection: a well-formed reply of a surprising type still consumed exactly one
//! reply from the stream.

use super::error::ClientError;
use super::reply::Reply;

impl Reply {
    /// The reply type name, for error messages.
    ///
    /// # Returns
    ///
    /// A stable lowercase label: `status`, `error`, `integer`, `bulk`, `nil`, or
    /// `array`.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Status(_) => "status",
            Self::Error { .. } => "error",
            Self::Integer(_) => "integer",
            Self::Bulk(_) => "bulk",
            Self::Nil => "nil",
            Self::Array(_) => "array",
        }
    }

    /// Whether this reply is the null reply.
    ///
    /// # Returns
    ///
    /// `true` only for [`Reply::Nil`]. An empty bulk string is not null; see
    /// [`Reply`].
    pub fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }

    /// Take the payload of a bulk string, mapping the null reply to `None`.
    ///
    /// # Arguments
    ///
    /// * `context` — Command name, so a mismatch names the caller.
    ///
    /// # Returns
    ///
    /// `None` only for [`Reply::Nil`]. A key holding the empty string yields
    /// `Some(vec![])`, keeping a miss distinguishable from an empty value.
    ///
    /// # Errors
    ///
    /// [`ClientError::UnexpectedType`] when the reply is neither bulk nor nil.
    pub fn optional_bulk(self, context: &str) -> Result<Option<Vec<u8>>, ClientError> {
        match self {
            Self::Bulk(bytes) => Ok(Some(bytes)),
            Self::Nil => Ok(None),
            other => Err(other.mismatch(context, "a bulk string or nil")),
        }
    }

    /// Take an integer reply.
    ///
    /// # Arguments
    ///
    /// * `context` — Command name for the error message.
    ///
    /// # Returns
    ///
    /// The signed integer, including Redis' negative TTL sentinels.
    ///
    /// # Errors
    ///
    /// [`ClientError::UnexpectedType`] for any other reply type.
    pub fn integer(self, context: &str) -> Result<i64, ClientError> {
        match self {
            Self::Integer(value) => Ok(value),
            other => Err(other.mismatch(context, "an integer")),
        }
    }

    /// Build the type-mismatch error for `context`, naming what was wanted.
    fn mismatch(&self, context: &str, wanted: &str) -> ClientError {
        ClientError::UnexpectedType(format!(
            "{context}: expected {wanted}, got {}",
            self.type_name()
        ))
    }
}
