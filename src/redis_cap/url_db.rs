//! Logical database index parsing for a `--grant-redis` URL.
//!
//! One concern: the `/db` path component. Separate from [`super::url`] so the scheme,
//! credentials, authority, and database each fail with their own message.
//!
//! # Why an out-of-range index is refused rather than clamped
//!
//! Redis serves 16 databases by default (`databases 16`), numbered `0`–`15`. A client
//! that clamped or defaulted an unparsable index would connect to database `0`, and a
//! session store that believed it was isolated on database `9` would be sharing keys
//! with a render cache. So an unparsable or negative index is refused here, and an
//! index the *server* rejects surfaces from `SELECT` at connect time, before any
//! script runs.

/// Parse the path component into a database index.
///
/// # Arguments
///
/// * `path` — Everything after the first `/`, or `""` when the URL had no path.
///
/// # Returns
///
/// The index, `0` for an absent or empty path: `redis://host` and `redis://host/` both
/// mean database `0`, which is what Redis itself defaults to.
///
/// # Errors
///
/// Returns an error naming the offending text when the path is not a non-negative
/// integer — a negative number, a name such as `sessions`, or a trailing segment such
/// as `0/extra`. The path never contains the password, so this cannot leak one.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis_cap::url_db;
///
/// assert_eq!(url_db::index("").unwrap(), 0);
/// assert_eq!(url_db::index("9").unwrap(), 9);
/// assert!(url_db::index("sessions").is_err());
/// assert!(url_db::index("-1").is_err());
/// assert!(url_db::index("0/extra").is_err());
/// ```
pub fn index(path: &str) -> Result<u32, String> {
    if path.is_empty() {
        return Ok(0);
    }
    path.parse::<u32>()
        .map_err(|_| format!("--grant-redis database must be a non-negative number (got `{path}`)"))
}
