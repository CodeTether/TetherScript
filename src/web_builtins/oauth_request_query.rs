//! The authorization request's query string, and the shared `name=value` encoder.
//!
//! Split from [`super::url`] so URL joining and parameter assembly are separate
//! concerns. The reasoning for mandatory `state` and `code_challenge` is in [`super`].
//!
//! Parameter order is fixed and deterministic so a test can assert on the whole URL and
//! a log line is diffable. Order is not semantically meaningful to the server.
//!
//! Every value is percent-encoded, so a `scope` such as `openid profile email` becomes
//! `openid%20profile%20email` rather than silently truncating the query at the first
//! space.

use std::collections::HashMap;

use super::super::percent::encode;
use super::super::pkce::METHOD;
use super::config::{opt_str, req_str};
use super::url::redirect;
use crate::value::Value;

/// Built-in name used in every error message from this file.
const LABEL: &str = "oauth_authorize_url";

/// Optional parameters, appended in this order when present.
const OPTIONAL: [&str; 3] = ["nonce", "prompt", "login_hint"];

/// Percent-encode one `name=value` pair.
///
/// # Arguments
///
/// * `name` — Parameter name.
/// * `value` — Raw parameter value.
///
/// # Returns
///
/// `name=value` with both sides percent-encoded.
pub(crate) fn pair(name: &str, value: &str) -> String {
    format!("{}={}", encode(name), encode(value))
}

/// Render the percent-encoded query string for the authorization request.
///
/// # Arguments
///
/// * `config` — The script-supplied config map.
///
/// # Returns
///
/// `response_type=code`, then `client_id`, `redirect_uri`, `scope`, `state`,
/// `code_challenge`, `code_challenge_method=S256`, then any optional fields.
///
/// # Errors
///
/// Returns `Err` when `client_id`, `redirect_uri`, `scope`, `state`, or
/// `code_challenge` is missing, empty, or not a string, or when `redirect_uri` fails
/// its shape check.
pub(crate) fn render(config: &HashMap<String, Value>) -> Result<String, String> {
    let mut parts = vec![
        pair("response_type", "code"),
        pair("client_id", &req_str(config, "client_id", LABEL)?),
        pair("redirect_uri", &redirect(config, LABEL)?),
        pair("scope", &req_str(config, "scope", LABEL)?),
        pair("state", &req_str(config, "state", LABEL)?),
        pair("code_challenge", &req_str(config, "code_challenge", LABEL)?),
        pair("code_challenge_method", METHOD),
    ];
    for name in OPTIONAL {
        if let Some(value) = opt_str(config, name, LABEL)? {
            parts.push(pair(name, &value));
        }
    }
    Ok(parts.join("&"))
}
