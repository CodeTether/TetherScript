//! RESP request framing primitives: bulk strings and array headers.
//!
//! # Why never build a command by string concatenation
//!
//! Redis accepts two request forms. The *inline* form is plain text terminated by
//! CRLF, where arguments are split on whitespace:
//!
//! ```text
//! SET session:42 hunter2\r\n
//! ```
//!
//! The *unified* (RESP array) form is length-prefixed:
//!
//! ```text
//! *3\r\n$3\r\nSET\r\n$10\r\nsession:42\r\n$7\r\nhunter2\r\n
//! ```
//!
//! Inline syntax is exactly how command injection happens. Because the inline
//! parser treats whitespace and CRLF as structure, any value containing them
//! *becomes* structure. A caller that writes `format!("SET {key} {value}\r\n")`
//! with `value = "x\r\nFLUSHALL"` has not sent one command with an awkward value;
//! it has sent two commands, the second of which destroys the database. A single
//! space in a value is enough to shift every later argument by one position, and
//! `\r\nCONFIG SET requirepass ...` is enough to take the server over.
//!
//! The unified form is immune because a length prefix, not a delimiter, decides
//! where each argument ends: the server reads exactly `$7` bytes, so a payload of
//! `x\r\nFLU` is just a seven-byte string. That is why `encode_command` is the only
//! supported way to build a request here, why it takes a slice of byte slices
//! rather than a pre-joined line, and why no function in this module accepts a
//! whole command string.

/// The RESP terminator. Two bytes, after every line and every bulk body.
pub(super) const CRLF: &[u8] = b"\r\n";

/// Append one bulk string: `$<len>\r\n<payload>\r\n`.
///
/// The payload is copied verbatim, so CRLF inside it is data, not structure.
pub(super) fn push_bulk(out: &mut Vec<u8>, payload: &[u8]) {
    out.push(b'$');
    out.extend_from_slice(payload.len().to_string().as_bytes());
    out.extend_from_slice(CRLF);
    out.extend_from_slice(payload);
    out.extend_from_slice(CRLF);
}

/// Append an array header: `*<count>\r\n`.
pub(super) fn push_header(out: &mut Vec<u8>, count: usize) {
    out.push(b'*');
    out.extend_from_slice(count.to_string().as_bytes());
    out.extend_from_slice(CRLF);
}
