//! # Named failures for binary wire decoding
//!
//! A wire-format decoder parses bytes that arrived from a **network peer**. The
//! peer may be a well-behaved PostgreSQL server, a proxy that truncated a frame,
//! or something hostile. So every length is checked *before* it is trusted and
//! every failure is a named [`DecodeError`] — never a slice-index panic, never an
//! `unwrap`, never a silent zero-fill. A panic inside a decoder would take down
//! the whole host, so the invariant is: **no decoder in this module may panic on
//! any input byte sequence, of any length, including empty.**
//!
//! [`DecodeError::UnsupportedOid`] is deliberately its own variant rather than a
//! generic error string: it is the signal the integrator uses to fall back to the
//! existing text path, so adding a column of an unknown type degrades to text
//! rather than failing the whole query. Test that predicate with
//! [`DecodeError::needs_text_fallback`].

/// Why a binary field could not be decoded, or a parameter could not be encoded.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::DecodeError;
///
/// let truncated = DecodeError::Truncated {
///     what: "int4",
///     need: 4,
///     have: 2,
/// };
/// assert!(truncated.to_string().contains("int4"));
/// assert!(!truncated.needs_text_fallback());
///
/// // An unknown type OID is recoverable: decode it as text instead.
/// let unknown = DecodeError::UnsupportedOid { oid: 600 };
/// assert!(unknown.needs_text_fallback());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Fewer bytes remained than the layout requires.
    Truncated {
        /// The field or sub-field being read, for the message.
        what: &'static str,
        /// Bytes the layout demands.
        need: usize,
        /// Bytes actually available.
        have: usize,
    },
    /// The layout was satisfied but trailing bytes remained, so the field is not
    /// what the type OID claimed. Reported rather than ignored.
    Overlong {
        /// The field being read.
        what: &'static str,
        /// Bytes the layout consumed.
        expected: usize,
        /// Bytes the server actually sent.
        got: usize,
    },
    /// No binary decoder is registered for this type OID. **Recoverable:** the
    /// caller should re-read the column in text format.
    UnsupportedOid {
        /// The PostgreSQL type OID that has no binary decoder.
        oid: u32,
    },
    /// An array arrived with a dimension count this decoder will not handle.
    /// Rejected by name rather than misread as a flat array.
    UnsupportedDimensions {
        /// The dimension count the server sent.
        ndim: i32,
    },
    /// A text-ish field was not valid UTF-8.
    BadUtf8 {
        /// The field being decoded.
        what: &'static str,
    },
    /// A `numeric` sign word was none of the five documented values.
    BadNumericSign {
        /// The unrecognised sign word.
        sign: u16,
    },
    /// The bytes were the right length but not a legal value of the type.
    BadValue {
        /// The field being decoded.
        what: &'static str,
        /// What specifically was wrong.
        detail: String,
    },
}

impl DecodeError {
    /// Whether the caller should retry this column through the text path.
    ///
    /// # Returns
    ///
    /// `true` only for [`DecodeError::UnsupportedOid`]. Every other variant means
    /// the bytes themselves were wrong, and retrying as text would hide a real
    /// protocol fault.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::postgres::binary::DecodeError;
    ///
    /// assert!(DecodeError::UnsupportedOid { oid: 1_000_000 }.needs_text_fallback());
    /// assert!(!DecodeError::BadUtf8 { what: "text" }.needs_text_fallback());
    /// ```
    pub fn needs_text_fallback(&self) -> bool {
        matches!(self, DecodeError::UnsupportedOid { .. })
    }
}

impl std::error::Error for DecodeError {}

#[path = "binary_error_display.rs"]
mod display;
