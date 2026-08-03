//! Query-string parsing for the OAuth callback.
//!
//! Split from [`super`], which decides what a callback *means*; this file only turns a
//! query string into fields. The success/error discrimination rationale is documented
//! there.
//!
//! A leading `?` is tolerated because `http_serve` hands handlers a bare query string
//! while a script may pass a URL's tail by hand, and silently mis-parsing the first
//! parameter name as `?code` would be a maddening bug.
//!
//! Only the four OAuth-defined fields are surfaced, and only the **first** occurrence of
//! each is kept. A provider appending unrelated parameters cannot shadow `code`, and a
//! crafted `?code=good&code=evil` cannot override the value the handler validated.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::super::percent::decode::decode;
use super::{Outcome, outcome};
use crate::value::Value;

/// Fields lifted out of the callback query.
const FIELDS: [&str; 4] = ["code", "state", "error", "error_description"];

/// Parse a callback query string into a `Result` of a four-field map.
///
/// # Arguments
///
/// * `query` — The raw query string, with or without a leading `?`.
///
/// # Returns
///
/// `Ok` of a map with `code`, `state`, `error`, and `error_description`. Absent fields are
/// `nil` rather than missing keys, so a script may read any of them without a containment
/// check.
///
/// # Errors
///
/// Returns `Err` when a value has a malformed percent escape, and — the important case —
/// when the callback reports an `error`, so an error callback can never be mistaken for a
/// success by a caller that only looks at `code`.
pub(crate) fn params(query: &str) -> Result<Value, String> {
    let found = collect(query.strip_prefix('?').unwrap_or(query))?;
    let field = |name: &str| found.get(name).map(String::as_str);
    if let Outcome::Failure(message) =
        outcome(field("code"), field("error"), field("error_description"))
    {
        return Err(message);
    }
    let mut map = HashMap::with_capacity(FIELDS.len());
    for name in FIELDS {
        let value = match found.get(name) {
            Some(text) => Value::Str(Rc::new(text.clone())),
            None => Value::Nil,
        };
        map.insert(name.to_string(), value);
    }
    Ok(Value::Map(Rc::new(RefCell::new(map))))
}

/// Percent-decode the first occurrence of each OAuth field.
fn collect(query: &str) -> Result<HashMap<String, String>, String> {
    let mut found: HashMap<String, String> = HashMap::new();
    for segment in query.split('&').filter(|part| !part.is_empty()) {
        let (raw_name, raw_value) = segment.split_once('=').unwrap_or((segment, ""));
        let name = decode(raw_name, "oauth_callback_params: parameter name")?;
        if !FIELDS.contains(&name.as_str()) || found.contains_key(&name) {
            continue;
        }
        let label = format!("oauth_callback_params: `{name}`");
        let value = decode(raw_value, &label)?;
        if !value.is_empty() {
            found.insert(name, value);
        }
    }
    Ok(found)
}
