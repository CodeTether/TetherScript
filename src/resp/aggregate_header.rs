//! # Validating an aggregate count header
//!
//! Split out from [`super::aggregate`] because the two do different jobs: this
//! module decides whether a peer's announced count and the current nesting depth
//! are acceptable *before* any element is read, while the sibling walks the
//! elements. Keeping the bound checks in one place means there is exactly one
//! spot to audit when asking "can a header make us allocate?".
//!
//! The answer is no: the count is validated but never used to reserve capacity,
//! so a header claiming a million elements costs nothing until a million elements
//! actually arrive.

use super::cursor::Cursor;
use super::error::DecodeError;
use super::limits::{MAX_AGGREGATE_LEN, MAX_DEPTH};
use super::scalar;

/// Read and validate the count line of an `*`, `~`, `>` or `%` header.
///
/// # Arguments
///
/// * `cursor` — positioned just after the type byte.
/// * `what` — the RESP type name, used in error messages.
/// * `depth` — aggregates entered so far, including this one.
///
/// # Returns
///
/// `Ok(None)` for the null count `-1` (the RESP2 null array), otherwise the
/// validated count. For a map the count is a number of *pairs*.
///
/// # Errors
///
/// [`DecodeError::Incomplete`] when the count line has not arrived;
/// [`DecodeError::Malformed`] for a depth past [`MAX_DEPTH`], a non-integer
/// count, a count below `-1`, or a count above [`MAX_AGGREGATE_LEN`]. The depth
/// check runs first so that a deeply nested hostile reply is rejected without
/// even parsing its innermost header.
pub(super) fn count(
    cursor: &mut Cursor<'_>,
    what: &str,
    depth: usize,
) -> Result<Option<i64>, DecodeError> {
    if depth > MAX_DEPTH {
        return Err(DecodeError::malformed(format!(
            "{what} nests deeper than the {MAX_DEPTH}-level limit"
        )));
    }
    let count = scalar::integer(cursor.line()?, what)?;
    if count == -1 {
        return Ok(None);
    }
    if count < -1 {
        return Err(DecodeError::malformed(format!(
            "{what} has negative length {count}"
        )));
    }
    if count > MAX_AGGREGATE_LEN {
        return Err(DecodeError::malformed(format!(
            "{what} length {count} exceeds the {MAX_AGGREGATE_LEN}-element limit"
        )));
    }
    Ok(Some(count))
}
