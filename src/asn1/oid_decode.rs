//! Assembling OID subidentifiers into dotted-decimal text.
//!
//! The first subidentifier expands into two arcs; every later one becomes a
//! single arc. Arcs are collected into a `Vec<String>` and joined once, so no
//! partial string is ever built inside the loop.

use super::{error::Error, oid_subid, tag};

/// Render OID content octets as dotted decimal.
///
/// # Arguments
///
/// * `content` — the OID's content octets.
/// * `offset` — absolute offset of the OID's tag.
///
/// # Returns
///
/// The dotted-decimal form, with the first octet expanded into two arcs.
///
/// # Errors
///
/// [`Error::MalformedValue`] for empty content or any subidentifier error.
///
/// # Panics
///
/// Never; the loop advances strictly and each octet is read via `slice::get`
/// inside [`oid_subid::next`].
pub(super) fn to_dotted(content: &[u8], offset: usize) -> Result<String, Error> {
    if content.is_empty() {
        return Err(Error::MalformedValue {
            offset,
            tag: tag::OBJECT_IDENTIFIER,
            reason: "OBJECT IDENTIFIER must have at least one subidentifier",
        });
    }
    let mut arcs: Vec<String> = Vec::new();
    let mut pos = 0usize;
    while pos < content.len() {
        let (value, next) = oid_subid::next(content, pos, offset)?;
        if pos == 0 {
            let (arc1, arc2) = split_root(value);
            arcs.push(arc1.to_string());
            arcs.push(arc2.to_string());
        } else {
            arcs.push(value.to_string());
        }
        pos = next;
    }
    Ok(arcs.join("."))
}

/// Split the packed first subidentifier into its two arcs.
///
/// # Arguments
///
/// * `value` — the first subidentifier, encoding `40 * arc1 + arc2`.
///
/// # Returns
///
/// `(arc1, arc2)`. Arcs 0 and 1 are capped at 40 members, so values of 80 and
/// above all belong to arc 2, where the second arc is unbounded.
fn split_root(value: u64) -> (u64, u64) {
    match value {
        0..=39 => (0, value),
        40..=79 => (1, value - 40),
        _ => (2, value - 80),
    }
}
