//! Assembly of the token-exchange request body.
//!
//! # This is where the client secret belongs
//!
//! Unlike the authorization URL — a browser `GET` that is logged everywhere, see
//! [`super`] — the token request is a server-to-server `POST` over TLS. Its parameters
//! are in the body, not the URL, so they are not in browser history, not in a `Referer`
//! header, and not in a default access log. That is the only place a `client_secret` may
//! appear.
//!
//! `client_secret` is **optional** here: a public client using PKCE has no secret, and
//! PKCE is what protects it. A confidential client supplies one and gets both.
//!
//! `code_verifier` is required, not optional, because without it the exchange is not
//! PKCE-protected at all. It is validated against the RFC 7636 length rules before it is
//! sent, so a truncated or empty verifier fails locally with a clear message rather than
//! as an opaque `invalid_grant` from the provider several hops later.
//!
//! # Examples
//!
//! ```tether
//! let body = oauth_token_request_body(config, params.code, verifier)?
//! let response = http_post(token_url, body)?
//! ```

use std::collections::HashMap;

use super::super::pkce::{MAX_VERIFIER, MIN_VERIFIER};
use super::config::{opt_str, req_str};
use super::query::pair;
use super::url::redirect;
use crate::value::Value;

/// Built-in name used in every error message from this file.
const LABEL: &str = "oauth_token_request_body";

/// Build the `application/x-www-form-urlencoded` token-exchange body.
///
/// # Arguments
///
/// * `config` — Map with `client_id` and `redirect_uri`, optionally `client_secret`.
/// * `code` — The authorization code from the callback.
/// * `verifier` — The PKCE code verifier whose challenge was sent earlier.
///
/// # Returns
///
/// `grant_type=authorization_code`, then `code`, `redirect_uri`, `client_id`,
/// `code_verifier`, and `client_secret` when configured, in a fixed order so output is
/// deterministic.
///
/// # Errors
///
/// Returns `Err` when `code` is empty, when `verifier` is outside the 43-128 character
/// range, or when a required config field is missing or the wrong type.
pub(crate) fn build(
    config: &HashMap<String, Value>,
    code: &str,
    verifier: &str,
) -> Result<String, String> {
    if code.is_empty() {
        return Err(format!("{LABEL}: code must not be empty"));
    }
    let len = verifier.len();
    if !(MIN_VERIFIER..=MAX_VERIFIER).contains(&len) {
        return Err(format!(
            "{LABEL}: code_verifier must be {MIN_VERIFIER}-{MAX_VERIFIER} characters, got {len}"
        ));
    }
    let mut parts = vec![
        pair("grant_type", "authorization_code"),
        pair("code", code),
        pair("redirect_uri", &redirect(config, LABEL)?),
        pair("client_id", &req_str(config, "client_id", LABEL)?),
        pair("code_verifier", verifier),
    ];
    if let Some(secret) = opt_str(config, "client_secret", LABEL)? {
        parts.push(pair("client_secret", &secret));
    }
    Ok(parts.join("&"))
}
