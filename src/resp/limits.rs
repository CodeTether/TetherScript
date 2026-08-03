//! # RESP decoding bounds
//!
//! A decoder that trusts the length prefix a peer sends is a denial-of-service
//! primitive: `$9999999999\r\n` would ask us to reserve ten gigabytes before a
//! single payload byte arrived. Every length in this codec is therefore checked
//! against a constant in this module *before* it is used for anything, and a
//! value outside the bound is reported as
//! [`DecodeError::Malformed`](super::error::DecodeError::Malformed) rather than
//! attempted.
//!
//! The bounds are deliberately public so a client layer can quote them in its
//! own error messages, and can size its read buffer knowing what the codec will
//! ever agree to parse.
//!
//! | Constant | Value | Applies to |
//! |---|---|---|
//! | [`MAX_BULK_LEN`] | 512 MiB | `$` bulk strings, `=` verbatim strings |
//! | [`MAX_AGGREGATE_LEN`] | 1 Mi elements | `*` array, `~` set, `>` push, `%` map |
//! | [`MAX_DEPTH`] | 32 | nesting of aggregates inside aggregates |
//! | [`MAX_LINE_LEN`] | 64 KiB | any CRLF-terminated line, headers included |
//!
//! Note that hitting a bound is not the same as allocating up to it. Bulk
//! payloads are only copied out once the bytes are already in the caller's
//! buffer, and aggregates are grown by pushing rather than by reserving the
//! announced element count, so a header claiming a million elements costs
//! nothing until a million elements actually arrive.

/// Largest accepted `$`/`=` payload, in **bytes**, matching the Redis server
/// default for `proto-max-bulk-len` (512 MiB).
pub const MAX_BULK_LEN: i64 = 512 * 1024 * 1024;

/// Largest accepted element count for `*`, `~`, `>` and `%` headers.
///
/// For a map this bounds the number of *pairs*, so at most `2 * MAX_AGGREGATE_LEN`
/// values are decoded for one map header.
pub const MAX_AGGREGATE_LEN: i64 = 1024 * 1024;

/// Largest accepted nesting depth of aggregates.
///
/// The decoder recurses once per aggregate level, so this bound is what keeps a
/// reply of nothing but `*1\r\n` repeated forever from overflowing the stack.
pub const MAX_DEPTH: usize = 32;

/// Largest accepted CRLF-terminated line, in bytes, excluding the CRLF.
///
/// This covers type headers and the scalar types whose payload *is* the line
/// (`+`, `-`, `:`, `,`, `#`, `(`). It does not apply to bulk payloads, which are
/// length-prefixed and bounded by [`MAX_BULK_LEN`] instead. Without it, a peer
/// that never sends a CRLF would keep the caller in
/// [`DecodeError::Incomplete`](super::error::DecodeError::Incomplete) forever
/// while its buffer grew without limit.
pub const MAX_LINE_LEN: usize = 64 * 1024;
