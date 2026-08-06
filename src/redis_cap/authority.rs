//! The `redis` authority: the object a granted script holds.
//!
//! One concern: owning the connection and handing it out under `&mut`. Method
//! dispatch lives in `super::invoke`, the command bodies in `super::methods`
//! and `super::methods_ttl`.
//!
//! # There is no ambient Redis access
//!
//! Nothing in this module is reachable unless the host constructs a
//! [`RedisAuthority`] and grants it, exactly as `db` is granted from
//! `--grant-db`. Without `--grant-redis` the global name `redis` is never bound, so
//! a script referencing it fails with an undefined-variable error rather than
//! quietly connecting to `127.0.0.1:6379`. This mirrors `--grant-db` and, like it,
//! is **not** implied by `--access-mode full`: a Redis URL carries credentials and a
//! database index that cannot be guessed from the environment.
//!
//! Contrast the `fs_*` builtins, which AGENTS.md records as still bypassing
//! capability grants. There is deliberately no `redis_get` builtin, so the
//! capability is the only path and there is no ambient hole to close later.
//!
//! # Interior mutability
//!
//! [`Authority::invoke`](crate::capability::Authority::invoke) takes `&self`, but
//! [`Connection`] commands need `&mut self` because a command writes a request and
//! then reads its reply. The connection therefore sits in a [`RefCell`]. The runtime
//! is single-threaded and no command re-enters the capability, so the borrow is
//! never contended; `super::invoke` still reports a failed borrow as an error
//! rather than panicking.

use std::cell::RefCell;

use crate::redis::{Config, Connection, RedisError};

/// A connected Redis capability.
///
/// Deliberately not `Debug`: the settings that produced this connection included a
/// password, and a panic message must never print one.
pub struct RedisAuthority {
    pub(super) connection: RefCell<Connection>,
}

impl RedisAuthority {
    /// Connect, authenticate, and select the configured database.
    ///
    /// Connecting here rather than lazily means a bad address, password, or database
    /// index fails while the CLI is still parsing its grant, not midway through a
    /// script.
    ///
    /// # Arguments
    ///
    /// * `config` — Address, credentials, database index, and timeouts.
    ///
    /// # Returns
    ///
    /// An authority ready to grant under the name `redis`.
    ///
    /// # Errors
    ///
    /// Returns the client's [`RedisError`] when the socket, `AUTH`, or `SELECT`
    /// fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tetherscript::redis::Config;
    /// use tetherscript::redis_cap::RedisAuthority;
    ///
    /// # fn main() -> Result<(), tetherscript::redis::RedisError> {
    /// let authority = RedisAuthority::connect(&Config::default())?;
    /// # let _ = authority;
    /// # Ok(())
    /// # }
    /// ```
    pub fn connect(config: &Config) -> Result<Self, RedisError> {
        Ok(Self {
            connection: RefCell::new(Connection::connect(config)?),
        })
    }
}
