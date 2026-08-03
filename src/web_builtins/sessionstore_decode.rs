//! Parsing the wire string back into a payload map.
//!
//! Splitting is safe *only because* both halves of every entry were escaped on the
//! way out, so no `;` or `=` survives inside a component. See
//! [`super::sessionstore_escape`].
//!
//! A duplicate key is an error rather than a last-writer-wins overwrite: the
//! encoder cannot produce one, so a duplicate means the text was assembled by
//! something else, and silently keeping one of two conflicting values for a field
//! such as `admin` is exactly the kind of ambiguity a store must not resolve by
//! guessing.

use std::collections::HashMap;

use super::sessionstore_escape::{ENTRY_SEP, PAIR_SEP};
use super::sessionstore_unescape::unescape;
use super::sessionstore_untag::parse;
use crate::value::Value;

/// Deserialize a payload map.
///
/// # Arguments
///
/// * `label` — Built-in and parameter name, used verbatim in errors.
/// * `text` — Output of [`super::sessionstore_encode::encode`].
///
/// # Returns
///
/// The reconstructed map; empty when `text` is empty.
///
/// # Errors
///
/// Returns a named error when an entry has no `=`, has an empty key, repeats a key,
/// carries a malformed escape, or carries an unknown type tag.
///
/// # Examples
///
/// ```rust,ignore
/// let map = decode("l", "uid=i42").unwrap();
/// assert!(matches!(map["uid"], crate::value::Value::Int(42)));
/// ```
pub(super) fn decode(label: &str, text: &str) -> Result<HashMap<String, Value>, String> {
    let mut out = HashMap::new();
    if text.is_empty() {
        return Ok(out);
    }
    for entry in text.split(ENTRY_SEP) {
        let (raw_key, raw_value) = entry.split_once(PAIR_SEP).ok_or_else(|| {
            format!("{label}: entry {entry:?} is missing the `{PAIR_SEP}` separator")
        })?;
        let key = unescape(label, raw_key)?;
        if key.is_empty() {
            return Err(format!("{label}: entry {entry:?} has an empty key"));
        }
        let body = unescape(label, raw_value)?;
        if out
            .insert(key.clone(), parse(label, &key, &body)?)
            .is_some()
        {
            return Err(format!("{label}: duplicate key {key:?}"));
        }
    }
    Ok(out)
}
