//! Post-connect handshake: optional `AUTH`, optional `SELECT`.
//!
//! Both steps are conditional, and both are skipped in the case where sending them
//! would be wrong rather than merely redundant:
//!
//! - `AUTH` is sent only when a password is configured. Sending it to a server
//!   without `requirepass` is an error reply (`ERR Client sent AUTH, but no
//!   password is set`), so an unconditional `AUTH` would break the common
//!   development setup.
//! - `SELECT` is sent only for a non-zero database index, since every connection
//!   already starts on database 0.
//!
//! The password is passed as a bulk-string argument like any other value, so a
//! password containing spaces or CRLF authenticates correctly instead of injecting.

use super::config::Config;
use super::connection::Connection;
use super::error::RedisError;

/// Run the handshake against a freshly connected socket.
///
/// # Arguments
///
/// * `connection` — Newly dialled connection.
/// * `config` — Credentials and database index.
///
/// # Errors
///
/// [`RedisError::Server`] when the credentials are rejected or the database index
/// is out of range, or [`RedisError::Transport`] when the socket fails mid
/// handshake.
pub(super) fn run(connection: &mut Connection, config: &Config) -> Result<(), RedisError> {
    if let Some(password) = &config.password {
        authenticate(connection, config.username.as_deref(), password)?;
    }
    if config.database != 0 {
        let index = config.database.to_string();
        connection
            .command(&[&b"SELECT"[..], index.as_bytes()])?
            .simple("SELECT")?;
    }
    Ok(())
}

/// Send `AUTH`, using the two-argument ACL form when a username is configured.
fn authenticate(
    connection: &mut Connection,
    username: Option<&str>,
    password: &str,
) -> Result<(), RedisError> {
    let reply = match username {
        Some(user) => connection.command(&[&b"AUTH"[..], user.as_bytes(), password.as_bytes()])?,
        None => connection.command(&[&b"AUTH"[..], password.as_bytes()])?,
    };
    reply.simple("AUTH")?;
    Ok(())
}
