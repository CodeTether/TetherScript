//! Request-map access and case-insensitive header lookup.
//!
//! The map shape is the one `src/http_server_request_map.rs` builds: `method`,
//! `path`, `query`, `headers`, `body`. Only `method` and `headers` matter to CORS.
//!
//! Header lookup is case-insensitive, matching `header_lookup`: the native parser
//! lower-cases names, but a script may build a header map by hand from a fixture
//! or an upstream response, so nothing here assumes normalization. Header *values*
//! are not folded — see `cors_origin` for why the origin is compared exactly.

use std::collections::HashMap;

use super::cors_args::{map_arg, str_arg};
use super::cors_fields as key;
use super::cors_token;
use crate::value::Value;

/// A CORS-relevant view of a request.
pub(super) struct Request {
    /// The request method, upper-cased.
    pub(super) method: String,
    /// The `Origin` header, when present.
    pub(super) origin: Option<String>,
    /// `Access-Control-Request-Method`, when present.
    pub(super) want_method: Option<String>,
    /// `Access-Control-Request-Headers`, when present.
    pub(super) want_headers: Option<String>,
}

/// Read a request map.
///
/// # Arguments
///
/// * `value` — The request map a handler received.
/// * `label` — Built-in name used in error messages.
///
/// # Returns
///
/// The [`Request`] the CORS decisions read. A missing `method` or `headers` is
/// tolerated so a script can pass a minimal fixture; both simply mean "not a
/// preflight, and no origin to echo".
///
/// # Errors
///
/// Returns an error when `value` is not a map, `headers` is present and not a
/// map, or `method` is present and not a str.
pub(super) fn read(value: &Value, label: &str) -> Result<Request, String> {
    let request = map_arg(value, &format!("{label}: request"))?;
    let headers = match request.get("headers") {
        None | Some(Value::Nil) => HashMap::new(),
        Some(map) => map_arg(map, &format!("{label}: request headers"))?,
    };
    Ok(Request {
        method: method_of(&request, label)?,
        origin: find(&headers, key::ORIGIN),
        want_method: find(&headers, key::REQUEST_METHOD),
        want_headers: find(&headers, key::REQUEST_HEADERS),
    })
}

/// Read and normalize the request method.
fn method_of(request: &HashMap<String, Value>, label: &str) -> Result<String, String> {
    match request.get("method") {
        None | Some(Value::Nil) => Ok(String::new()),
        Some(value) => {
            let raw = str_arg(value, &format!("{label}: request `method`"))?;
            Ok(cors_token::method(&raw))
        }
    }
}

/// Find a header value without regard to the stored name's case.
fn find(headers: &HashMap<String, Value>, name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(stored, _)| stored.eq_ignore_ascii_case(name))
        .and_then(|(_, value)| match value {
            Value::Str(text) => Some(text.trim().to_string()),
            _ => None,
        })
}
