//! Chunk-extension handling: split a size line into its size field.
//!
//! A chunk-size line may carry extensions: `1a;name=value;flag\r\n`. RFC 9112 says a
//! recipient MUST ignore unrecognised extensions, and since no extension is registered for
//! general use, this codec ignores *all* of them. Ignoring is not the same as tolerating
//! sloppiness — the split point is the first `;`, and everything before it must still be a
//! strict hex size, so `5 ;a=b` is rejected for the space rather than silently trimmed.
//!
//! Extensions are discarded, not returned. Nothing in the streaming path needs them, and
//! handing attacker-controlled key/value bytes to a caller who will not inspect them is a
//! liability with no upside.
//!
//! # Panics
//!
//! None. `split_at` is only reached with an index returned by `position`, which is always
//! within bounds; the no-`;` path returns the input slice unchanged.

/// Strip any chunk extensions from a size line, yielding the bare size field.
///
/// # Arguments
///
/// * `line` — One chunk-size line with its CRLF already removed.
///
/// # Returns
///
/// The bytes before the first `;`, unmodified and untrimmed. Validation is the size
/// parser's job, so a malformed size survives this step to be rejected with a precise
/// message.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::strip_extensions;
///
/// assert_eq!(strip_extensions(b"1a"), b"1a");
/// assert_eq!(strip_extensions(b"1a;name=value"), b"1a");
/// assert_eq!(strip_extensions(b"0;signature=deadbeef;flag"), b"0");
///
/// // Whitespace is preserved so the size parser can reject it by name.
/// assert_eq!(strip_extensions(b"1a ;x=1"), b"1a ");
/// ```
pub fn strip_extensions(line: &[u8]) -> &[u8] {
    match line.iter().position(|byte| *byte == b';') {
        Some(at) => line.split_at(at).0,
        None => line,
    }
}
