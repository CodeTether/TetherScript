//! # RESP: the Redis wire protocol codec
//!
//! Encodes commands and decodes replies for RESP2 plus the RESP3 types a server
//! starts sending once the client has issued `HELLO 3`. This module is **only**
//! the codec: it opens no sockets, owns no connection, and knows nothing about
//! pooling, capabilities, or Redis semantics. It turns bytes into [`Reply`]
//! values and command arguments into bytes; a client layer built on top supplies
//! everything else.
//!
//! ## Quick start
//!
//! ```rust
//! use tetherscript::resp::codec::{decode, encode_command, Reply};
//!
//! // Sending: a command is an array of bulk strings, the only form a client emits.
//! let request = encode_command(&[b"GET".as_slice(), b"session:42"]);
//! assert_eq!(request, b"*2\r\n$3\r\nGET\r\n$10\r\nsession:42\r\n".to_vec());
//!
//! // Receiving: the reply plus how many bytes of the buffer it used.
//! let (reply, consumed) = decode(b"$5\r\nhello\r\n").unwrap();
//! assert_eq!(reply, Reply::Bulk(b"hello".to_vec()));
//! assert_eq!(consumed, 11);
//! ```
//!
//! ## Incremental decoding
//!
//! TCP does not deliver replies, it delivers bytes, so a reply may be split
//! across any number of reads. [`decode`] therefore borrows the buffer, never
//! drains it, and separates the two failure modes strictly:
//!
//! - [`DecodeError::Incomplete`] — a valid prefix. Read more and call again with
//!   the same buffer, grown. **Nothing was consumed.**
//! - [`DecodeError::Malformed`] — not RESP, or past a bound. Unrecoverable,
//!   because framing depends on the bytes that turned out to be wrong; drop the
//!   connection.
//!
//! On success the returned count is how many leading bytes formed the reply, so a
//! pipelined buffer is drained one reply at a time:
//!
//! ```rust
//! use tetherscript::resp::codec::{decode, DecodeError, Reply};
//!
//! let mut buf: &[u8] = b":1\r\n:2\r\n";
//! let (first, used) = decode(buf).unwrap();
//! assert_eq!((first, used), (Reply::Integer(1), 4));
//! buf = &buf[used..];
//! assert_eq!(decode(buf).unwrap().0, Reply::Integer(2));
//!
//! // A prefix of a reply is Incomplete, never Malformed.
//! assert_eq!(decode(b":1").unwrap_err(), DecodeError::Incomplete);
//! ```
//!
//! ## Bounds
//!
//! Lengths come from the peer, so every one of them is checked before use: see
//! [`limits`] for the bulk-length, element-count, depth and line-length caps and
//! why each exists. A hostile `$9999999999` is rejected, not allocated.
//!
//! ## Errors are data
//!
//! A `-WRONGTYPE ...` reply decodes to [`Reply::Error`], not to a
//! [`DecodeError`]. The transport is healthy when one arrives, and only the client
//! layer can judge whether the error is fatal, expected, or a redirect worth
//! following. See [`Reply::Error`] for the full argument.
//!
//! ## Architecture
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`limits`] | The size and depth bounds, and their rationale |
//! | [`error`] | `Incomplete` versus `Malformed` |
//! | [`reply`], `reply_access` | The decoded value type and its accessors |
//! | `crlf`, `cursor` | Framing: terminator search and buffer position |
//! | `scalar` | Line payloads: text, integer, double, boolean, big number |
//! | `bulk` | Length-prefixed `$` and `=` payloads |
//! | `aggregate`, `aggregate_header` | Counted `*`, `~`, `>`, `%`; count and depth bounds |
//! | `parse` | Type-byte dispatch |
//! | `encode` | The command request form |

// The concern modules are siblings of this file inside `src/resp/`, so each
// declaration carries an explicit path attribute. Without it Rust would look
// under `src/resp/codec/`, and a `codec/` directory would repeat the module name
// in every path for no gain.
#[path = "error.rs"]
pub mod error;
#[path = "limits.rs"]
pub mod limits;
#[path = "reply.rs"]
pub mod reply;

#[path = "aggregate.rs"]
mod aggregate;
#[path = "aggregate_header.rs"]
mod aggregate_header;
#[path = "bulk.rs"]
mod bulk;
#[path = "crlf.rs"]
mod crlf;
#[path = "cursor.rs"]
mod cursor;
#[path = "encode.rs"]
mod encode;
#[path = "parse.rs"]
mod parse;
#[path = "reply_access.rs"]
mod reply_access;
#[path = "scalar.rs"]
mod scalar;

pub use encode::encode_command;
pub use error::DecodeError;
pub use reply::Reply;

/// Decode the first complete reply at the front of `buf`.
///
/// # Arguments
///
/// * `buf` — bytes received so far. Borrowed and never modified; may hold a
///   partial reply, exactly one reply, or several pipelined replies.
///
/// # Returns
///
/// `Ok((reply, consumed))`, where `consumed` is the number of leading bytes the
/// reply occupied. Any bytes past `consumed` belong to the next reply and are
/// left for the caller.
///
/// # Errors
///
/// [`DecodeError::Incomplete`] if `buf` is a valid but unfinished prefix — retry
/// with more bytes and do not advance the buffer. [`DecodeError::Malformed`] if
/// `buf` violates RESP or a bound in [`limits`]; the message names the problem.
///
/// # Examples
///
/// ```rust
/// use tetherscript::resp::codec::{decode, Reply};
///
/// // Trailing bytes are reported, not swallowed.
/// let (reply, consumed) = decode(b"+OK\r\n+NEXT\r\n").unwrap();
/// assert_eq!(reply, Reply::Simple("OK".into()));
/// assert_eq!(consumed, 5);
///
/// // Bulk payloads are bytes, so an embedded CRLF is just data.
/// let (reply, _) = decode(b"$4\r\na\r\nb\r\n").unwrap();
/// assert_eq!(reply, Reply::Bulk(b"a\r\nb".to_vec()));
/// ```
pub fn decode(buf: &[u8]) -> Result<(Reply, usize), DecodeError> {
    let mut cursor = cursor::Cursor::new(buf);
    let reply = parse::value(&mut cursor, 0)?;
    Ok((reply, cursor.position()))
}
