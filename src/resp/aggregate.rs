//! # Counted collections: `*`, `~`, `>` and `%`
//!
//! Aggregates announce an element count and then simply concatenate that many
//! encoded values, which is why they nest and why nesting is where a decoder can
//! be attacked. This module walks the elements; the count and depth bounds that
//! gate it live in [`super::aggregate_header`].
//!
//! `-1` is the RESP2 null array and decodes to [`Reply::Nil`], distinct from a
//! zero-element array — an empty `LRANGE` result and a missing key are not the
//! same answer.

use super::aggregate_header::count;
use super::cursor::Cursor;
use super::error::DecodeError;
use super::parse;
use super::reply::Reply;

/// Decode an `*` array, `~` set or `>` push, header line not yet read.
///
/// # Arguments
///
/// * `what` — the RESP type name, for error messages.
/// * `wrap` — builds the matching [`Reply`] variant from the decoded elements.
/// * `depth` — aggregates already entered, including this one.
///
/// # Errors
///
/// [`DecodeError::Incomplete`] while elements are still outstanding;
/// [`DecodeError::Malformed`] for a bound violation in the header or a malformed
/// element.
pub(super) fn sequence(
    cursor: &mut Cursor<'_>,
    what: &str,
    wrap: fn(Vec<Reply>) -> Reply,
    depth: usize,
) -> Result<Reply, DecodeError> {
    let Some(len) = count(cursor, what, depth)? else {
        return Ok(Reply::Nil);
    };
    let mut items = Vec::new();
    for _ in 0..len {
        items.push(parse::value(cursor, depth)?);
    }
    Ok(wrap(items))
}

/// Decode a `%` map, header line not yet read.
///
/// The header counts **pairs**, so `2 * count` values follow. Pairs stay in wire
/// order; see [`Reply::Map`] for why this is not a `HashMap`.
///
/// # Errors
///
/// As [`sequence`].
pub(super) fn map(cursor: &mut Cursor<'_>, depth: usize) -> Result<Reply, DecodeError> {
    let Some(len) = count(cursor, "map", depth)? else {
        return Ok(Reply::Nil);
    };
    let mut pairs = Vec::new();
    for _ in 0..len {
        let key = parse::value(cursor, depth)?;
        pairs.push((key, parse::value(cursor, depth)?));
    }
    Ok(Reply::Map(pairs))
}
