//! # Encoding commands to send
//!
//! A client only ever needs to send one shape: the *unified request protocol*, an
//! array of bulk strings whose first element is the command name and whose rest
//! are its arguments. Every modern Redis command, `HELLO 3` included, is sent this
//! way, so this module encodes that form and nothing else. There is deliberately
//! no general "encode any [`Reply`](super::reply::Reply)" function: a client that
//! could emit a map or a push frame would only be able to emit frames no server
//! accepts.
//!
//! Arguments are byte slices, not `&str`, for the same reason bulk strings decode
//! to bytes: keys and values are binary safe, and a caller storing a serialised
//! blob must not have to pretend it is text.

use super::limits::MAX_BULK_LEN;

/// Encode a command as a RESP array of bulk strings.
///
/// # Arguments
///
/// * `args` — the command name followed by its arguments, each an opaque byte
///   string. Case of the command name is not normalised; Redis accepts either.
///   Note that byte-string literals are fixed-size arrays, so call sites need
///   `.as_slice()` (or `&b"..."[..]`) to build a `&[&[u8]]`.
///
/// # Returns
///
/// The complete request bytes, ready to write to a socket.
///
/// # Panics
///
/// Panics if `args` is empty, or if any argument is longer than [`MAX_BULK_LEN`].
/// Both are programming errors in the calling client rather than protocol
/// conditions: there is no such thing as a command with no name, and a server
/// would refuse an over-long argument anyway.
///
/// # Examples
///
/// ```rust
/// use tetherscript::resp::codec::encode_command;
///
/// let request = encode_command(&[b"SET".as_slice(), b"key", b"value"]);
/// assert_eq!(request, b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n".to_vec());
/// ```
///
/// Binary-safe arguments survive intact, CRLF included:
///
/// ```rust
/// use tetherscript::resp::codec::encode_command;
///
/// let request = encode_command(&[b"SET".as_slice(), b"k", b"a\r\nb"]);
/// assert_eq!(request, b"*3\r\n$3\r\nSET\r\n$1\r\nk\r\n$4\r\na\r\nb\r\n".to_vec());
/// ```
pub fn encode_command(args: &[&[u8]]) -> Vec<u8> {
    assert!(!args.is_empty(), "resp: a command needs at least a name");
    let mut out = Vec::new();
    out.extend_from_slice(format!("*{}\r\n", args.len()).as_bytes());
    for arg in args {
        assert!(
            arg.len() as i64 <= MAX_BULK_LEN,
            "resp: argument of {} bytes exceeds the {MAX_BULK_LEN}-byte limit",
            arg.len()
        );
        out.extend_from_slice(format!("${}\r\n", arg.len()).as_bytes());
        out.extend_from_slice(arg);
        out.extend_from_slice(b"\r\n");
    }
    out
}
