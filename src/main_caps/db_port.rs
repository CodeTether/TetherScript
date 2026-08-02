//! Host and port splitting for `--grant-db`.
//!
//! Separate from the URL parser so each file owns one concern, and so the default
//! port lives next to the parsing that applies it.

/// PostgreSQL's default port, used when the URL omits one.
const DEFAULT_PORT: u16 = 5432;

/// Split `host:port` into its parts, defaulting the port.
///
/// # Arguments
///
/// * `authority` — The `host` or `host:port` portion of a connection string.
///
/// # Returns
///
/// The host and the resolved port.
///
/// # Errors
///
/// Returns an error when the host is empty or the port is not a number, naming the
/// offending text so a typo is obvious.
pub(super) fn split(authority: &str) -> Result<(&str, u16), String> {
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => {
            let parsed = port
                .parse::<u16>()
                .map_err(|_| format!("--grant-db port must be a number (got `{port}`)"))?;
            (host, parsed)
        }
        None => (authority, DEFAULT_PORT),
    };
    if host.is_empty() {
        return Err("--grant-db needs a host before the port".to_string());
    }
    Ok((host, port))
}
