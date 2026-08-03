//! Post-connect handshake: optional `AUTH`, optional `SELECT`.
//!
//! Both steps are conditional, and each is skipped in the case where sending it
//! would be *wrong* rather than merely redundant:
//!
//! - `AUTH` is sent only when a password is configured. Against a server without
//!   `requirepass` it is an error reply (`ERR Client sent AUTH, but no password is
//!   set`), so an unconditional `AUTH` would break the common development setup.
//! - `SELECT` is sent only for a non-zero index, since a connection already starts
//!   on database 0.
//!
//! The password travels as an ordinary bulk-string argument, so one containing
//! spaces or CRLF authenticates correctly rather than injecting a second command.
//! It is never formatted into a message: the only borrow of it is the argument
//! slice below.

use super::config::Config;
use super::connection::Connection;
use super::error::ClientError;

impl Connection {
    /// Authenticate and select, as configured.
    ///
    /// # Arguments
    ///
    /// * `config` — Credentials and database index.
    ///
    /// # Returns
    ///
    /// `()` once every required step has been acknowledged.
    ///
    /// # Errors
    ///
    /// [`ClientError::Server`] when `AUTH` or `SELECT` is refused — the message
    /// carries the server's own text, which never echoes the password — or a
    /// transport error. Never includes the password itself.
    pub(super) fn handshake(&mut self, config: &Config) -> Result<(), ClientError> {
        if let Some(password) = &config.password {
            self.command(&[&b"AUTH"[..], password.as_bytes()])?;
        }
        match config.database {
            Some(index) if index != 0 => {
                let index = index.to_string();
                self.command(&[&b"SELECT"[..], index.as_bytes()])?;
            }
            _ => {}
        }
        Ok(())
    }
}
