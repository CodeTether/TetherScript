//! Argument assembly for `SET`.
//!
//! Separate from the command method so the method stays one screen long, and so the
//! modifier order is stated in one place.
//!
//! Order follows the documented grammar — `SET key value [NX|XX] [EX seconds]` —
//! because the server's parser is positional about the modifier block.

use super::options::SetOptions;

/// Build the argument list for `SET`.
///
/// # Arguments
///
/// * `key` — Key bytes.
/// * `value` — Value bytes; binary-safe, CRLF included, because each argument is
///   length-prefixed by `encode_command`.
/// * `options` — `NX` and `EX` modifiers.
/// * `seconds` — The already-rendered `EX` argument, or `None`. The caller owns the
///   string so the returned slices borrow from it rather than from a temporary.
///
/// # Returns
///
/// Arguments beginning with `SET`, ready for `encode_command`.
pub(super) fn build<'a>(
    key: &'a [u8],
    value: &'a [u8],
    options: &SetOptions,
    seconds: Option<&'a str>,
) -> Vec<&'a [u8]> {
    let mut args: Vec<&[u8]> = vec![&b"SET"[..], key, value];
    if options.if_not_exists {
        args.push(&b"NX"[..]);
    }
    if let Some(ttl) = seconds {
        args.push(&b"EX"[..]);
        args.push(ttl.as_bytes());
    }
    args
}
