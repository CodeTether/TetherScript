//! `cors_policy(config)` — validate a config map once, at startup.
//!
//! # Security
//!
//! This is where the credential-leak is stopped. `Access-Control-Allow-Origin: *`
//! together with `Access-Control-Allow-Credentials: true` is forbidden by the
//! Fetch spec, and a server that emits the pair invites every site on the
//! internet to read its authenticated responses. The check lives here — at
//! construction — rather than on the request path, so the mistake surfaces once
//! when the policy is built instead of being re-decided per request, where the
//! only branch that catches it might be the branch nobody exercised.

use std::collections::HashMap;

use super::cors_args::map_arg;
use super::cors_config_build::{Fields, policy};
use super::cors_fields as key;
use super::{cors_config_lists as lists, cors_config_scalars as scalars, cors_token};
use crate::value::Value;

/// The message for the one forbidden field combination.
const CONFLICT: &str = "cors_policy: `origins` \"*\" conflicts with `credentials` true: the \
     Fetch spec forbids Access-Control-Allow-Origin: * together with \
     Access-Control-Allow-Credentials: true, because it would let any origin read \
     authenticated responses; list the exact origins instead";

/// Validate a config map and produce the policy map a script holds onto.
///
/// # Arguments
///
/// * `value` — The config map: `origins`, `methods`, `headers`, `expose`,
///   `credentials`, `max_age`.
///
/// # Returns
///
/// A policy map carrying the normalized fields plus a `wildcard` bool.
///
/// # Errors
///
/// Returns an error when a key is unknown, a field has the wrong type, an origin
/// is malformed, or `origins` is `"*"` while `credentials` is true.
pub(super) fn build(value: &Value) -> Result<Value, String> {
    let config = map_arg(value, "cors_policy: config")?;
    reject_unknown(&config)?;
    let (wildcard, origins) = super::cors_config_origins::parse(config.get(key::ORIGINS))?;
    let credentials = scalars::credentials(&config)?;
    if wildcard && credentials {
        return Err(CONFLICT.to_string());
    }
    Ok(policy(Fields {
        wildcard,
        origins,
        methods: lists::methods(&config)?,
        headers: lists::tokens(&config, key::HEADERS, cors_token::header)?,
        expose: lists::tokens(&config, key::EXPOSE, cors_token::header)?,
        credentials,
        max_age: scalars::max_age(&config)?,
    }))
}

/// Reject any key `cors_policy` does not understand.
fn reject_unknown(config: &HashMap<String, Value>) -> Result<(), String> {
    for name in config.keys() {
        if !key::CONFIG_KEYS.contains(&name.as_str()) {
            return Err(format!(
                "cors_policy: unknown config key `{name}`; expected one of {}",
                key::CONFIG_KEYS.join(", ")
            ));
        }
    }
    Ok(())
}
