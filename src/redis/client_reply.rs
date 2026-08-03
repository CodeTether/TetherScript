//! The reply value model at the codec boundary.
//!
//! This is the client's own vocabulary type, not the RESP codec's. Keeping it
//! separate is what makes [`RespCodec`](super::codec::RespCodec) a *boundary*: the
//! adapter maps whatever the codec produces onto this enum once, and no command
//! method ever names a codec type.
//!
//! ## Absence is a variant, not an empty value
//!
//! [`Reply::Nil`] (`$-1\r\n` on the wire) is **not** `Reply::Bulk(vec![])`
//! (`$0\r\n\r\n`). The first means *the key does not exist*; the second means *the
//! key exists and holds the empty string*. Collapsing them turns a cache miss
//! into a cached empty page and a logged-out user into one with a blank session,
//! so they are separate variants and compare unequal all the way out to
//! `Option<Vec<u8>>` at the public surface.
//!
//! Bulk payloads are `Vec<u8>`, not `String`: Redis strings are binary-safe and
//! may contain CRLF or invalid UTF-8.

/// A decoded Redis reply, in the client's terms.
///
/// # Examples
///
/// ```rust,ignore
/// use tetherscript::redis::client::Reply;
///
/// assert_ne!(Reply::Nil, Reply::Bulk(Vec::new()));
/// assert_eq!(Reply::Integer(-2).type_name(), "integer");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    /// A status line such as `+OK`.
    Status(String),
    /// An error reply. Carried as a value so it can sit nested in an array; the
    /// exchange layer promotes a top-level one to
    /// [`ClientError::Server`](super::error::ClientError::Server).
    Error {
        /// Leading token, e.g. `ERR` or `WRONGTYPE`.
        kind: String,
        /// Remainder of the error line.
        message: String,
    },
    /// A 64-bit signed integer reply.
    Integer(i64),
    /// A binary-safe string. May legitimately be empty.
    Bulk(Vec<u8>),
    /// The null reply: the value or result is absent.
    Nil,
    /// An array reply, possibly nested.
    Array(Vec<Reply>),
}
