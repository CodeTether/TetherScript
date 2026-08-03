//! Accessors that pull typed data out of a [`RespValue`].
//!
//! These are pure classification helpers with no failure mode; the fallible typed
//! extractions live in the `value_bulk` and `value_int` modules. Split from the
//! enum definition to keep one concern per file.

use super::value::RespValue;

impl RespValue {
    /// The RESP type name, for error messages.
    ///
    /// # Returns
    ///
    /// A stable lowercase label: `simple`, `error`, `integer`, `bulk`,
    /// `null-bulk`, `array`, or `null-array`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::redis::RespValue;
    ///
    /// assert_eq!(RespValue::Integer(1).type_name(), "integer");
    /// assert_eq!(RespValue::NullBulk.type_name(), "null-bulk");
    /// ```
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Simple(_) => "simple",
            Self::Error { .. } => "error",
            Self::Integer(_) => "integer",
            Self::Bulk(_) => "bulk",
            Self::NullBulk => "null-bulk",
            Self::Array(_) => "array",
            Self::NullArray => "null-array",
        }
    }

    /// Whether this reply is one of the two null forms.
    ///
    /// # Returns
    ///
    /// `true` for [`RespValue::NullBulk`] and [`RespValue::NullArray`]. An empty
    /// bulk string or empty array is **not** null; see [`RespValue`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::redis::RespValue;
    ///
    /// assert!(RespValue::NullBulk.is_null());
    /// assert!(!RespValue::Bulk(Vec::new()).is_null());
    /// ```
    pub fn is_null(&self) -> bool {
        matches!(self, Self::NullBulk | Self::NullArray)
    }
}
