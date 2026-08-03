//! # The decoded reply type
//!
//! [`Reply`] is the owned, allocation-complete form of one RESP value. It is
//! deliberately **not** called `Value`: this crate already has
//! [`crate::value::Value`], the tetherscript runtime value, and the two are
//! different things at different layers. A `Reply` is what came off a socket in
//! Redis' wire vocabulary (verbatim strings, push frames, big numbers); a
//! `Value` is what a tetherscript program can hold. Mapping one to the other is
//! the client layer's job, and giving them the same name would make every `use`
//! in that layer a coin toss.
//!
//! ## Examples
//!
//! ```rust
//! use tetherscript::resp::codec::{decode, Reply};
//!
//! let (reply, _) = decode(b"$5\r\nhello\r\n").unwrap();
//! assert_eq!(reply, Reply::Bulk(b"hello".to_vec()));
//!
//! // A missing key is `Nil`, and `Nil` is not an empty string.
//! let (missing, _) = decode(b"$-1\r\n").unwrap();
//! let (empty, _) = decode(b"$0\r\n\r\n").unwrap();
//! assert_eq!(missing, Reply::Nil);
//! assert_eq!(empty, Reply::Bulk(Vec::new()));
//! assert_ne!(missing, empty);
//! ```

/// One decoded RESP value.
///
/// # Examples
///
/// ```rust
/// use tetherscript::resp::codec::Reply;
///
/// let reply = Reply::Integer(1);
/// match reply {
///     Reply::Integer(n) => assert_eq!(n, 1),
///     Reply::Error(ref text) => panic!("server said: {text}"),
///     _ => panic!("unexpected reply"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Reply {
    /// `+OK\r\n` — a status line. Never contains CR or LF.
    Simple(String),
    /// `-WRONGTYPE ...\r\n` — an error *reply*, carried as data rather than as a
    /// [`DecodeError`](super::error::DecodeError).
    ///
    /// The transport is healthy when this arrives: the server understood the
    /// request and answered it, so the connection stays usable and the next
    /// pipelined reply still has to be read. Only the client layer knows whether
    /// a given error is fatal, expected (`BUSYGROUP` on a re-created stream), or
    /// worth retrying (`MOVED`/`ASK` in cluster mode), so the codec refuses to
    /// decide and hands the text up intact, prefix included.
    Error(String),
    /// `:42\r\n` — a signed 64-bit integer.
    Integer(i64),
    /// `$5\r\nhello\r\n` — a length-prefixed byte payload.
    ///
    /// Held as bytes, not `String`: the length is a byte count and Redis values
    /// are binary safe, so a payload may embed CRLF or not be UTF-8 at all.
    Bulk(Vec<u8>),
    /// `$-1\r\n` or `*-1\r\n` (and RESP3 `_\r\n`) — the absent value.
    ///
    /// Distinct from `Bulk(vec![])` and from `Array(vec![])`. A cache layer that
    /// flattens the two turns "no entry" into "entry that is the empty string".
    Nil,
    /// `*2\r\n...` — an ordered sequence.
    Array(Vec<Reply>),
    /// `%1\r\n<key><value>` — RESP3 map, kept as ordered pairs.
    ///
    /// A `Vec` rather than a `HashMap` because keys are arbitrary `Reply`s
    /// (not all hashable), duplicates are possible on the wire, and `HELLO`
    /// replies read better in field order.
    Map(Vec<(Reply, Reply)>),
    /// `~2\r\n...` — RESP3 set. Uniqueness is the server's claim, not enforced.
    Set(Vec<Reply>),
    /// `,3.14\r\n` — RESP3 double, including `inf`, `-inf`, and `nan`.
    Double(f64),
    /// `#t\r\n` / `#f\r\n` — RESP3 boolean.
    Boolean(bool),
    /// `(3492890328409238509324850943850943825024385\r\n` — RESP3 big number,
    /// kept as its decimal text because it does not fit an `i64`.
    BigNumber(String),
    /// `=15\r\ntxt:Some string\r\n` — RESP3 verbatim string.
    Verbatim {
        /// The three-byte hint before the colon, such as `txt` or `mkd`.
        format: String,
        /// The payload after the colon, as bytes for the same reason as
        /// [`Reply::Bulk`].
        text: Vec<u8>,
    },
    /// `>3\r\n...` — RESP3 out-of-band push frame (pub/sub, invalidation).
    ///
    /// Kept distinct from [`Reply::Array`] so a client can route it to a handler
    /// instead of mistaking it for the answer to the command in flight.
    Push(Vec<Reply>),
}
