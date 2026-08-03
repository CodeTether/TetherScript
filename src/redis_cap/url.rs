//! URL parsing for the `--grant-redis` flag.
//!
//! One concern: turning `redis://user:password@host:port/db` into a [`Config`]. Port
//! handling lives in [`super::url_port`], the database index in [`super::url_db`], and
//! credentials in [`super::url_credentials`].
//!
//! # `rediss://` is refused, never downgraded
//!
//! The in-tree client speaks RESP over a plain [`TcpStream`](std::net::TcpStream);
//! there is no TLS transport for it yet, and [`Config`] has no `tls` field to set. So
//! `rediss://` is rejected with a message saying exactly that. Connecting in cleartext
//! would be the worst available option: the caller wrote `rediss://` *because* the URL
//! carries a password, and they would have no way to notice it crossed the network in
//! the clear. This is the judgement `--grant-db` already makes when it refuses
//! `sslmode=prefer`.
//!
//! # The URL must never appear in an error message
//!
//! `redis://app:s3cr3t@cache/0` contains a password, and these errors reach stderr,
//! logs, and CI output. Every message below names only the component that was wrong —
//! the required scheme, the port text, the database text — and never interpolates the
//! URL or the credentials. [`Config`] is likewise not `Debug`, so a panic cannot print
//! the password either, which is why the tests assert on `Err` rather than
//! `unwrap_err`-ing a `Config`.
//!
//! Accepted forms: `redis://host`, `redis://host:port`, `redis://host/db`,
//! `redis://host:port/db`, `redis://:password@host`, `redis://user@host`,
//! `redis://user:password@host:port/db`.

use crate::redis::Config;

/// Parse a `--grant-redis` URL into connection settings.
///
/// # Arguments
///
/// * `url` — A `redis://` URL. The `user:password@`, `:port`, and `/db` parts are all
///   optional.
///
/// # Returns
///
/// A [`Config`] carrying the client's default timeouts. An absent password stays `None`
/// so no `AUTH` is sent, which is correct for a server without `requirepass`.
///
/// # Errors
///
/// Returns an error naming the offending component for: a missing or unknown scheme,
/// `rediss://` (TLS is not wired), an empty host, a non-numeric or out-of-range port,
/// and a non-numeric database index. No message contains the URL or the password.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis_cap::url;
///
/// let config = url::parse("redis://app:pw@cache.internal:6380/3").expect("parse");
/// assert_eq!(config.host, "cache.internal");
/// assert_eq!(config.port, 6380);
/// assert_eq!(config.database, 3);
///
/// // Bare host: default port, database 0, no AUTH.
/// let plain = url::parse("redis://localhost").expect("parse");
/// assert_eq!(plain.port, 6379);
/// assert!(plain.password.is_none());
///
/// // TLS is refused rather than silently downgraded.
/// assert!(url::parse("rediss://cache/0").is_err());
/// ```
pub fn parse(url: &str) -> Result<Config, String> {
    let rest = scheme(url)?;
    let (credentials, location) = match rest.rsplit_once('@') {
        Some((credentials, location)) => (Some(credentials), location),
        None => (None, rest),
    };
    let (authority, path) = location.split_once('/').unwrap_or((location, ""));
    let (host, port) = super::url_port::split(authority)?;
    let (username, password) = super::url_credentials::split(credentials);
    Ok(Config {
        host: host.to_string(),
        port,
        username,
        password,
        database: super::url_db::index(path)?,
        ..Config::default()
    })
}

/// Strip the scheme, refusing `rediss://` explicitly.
///
/// # Errors
///
/// Returns a named error for `rediss://` explaining that TLS is not wired, and for any
/// other scheme naming the one required. Neither message echoes the URL.
fn scheme(url: &str) -> Result<&str, String> {
    if url.starts_with("rediss://") {
        return Err("--grant-redis: rediss:// is not supported because TLS is not wired \
                    for the Redis transport yet; refusing rather than sending your \
                    password in cleartext. Use a TLS tunnel and redis://."
            .into());
    }
    url.strip_prefix("redis://")
        .ok_or_else(|| "--grant-redis must start with redis://".to_string())
}
