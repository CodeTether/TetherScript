//! Raw `Cookie` request-header lookup and splitting.
//!
//! Split from `abtest_cookie` so jar access and header parsing are separate
//! concerns: one knows the request map, the other knows RFC 6265 syntax.
//!
//! Header lookup is case-insensitive, matching `header_lookup`. The native parser
//! lower-cases header names, but a script may build a header map by hand from a
//! fixture or an upstream response, so nothing here assumes normalisation.

use std::collections::HashMap;

use super::abtest_args as args;
use crate::value::Value;

/// Find the raw `Cookie` request header.
///
/// # Arguments
///
/// * `request` — The request map.
/// * `label` — Built-in name used in error messages.
///
/// # Returns
///
/// The header value, or `None` when there is no `headers` map or no `Cookie` in it.
///
/// # Errors
///
/// Returns an error when `headers` is present but not a map.
pub(super) fn header(
    request: &HashMap<String, Value>,
    label: &str,
) -> Result<Option<String>, String> {
    let headers = match request.get("headers") {
        None | Some(Value::Nil) => return Ok(None),
        Some(value) => args::map_arg(value, &format!("{label}: request `headers`"))?,
    };
    Ok(headers
        .iter()
        .find(|(stored, _)| stored.eq_ignore_ascii_case("cookie"))
        .and_then(|(_, value)| match value {
            Value::Str(text) => Some((**text).clone()),
            _ => None,
        }))
}

/// Pull one cookie value out of a raw `Cookie` header.
///
/// # Arguments
///
/// * `raw` — The header value, pairs separated by `;`.
/// * `name` — Cookie name to extract.
///
/// # Returns
///
/// The matching value with surrounding whitespace and one layer of double quotes
/// removed, or `None` when absent or empty.
///
/// Only the first `=` splits a pair, so a value containing `=` survives. Parsing is
/// lenient: a malformed pair is skipped rather than failing the request, because a
/// browser may send cookies this server never set. The last occurrence wins,
/// matching `cookie_parse`.
pub(super) fn split(raw: &str, name: &str) -> Option<String> {
    let mut found = None;
    for pair in raw.split(';') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        if key.trim() != name {
            continue;
        }
        let value = value.trim().trim_matches('"');
        if !value.is_empty() {
            found = Some(value.to_string());
        }
    }
    found
}
