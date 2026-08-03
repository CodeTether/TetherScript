//! Reading the token-list fields of a CORS config.
//!
//! Every field except `origins` has a default, and each default is chosen to fail
//! closed: an omitted `methods` allows only the two read-only methods, an omitted
//! `headers` allows none, and an omitted `credentials` is false. A caller who
//! needs more must say so, so widening the policy is always visible in the config.

use std::collections::HashMap;

use super::cors_args::string_list;
use super::cors_fields as key;
use super::cors_token;
use crate::value::Value;

/// The default `methods`: read-only, so an omitted field cannot enable writes.
pub(super) const DEFAULT_METHODS: [&str; 2] = ["GET", "HEAD"];

/// Read a token list field, normalizing each entry with `normalize`.
///
/// # Arguments
///
/// * `config` — The config map.
/// * `name` — Field name.
/// * `normalize` — `cors_token::method` or `cors_token::header`.
///
/// # Returns
///
/// The normalized tokens, or an empty vec when the field is absent. An
/// all-whitespace entry is dropped rather than becoming an empty token that could
/// never match anything on the wire.
///
/// # Errors
///
/// Returns an error naming the field when it is not a list of strs.
pub(super) fn tokens(
    config: &HashMap<String, Value>,
    name: &str,
    normalize: fn(&str) -> String,
) -> Result<Vec<String>, String> {
    let listed = match config.get(name) {
        None | Some(Value::Nil) => return Ok(Vec::new()),
        Some(value) => string_list(value, &format!("cors_policy: `{name}`"))?,
    };
    Ok(listed
        .iter()
        .map(|token| normalize(token.as_str()))
        .filter(|token| !token.is_empty())
        .collect())
}

/// Read the `methods` field, applying [`DEFAULT_METHODS`] when absent.
///
/// # Returns
///
/// The upper-cased methods the policy allows.
///
/// # Errors
///
/// Returns an error when the field is present but not a list of strs.
pub(super) fn methods(config: &HashMap<String, Value>) -> Result<Vec<String>, String> {
    let listed = tokens(config, key::METHODS, cors_token::method)?;
    if !listed.is_empty() {
        return Ok(listed);
    }
    Ok(DEFAULT_METHODS.iter().map(|m| m.to_string()).collect())
}
