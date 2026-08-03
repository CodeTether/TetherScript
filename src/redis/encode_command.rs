//! The public request encoder.
//!
//! One entry point, [`encode_command`], which is the only supported way to build
//! a Redis request in this crate. See the `encode` module for why the inline
//! command form is not offered at all: a value containing CRLF would become a
//! second command.

use super::encode::{push_bulk, push_header};
use super::error::RedisError;
use super::limits::{MAX_ARRAY_LEN, MAX_BULK_LEN};

/// Encode a command as a RESP array of bulk strings.
///
/// # Arguments
///
/// * `args` — The command name followed by its arguments, each as raw bytes. The
///   name is just `args[0]`; nothing is parsed or split, so an argument may
///   contain spaces, CRLF, NUL bytes, or invalid UTF-8 and still arrives as one
///   argument.
///
/// # Returns
///
/// The exact bytes to write to the socket, ready as a single `write_all`.
///
/// # Errors
///
/// * [`RedisError::Protocol`] when `args` is empty — a command needs a name.
/// * [`RedisError::Protocol`] when the argument count or any argument length
///   exceeds the documented maximum in the `limits` module, so the client refuses
///   to emit a request no server would accept.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis::encode_command;
///
/// let bytes = encode_command(&[&b"GET"[..], &b"k"[..]]).unwrap();
/// assert_eq!(bytes, b"*2\r\n$3\r\nGET\r\n$1\r\nk\r\n".to_vec());
///
/// // A CRLF inside a value stays data: it is length-counted, not delimited.
/// let hostile = encode_command(&[&b"SET"[..], &b"k"[..], &b"a\r\nFLUSHALL"[..]]).unwrap();
/// assert_eq!(hostile, b"*3\r\n$3\r\nSET\r\n$1\r\nk\r\n$11\r\na\r\nFLUSHALL\r\n".to_vec());
/// ```
pub fn encode_command(args: &[&[u8]]) -> Result<Vec<u8>, RedisError> {
    if args.is_empty() {
        return Err(RedisError::Protocol(
            "encode: a command needs at least a name".into(),
        ));
    }
    if args.len() > MAX_ARRAY_LEN {
        return Err(RedisError::Protocol(format!(
            "encode: {} arguments exceeds the {MAX_ARRAY_LEN} limit",
            args.len()
        )));
    }
    let mut out = Vec::new();
    push_header(&mut out, args.len());
    for arg in args {
        if arg.len() > MAX_BULK_LEN {
            return Err(RedisError::Protocol(format!(
                "encode: argument of {} bytes exceeds the {MAX_BULK_LEN} limit",
                arg.len()
            )));
        }
        push_bulk(&mut out, arg);
    }
    Ok(out)
}
