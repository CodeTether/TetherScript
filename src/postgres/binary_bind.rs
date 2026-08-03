//! # Format codes for the `Bind` message
//!
//! This is how a caller *asks* for binary. Everything else in this module is useless
//! without it, because a decoder for binary bytes never sees any unless the `Bind`
//! message requested format code 1.
//!
//! ## The `Bind` message, annotated
//!
//! ```text
//! 'B'
//! int32   length
//! cstr    destination portal name ("" = unnamed)
//! cstr    source prepared statement name ("" = unnamed)
//! int16   parameter format-code count      <-- (a)
//! int16[] parameter format codes
//! int16   parameter count
//! for each parameter:
//!   int32 length (-1 = NULL, no bytes follow)
//!   bytes value
//! int16   result-column format-code count  <-- (b)
//! int16[] result-column format codes
//! ```
//!
//! Both count fields (a) and (b) follow the same three-way convention, which is the
//! part most easily misread:
//!
//! | Count | Meaning |
//! |---|---|
//! | `0` | No codes follow; **everything is text** (format 0) |
//! | `1` | One code follows; it applies to **all** values |
//! | `n` | Exactly `n` codes follow, one per value; `n` must match the value count |
//!
//! So the current `src/postgres/extended.rs`, which writes `0` for both, is asking for
//! text parameters and text rows. Switching to binary means writing a count and the
//! codes — which is what [`format_codes`] produces.
//!
//! A count of `n` that disagrees with the parameter count is a protocol error the
//! server will reject, so [`format_codes`] derives the count from the slice rather than
//! taking it separately.

/// Format code for text format.
pub const FORMAT_TEXT: i16 = 0;

/// Format code for binary format.
pub const FORMAT_BINARY: i16 = 1;

/// Build a format-code array for `Bind`, as big-endian bytes.
///
/// # Arguments
///
/// * `codes` — one code per value, each [`FORMAT_TEXT`] or [`FORMAT_BINARY`]. An empty
///   slice produces the count `0`, meaning "all text".
///
/// # Returns
///
/// The count word followed by the codes, all big-endian, ready to splice into a `Bind`
/// message. When every code is identical and there is more than one, the compact
/// "count 1" form is emitted instead — the server applies a single code to every value,
/// so this is smaller and cannot fall out of step with the value count.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::{FORMAT_BINARY, FORMAT_TEXT, format_codes};
///
/// // No codes: count 0, meaning every value is text.
/// assert_eq!(format_codes(&[]), vec![0, 0]);
///
/// // All binary collapses to the compact form: count 1, then the single code.
/// assert_eq!(
///     format_codes(&[FORMAT_BINARY, FORMAT_BINARY]),
///     vec![0, 1, 0, 1]
/// );
///
/// // A mix must be spelled out: count 2, then one code each.
/// assert_eq!(
///     format_codes(&[FORMAT_BINARY, FORMAT_TEXT]),
///     vec![0, 2, 0, 1, 0, 0]
/// );
/// ```
pub fn format_codes(codes: &[i16]) -> Vec<u8> {
    let uniform = codes.first().is_some_and(|first| {
        // Collapse only when every code agrees, so the compact form is never a lie.
        codes.iter().all(|code| code == first)
    });
    let emit: &[i16] = if uniform { &codes[..1] } else { codes };
    let mut out = Vec::with_capacity(2 + emit.len() * 2);
    out.extend_from_slice(&(emit.len() as i16).to_be_bytes());
    for code in emit {
        out.extend_from_slice(&code.to_be_bytes());
    }
    out
}
