//! Host and port splitting for a `--grant-redis` URL.
//!
//! One concern: the authority component. Separate from [`super::url`] so the default
//! port lives next to the parsing that applies it, mirroring
//! `main_caps::db_port`.
//!
//! IPv6 literals in brackets are **not** supported. Accepting `[::1]:6379` needs a
//! bracket-aware split, and `rsplit_once(':')` would otherwise read `:1]` as the
//! port. Rejecting is better than misparsing an address; pass a hostname instead.

/// Redis' registered default port, used when the URL omits one.
pub const DEFAULT_PORT: u16 = 6379;

/// Split `host` or `host:port`, defaulting the port.
///
/// # Arguments
///
/// * `authority` — The portion after any `user:password@` and before any `/db` path.
///
/// # Returns
///
/// The host and the resolved port, [`DEFAULT_PORT`] when absent.
///
/// # Errors
///
/// Returns an error naming the offending text when the port is not a number or is out
/// of `u16` range, and an error naming the missing host when it is empty, so a
/// truncated URL fails loudly rather than connecting somewhere unintended. The
/// authority never contains the password, so neither message can leak one.
///
/// # Examples
///
/// ```rust
/// use tetherscript::redis_cap::url_port;
///
/// assert_eq!(url_port::split("localhost:6380").unwrap(), ("localhost", 6380));
/// assert_eq!(url_port::split("cache.internal").unwrap(), ("cache.internal", 6379));
/// assert!(url_port::split(":6379").is_err());
/// assert!(url_port::split("localhost:nope").is_err());
/// ```
pub fn split(authority: &str) -> Result<(&str, u16), String> {
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => {
            let parsed = port
                .parse::<u16>()
                .map_err(|_| format!("--grant-redis port must be a number (got `{port}`)"))?;
            (host, parsed)
        }
        None => (authority, DEFAULT_PORT),
    };
    if host.is_empty() {
        return Err("--grant-redis needs a host".to_string());
    }
    Ok((host, port))
}
