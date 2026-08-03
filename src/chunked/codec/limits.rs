//! Hard bounds for the chunked codec.
//!
//! Chunked bodies are attacker-controlled bytes. Every length in the wire format is
//! self-described, so an unbounded parser is a memory-exhaustion primitive: `FFFFFFFF\r\n`
//! asks a naive decoder to reserve 4 GiB. Every bound below is therefore a *refusal*
//! limit, not a hint, and exceeding one is [`ChunkedError::Malformed`], never
//! [`ChunkedError::Incomplete`] — an over-long claim is decidable immediately and must
//! not keep a connection alive while the peer sends more.
//!
//! [`ChunkedError::Malformed`]: super::ChunkedError::Malformed
//! [`ChunkedError::Incomplete`]: super::ChunkedError::Incomplete

/// Largest payload a single chunk may carry: 1 MiB.
///
/// A size line claiming more is rejected before any allocation happens.
pub const MAX_CHUNK_BYTES: usize = 1 << 20;

/// Largest total decoded payload across all chunks of one body: 8 MiB.
///
/// Checked *before* each chunk is appended, so a body that would exceed it never
/// materialises in memory.
pub const MAX_BODY_BYTES: usize = 8 << 20;

/// Largest chunk-size line, including any `;ext=value` extensions: 256 bytes.
///
/// Bounds the scan for CRLF, so a peer cannot force an unbounded search by never
/// sending a line terminator.
pub const MAX_SIZE_LINE_BYTES: usize = 256;

/// Largest single trailer field line: 1024 bytes.
pub const MAX_TRAILER_LINE_BYTES: usize = 1024;

/// Largest total trailer section, terminating empty line included: 4096 bytes.
pub const MAX_TRAILER_BYTES: usize = 4096;

/// Largest number of trailer fields after the zero chunk: 16.
pub const MAX_TRAILERS: usize = 16;
