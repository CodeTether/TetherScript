//! `sslmode` handling for `--grant-db` connection strings.
//!
//! Uses libpq's spelling so an existing connection string works unchanged. Only
//! the modes that map cleanly onto the in-tree TLS transport are accepted; the
//! rest are rejected rather than silently downgraded, because a caller who wrote
//! `sslmode=verify-ca` and got cleartext would have no way to notice.

/// Whether the query string asks for TLS.
///
/// # Arguments
///
/// * `query` — Everything after `?` in the connection string, possibly empty.
///
/// # Returns
///
/// True when TLS must be negotiated before the startup message.
///
/// # Errors
///
/// Returns an error for an unknown or unsupported `sslmode`. `prefer` is rejected
/// on purpose: it silently falls back to cleartext, so a caller could believe the
/// connection was encrypted when it was not.
pub(super) fn wanted(query: &str) -> Result<bool, String> {
    let mut tls = false;
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if key != "sslmode" {
            return Err(format!("--grant-db: unsupported query parameter `{key}`"));
        }
        tls = match value {
            "disable" => false,
            // verify-full is the same handshake here: the in-tree connector always
            // validates the chain and the hostname, with no weaker option.
            "require" | "verify-full" => true,
            "prefer" | "allow" => {
                return Err(
                    "--grant-db: sslmode=prefer/allow would fall back to cleartext without \
                     telling you; use require or disable"
                        .into(),
                );
            }
            "verify-ca" => {
                return Err(
                    "--grant-db: sslmode=verify-ca is not supported; use verify-full, which \
                     also checks the hostname"
                        .into(),
                );
            }
            other => {
                return Err(format!(
                    "--grant-db: unknown sslmode `{other}` (have: disable, require, verify-full)"
                ));
            }
        };
    }
    Ok(tls)
}
