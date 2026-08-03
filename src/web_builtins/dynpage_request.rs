//! Request-map access and case-insensitive header lookup.
//!
//! The map shape is the one `src/http_server_request_map.rs` builds: `method`,
//! `path`, `query`, `headers`, `body`. Only `headers` matters here.
//!
//! Lookup is case-insensitive because HTTP header names are (RFC 9110 §5.1). The
//! native parser lower-cases every name it stores, but a script may build a
//! header map by hand from a fixture or an upstream response, so nothing here
//! assumes normalisation.
//!
//! `header.rs` has an equivalent helper, but `header_lookup::find` is
//! `pub(super)` inside that group and is therefore unreachable from this one.
//! Rather than widen another author's visibility — which would be an edit to a
//! file this change does not own — the four lines are restated here, with the
//! same case-folding rule so the two cannot disagree.

use std::collections::HashMap;

use super::dynpage_args::map_arg;
use crate::value::Value;

/// Read the `headers` sub-map out of a request map.
///
/// # Arguments
///
/// * `request` — The request map a handler received.
/// * `label` — Built-in name used in error messages.
///
/// # Returns
///
/// The header map, or an empty map when `headers` is absent or nil, so a minimal
/// fixture is usable and simply means "no headers were sent".
///
/// # Errors
///
/// Returns an error when `request` is not a map, or when `headers` is present and
/// not a map.
pub(super) fn headers_of(request: &Value, label: &str) -> Result<HashMap<String, Value>, String> {
    let request = map_arg(request, &format!("{label}: request"))?;
    match request.get("headers") {
        None | Some(Value::Nil) => Ok(HashMap::new()),
        Some(map) => map_arg(map, &format!("{label}: request headers")),
    }
}

/// Find a header value, comparing names without regard to case.
///
/// # Arguments
///
/// * `headers` — Header map.
/// * `name` — Header name in any casing.
///
/// # Returns
///
/// The trimmed value, or `None` when no name matches or the value is not a str.
pub(super) fn find(headers: &HashMap<String, Value>, name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .and_then(|(_, value)| match value {
            Value::Str(text) => Some(text.trim().to_string()),
            _ => None,
        })
}
