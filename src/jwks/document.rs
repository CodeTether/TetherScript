//! Parsing a JWKS document body into keys plus skip records.
//!
//! One responsibility: apply the size bound, run the in-tree JSON parser, and
//! partition the `keys` array into usable keys and recorded skips. Top-level shape
//! rules live in `crate::jwks::keys_array`.
//!
//! # Which JSON parser this reuses
//!
//! `crate::json::parse_str` — the in-tree, dependency-free parser that backs the
//! `json_parse` built-in. It returns a [`Value`] and reports failure as a plain
//! `String` ending in `at byte N`; that message is wrapped verbatim in
//! [`JwksError::MalformedJson`] so the byte offset survives to the caller. No
//! second JSON parser is written here.
//!
//! # Why the size check precedes the parse
//!
//! The bound is applied to the *source bytes*, before any allocation, because the
//! document arrives from a remote endpoint. Checking after parsing would mean the
//! parse itself is the unbounded work the bound exists to prevent.

use crate::json::parse_str;
use crate::jwks::error::JwksError;
use crate::jwks::fields::opt_str;
use crate::jwks::keys_array::keys_array;
use crate::jwks::limits::MAX_DOCUMENT_BYTES;
use crate::jwks::parse_key::parse_key;
use crate::jwks::{key::RsaPublicKey, skipped::SkippedKey};
use crate::value::Value;

/// The two outputs of walking a `keys` array: usable keys, and dropped entries.
pub(crate) type Parsed = (Vec<RsaPublicKey>, Vec<SkippedKey>);

/// Parse a JWKS document body.
///
/// # Arguments
///
/// * `body` — The document source, as fetched from the JWKS endpoint.
///
/// # Returns
///
/// The usable keys in document order, and a record for every entry dropped.
///
/// # Errors
///
/// Returns [`JwksError::DocumentTooLarge`] or [`JwksError::MalformedJson`], or any
/// shape error from [`keys_array`]. An individual unusable key is deliberately
/// **not** an error: it is recorded as a skip.
///
/// # Panics
///
/// Does not panic.
pub(crate) fn parse_document(body: &str) -> Result<Parsed, JwksError> {
    if body.len() > MAX_DOCUMENT_BYTES {
        return Err(JwksError::DocumentTooLarge {
            bytes: body.len(),
            limit: MAX_DOCUMENT_BYTES,
        });
    }
    let document = parse_str(body).map_err(JwksError::MalformedJson)?;
    Ok(walk(&keys_array(&document)?))
}

/// Validate each entry, partitioning into keys and skips.
fn walk(entries: &[Value]) -> Parsed {
    let mut keys = Vec::new();
    let mut skipped = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        let label = format!("jwks: keys[{index}]");
        match parse_key(entry, &label) {
            Ok(key) => keys.push(key),
            Err(reason) => skipped.push(SkippedKey {
                index,
                kid: opt_str(entry, "kid", &label).ok().flatten(),
                reason,
            }),
        }
    }
    (keys, skipped)
}
