//! Credential splitting for a `--grant-redis` URL.
//!
//! One concern: turning the optional `user:password` component into the two
//! [`Config`](crate::redis::Config) fields. Separate from [`super::url`] so the
//! password passes through the smallest possible amount of code — and note that this
//! module returns values and never formats an error at all, so it has no way to leak
//! one.
//!
//! | URL component | `username` | `password` | `AUTH` sent |
//! |---|---|---|---|
//! | absent | `None` | `None` | none — correct without `requirepass` |
//! | `:pw@` | `None` | `Some("pw")` | legacy one-argument `AUTH pw` |
//! | `user:pw@` | `Some("user")` | `Some("pw")` | ACL two-argument `AUTH user pw` |
//! | `user@` | `Some("user")` | `None` | none; `Config` skips `AUTH` without a password |
//!
//! An empty password (`redis://user:@host`) yields `Some("")` rather than `None`: a
//! server with `requirepass ""` is a different situation from one with no password at
//! all, and guessing would pick the wrong one.

/// Split the credential component into username and password.
///
/// # Arguments
///
/// * `credentials` — The text before `@`, or `None` when the URL had no `@`.
///
/// # Returns
///
/// The username and password, both optional. Percent-decoding is deliberately not
/// performed: silently rewriting a password is worse than requiring a literal one, and
/// a password containing `@` or `:` belongs in a config file the host reads, not on a
/// command line where `ps` shows it.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis_cap::url_credentials;
///
/// assert_eq!(url_credentials::split(None), (None, None));
/// assert_eq!(url_credentials::split(Some(":pw")), (None, Some("pw".to_string())));
/// assert_eq!(
///     url_credentials::split(Some("u:pw")),
///     (Some("u".to_string()), Some("pw".to_string()))
/// );
/// assert_eq!(url_credentials::split(Some("u")), (Some("u".to_string()), None));
/// ```
pub fn split(credentials: Option<&str>) -> (Option<String>, Option<String>) {
    let Some(credentials) = credentials else {
        return (None, None);
    };
    match credentials.split_once(':') {
        Some(("", password)) => (None, Some(password.to_string())),
        Some((user, password)) => (Some(user.to_string()), Some(password.to_string())),
        None if credentials.is_empty() => (None, None),
        None => (Some(credentials.to_string()), None),
    }
}
