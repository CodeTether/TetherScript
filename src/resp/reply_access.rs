//! # Reading a decoded reply
//!
//! Convenience accessors on [`Reply`] for the questions a client layer asks
//! constantly: "is this a cache miss?", "give me the payload bytes", "what error
//! code did the server use?". They are inherent methods rather than
//! `From`/`TryFrom` conversions so that no accessor can silently paper over the
//! [`Reply::Nil`]-versus-empty distinction.

use super::reply::Reply;

impl Reply {
    /// Whether this reply is the absent value.
    ///
    /// # Returns
    ///
    /// `true` only for [`Reply::Nil`]. An empty bulk string, an empty array, and
    /// a zero integer are all present values and return `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::resp::codec::Reply;
    ///
    /// assert!(Reply::Nil.is_nil());
    /// assert!(!Reply::Bulk(Vec::new()).is_nil());
    /// assert!(!Reply::Array(Vec::new()).is_nil());
    /// ```
    pub fn is_nil(&self) -> bool {
        matches!(self, Reply::Nil)
    }

    /// The payload bytes of a string-shaped reply.
    ///
    /// # Returns
    ///
    /// `Some` for [`Reply::Bulk`], [`Reply::Verbatim`] (payload only, without the
    /// format hint), and [`Reply::Simple`]. `None` for everything else,
    /// including [`Reply::Nil`], so a miss can never be read as `b""`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::resp::codec::Reply;
    ///
    /// assert_eq!(Reply::Bulk(b"hi".to_vec()).as_bytes(), Some(&b"hi"[..]));
    /// assert_eq!(Reply::Nil.as_bytes(), None);
    /// ```
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Reply::Bulk(bytes) => Some(bytes.as_slice()),
            Reply::Verbatim { text, .. } => Some(text.as_slice()),
            Reply::Simple(text) => Some(text.as_bytes()),
            _ => None,
        }
    }

    /// The leading uppercase word of an error reply, such as `WRONGTYPE`.
    ///
    /// # Returns
    ///
    /// `Some(code)` for [`Reply::Error`], where `code` is the text up to the
    /// first space (the whole text when there is no space). `None` for every
    /// other variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::resp::codec::Reply;
    ///
    /// let reply = Reply::Error("WRONGTYPE Operation against a key".into());
    /// assert_eq!(reply.error_code(), Some("WRONGTYPE"));
    /// assert_eq!(Reply::Integer(1).error_code(), None);
    /// ```
    pub fn error_code(&self) -> Option<&str> {
        match self {
            Reply::Error(text) => Some(text.split(' ').next().unwrap_or(text.as_str())),
            _ => None,
        }
    }
}
