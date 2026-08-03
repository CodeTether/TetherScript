//! Hard ceilings applied to every length a peer declares.
//!
//! RESP is length-prefixed, which means a reply can *ask* the client to allocate
//! before a single payload byte has arrived. A corrupt or hostile server that
//! sends `$9223372036854775807\r\n` would otherwise drive an unbounded
//! allocation and take the process down with an OOM, without ever sending the
//! data. The decoder therefore rejects any declared length above these limits as
//! a [`RedisError::Protocol`](super::error::RedisError::Protocol) *before*
//! reserving memory.
//!
//! The numbers are chosen to sit at or just past what a real server can produce,
//! so a legitimate reply is never refused:
//!
//! | Limit | Value | Rationale |
//! |---|---|---|
//! | [`MAX_BULK_LEN`] | 512 MiB | Redis' own maximum string size. |
//! | [`MAX_ARRAY_LEN`] | 1 Mi elements | Bounds `LRANGE`-style multi-bulk fanout. |
//! | [`MAX_LINE_LEN`] | 64 KiB | A status, error, or length line is tiny. |

/// Largest bulk string this client will accept, in bytes.
///
/// Matches the 512 MiB `proto-max-bulk-len` ceiling of Redis itself, so anything
/// larger is definitionally not a well-formed reply.
pub const MAX_BULK_LEN: usize = 512 * 1024 * 1024;

/// Largest number of elements accepted in a single RESP array.
pub const MAX_ARRAY_LEN: usize = 1024 * 1024;

/// Largest CRLF-terminated control line accepted.
///
/// Bounds the scan for the terminator too: without it, a peer that never sends
/// `\r\n` would make the read buffer grow forever while the decoder politely
/// asked for more bytes.
pub const MAX_LINE_LEN: usize = 64 * 1024;

/// Deepest nesting accepted while decoding arrays.
///
/// Arrays may contain arrays, so a reply of nothing but `*1\r\n` repeated would
/// recurse until the stack overflowed. Depth is capped instead.
pub const MAX_DEPTH: usize = 32;
