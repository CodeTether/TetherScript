//! Serializing a payload map to the compact wire string.
//!
//! # Format
//!
//! ```text
//! name=sAda\sLovelace;uid=i42;admin=btrue
//! ```
//!
//! Entries are `escape(key)=escape(tag(value))`, joined by `;`. Both halves are
//! escaped, so neither a key nor a value can introduce a separator; see
//! [`super::sessionstore_escape`] for the escape table and why a naive split loses
//! data.
//!
//! # Deterministic order
//!
//! Keys are sorted. A tetherscript map is a `HashMap`, whose iteration order varies
//! between runs, so an unsorted encoding of the same payload would differ run to
//! run. That would make the output unusable as an ETag or a cache key and would make
//! any byte-comparison test flaky. Sorting costs one `sort` per save.
//!
//! An empty map encodes to the empty string, and the decoder maps the empty string
//! back to an empty map — that pair is what makes the round-trip total.

use std::collections::HashMap;

use super::sessionstore_escape::{escape, ENTRY_SEP, PAIR_SEP};
use super::sessionstore_tag::tagged;
use crate::value::Value;

/// Serialize a payload map.
///
/// # Arguments
///
/// * `label` — Built-in and parameter name, used verbatim in errors.
/// * `payload` — The map to serialize.
///
/// # Returns
///
/// A compact single-line string, empty when `payload` is empty.
///
/// # Errors
///
/// Returns a named error when a key is empty, or when a value is not one of str,
/// int, float, bool, or nil.
///
/// # Examples
///
/// ```rust,ignore
/// let mut map = std::collections::HashMap::new();
/// map.insert("uid".to_string(), crate::value::Value::Int(42));
/// assert_eq!(encode("l", &map).unwrap(), "uid=i42");
/// ```
pub(super) fn encode(label: &str, payload: &HashMap<String, Value>) -> Result<String, String> {
    let mut keys: Vec<&String> = payload.keys().collect();
    keys.sort();
    let mut entries: Vec<String> = Vec::with_capacity(keys.len());
    for key in keys {
        if key.is_empty() {
            return Err(format!("{label}: payload keys must not be empty"));
        }
        let value = tagged(label, key, &payload[key])?;
        entries.push(format!("{}{PAIR_SEP}{}", escape(key), escape(&value)));
    }
    // `join` wants a `&str`, and the separator is declared as a `char` so the escape
    // table can match on it; one small allocation per save is the honest cost.
    Ok(entries.join(&ENTRY_SEP.to_string()))
}
