//! Database capability selection for CLI grants.
//!
//! Turns a `--grant-db postgres://…` argument into a [`DatabaseAuthority`] backed
//! by the in-tree PostgreSQL client, so a script reaches SQL through the `db`
//! capability without the host embedding tetherscript in Rust.
//!
//! Unlike the filesystem grant, this is never implied by `--access-mode full`: a
//! connection string carries credentials that cannot be guessed from the
//! environment, so the grant must always be explicit.

use crate::database::DatabaseAuthority;
use crate::postgres::{Config, PostgresHandler};

/// Build the `db` authority for an explicit `--grant-db` argument.
///
/// # Arguments
///
/// * `explicit` — Connection string, or `None` when the flag was absent.
///
/// # Returns
///
/// `Ok(None)` when no grant was requested, so the `db` name stays undefined.
///
/// # Errors
///
/// Returns an error when the URL cannot be parsed or the connection fails, since
/// a script that expects a database should not start without one.
pub(super) fn authority(explicit: &Option<String>) -> Result<Option<DatabaseAuthority>, String> {
    let Some(url) = explicit else {
        return Ok(None);
    };
    let config = parse_url(url)?;
    let handler = PostgresHandler::connect(&config)?;
    Ok(Some(DatabaseAuthority::new(handler)))
}

/// Parse `postgres://user:password@host:port/database`.
///
/// # Errors
///
/// Returns an error naming the missing component, because a connection string
/// silently defaulting to the wrong database is worse than refusing to start.
pub(super) fn parse_url(url: &str) -> Result<Config, String> {
    let rest = url
        .strip_prefix("postgres://")
        .or_else(|| url.strip_prefix("postgresql://"))
        .ok_or_else(|| format!("--grant-db must start with postgres:// (got `{url}`)"))?;
    let (credentials, location) = rest
        .split_once('@')
        .ok_or("--grant-db needs user:password@host (no `@` found)")?;
    let (user, password) = credentials.split_once(':').unwrap_or((credentials, ""));
    let (authority, database) = location
        .split_once('/')
        .ok_or("--grant-db needs a /database path")?;
    let (host, port) = super::db_port::split(authority)?;
    Ok(Config {
        host: host.to_string(),
        port,
        user: user.to_string(),
        password: password.to_string(),
        database: database.to_string(),
    })
}
