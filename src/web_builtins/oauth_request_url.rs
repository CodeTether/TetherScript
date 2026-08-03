//! Assembly of the authorization request URL.
//!
//! The security reasoning — exact `redirect_uri` and no client secret in the URL — is
//! in [`super`]. The query parameters themselves are built by [`super::query`]; this
//! file only performs the config guard and the base-URL join.

use std::collections::HashMap;

use super::config::{opt_str, req_str};
use super::{query, uri};
use crate::value::Value;

/// Built-in name used in every error message from this file.
const LABEL: &str = "oauth_authorize_url";

/// Build the full authorization URL from a config map.
///
/// # Arguments
///
/// * `config` — Map with `authorize_url`, `client_id`, `redirect_uri`, `scope`,
///   `state`, and `code_challenge`. Optional `nonce`, `prompt`, and `login_hint` are
///   appended when present.
///
/// # Returns
///
/// The complete URL. `?` or `&` is chosen according to whether `authorize_url` already
/// carries a query, so a discovery document that hands back a parameterised endpoint
/// still yields a valid URL.
///
/// # Errors
///
/// Returns `Err` when a required field is missing, empty, or the wrong type; when
/// `redirect_uri` is not an exactly-comparable absolute URL; or when the config
/// contains a `client_secret`, which must never travel in this URL.
pub(crate) fn build(config: &HashMap<String, Value>) -> Result<String, String> {
    if opt_str(config, "client_secret", LABEL)?.is_some() {
        return Err(format!(
            "{LABEL}: config must not contain `client_secret`; the authorization URL is a browser GET that lands in history, Referer headers, and access logs. Pass the secret to oauth_token_request_body instead."
        ));
    }
    let base = req_str(config, "authorize_url", LABEL)?;
    let joiner = if base.contains('?') { '&' } else { '?' };
    Ok(format!("{base}{joiner}{}", query::render(config)?))
}

/// Read and validate the `redirect_uri` field.
///
/// Shared by [`build`] and [`super::body::build`] so both requests carry the identical
/// value; see [`super`] for why that identity matters.
///
/// # Arguments
///
/// * `config` — The script-supplied config map.
/// * `label` — Built-in name, used verbatim in the error message.
///
/// # Returns
///
/// The validated redirect URI, unchanged.
///
/// # Errors
///
/// Returns `Err` when the field is missing or fails [`uri::validate`].
pub(crate) fn redirect(config: &HashMap<String, Value>, label: &str) -> Result<String, String> {
    uri::validate(&req_str(config, "redirect_uri", label)?, label)
}
