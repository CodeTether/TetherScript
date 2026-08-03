//! Discrimination of a success callback from an error callback.
//!
//! # Why reading only `code` is a bug
//!
//! RFC 6749 §4.1.2.1 says a failed authorization redirects back with `error` and
//! optionally `error_description`, and with **no** `code`. A callback handler that reaches
//! straight for `code` therefore sees `nil` and has no idea why. What it does next is
//! usually one of:
//!
//! * proceed with an empty code, `POST` it to the token endpoint, and surface the
//!   provider's `invalid_grant` — a message about the wrong thing, several network hops
//!   from the real cause; or
//! * report "login failed" with no detail, discarding an `access_denied` or
//!   `consent_required` the provider had spelled out precisely.
//!
//! Both turn a clean, actionable failure into a confusing one much later. So the map
//! returned by [`parse::params`] always carries all four fields, and [`outcome`] states
//! which kind of callback it is rather than leaving the caller to infer it from a missing
//! value.
//!
//! # Both `code` and `error` present
//!
//! Not a legal callback: a conforming server sends one or the other. It is treated as an
//! error, because the failure signal is the one that must not be lost, and because a
//! request carrying both is a sign of a tampered or stitched-together redirect.
//!
//! # Neither present
//!
//! Also an error. An empty callback is not a successful login, and reporting it as
//! "neither code nor error" names the actual problem.
//!
//! # Examples
//!
//! ```tether
//! let params = oauth_callback_params(req.query)?   // Err on an error callback
//! let path = oauth_state_verify(secret, params.state)?
//! ```

#[path = "oauth_callback_parse.rs"]
pub(crate) mod parse;

/// Which kind of callback a query string represents.
///
/// # Examples
///
/// ```rust,ignore
/// match outcome(Some("abc"), None, None) {
///     Outcome::Success => println!("proceed to the token exchange"),
///     Outcome::Failure(message) => println!("{message}"),
/// }
/// ```
pub(crate) enum Outcome {
    /// `code` is present and no `error` is.
    Success,
    /// The provider reported a failure; the payload is the formatted message.
    Failure(String),
}

/// Classify a callback from its `code` and `error` fields.
///
/// # Arguments
///
/// * `code` — The `code` parameter, if present and non-empty.
/// * `error` — The `error` parameter, if present and non-empty.
/// * `description` — The `error_description` parameter, if present and non-empty.
///
/// # Returns
///
/// [`Outcome::Success`] only when a code is present and no error is. Otherwise
/// [`Outcome::Failure`] with a message naming the provider's error code and its
/// description, so nothing the provider said is discarded.
pub(crate) fn outcome(
    code: Option<&str>,
    error: Option<&str>,
    description: Option<&str>,
) -> Outcome {
    match (code, error) {
        (Some(_), None) => Outcome::Success,
        (_, Some(error)) => Outcome::Failure(match description {
            Some(text) => {
                format!("oauth_callback_params: authorization failed with error `{error}`: {text}")
            }
            None => format!("oauth_callback_params: authorization failed with error `{error}`"),
        }),
        (None, None) => Outcome::Failure(
            "oauth_callback_params: callback has neither `code` nor `error`; it is not a successful authorization".into(),
        ),
    }
}
