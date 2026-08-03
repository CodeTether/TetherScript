//! Capability adapter over the native Redis client.
//!
//! [`RedisHandler`] owns a [`Pool`] and exposes the script-facing surface a `.tether`
//! program reaches through a granted capability. It is the Redis counterpart to
//! [`crate::postgres::PostgresHandler`], and like that adapter it exists so a host
//! can grant data access without any driver dependency entering the build.
//!
//! # The correctness point that matters most
//!
//! Redis replies are read off a shared socket in order, so *how* a failure is
//! classified decides whether every later reply is correct:
//!
//! * A **transport error** — a short write, a closed socket, a read that failed
//!   part-way through a reply — leaves an unknown number of bytes queued on the
//!   connection. That connection is [`Pool::discard`]ed: reusing it would return
//!   the tail of this command as the answer to the next one, and nothing after it
//!   would line up again.
//! * A **server-side error reply** ([`super::value::RespValue::Error`], e.g. `WRONGTYPE`
//!   or `ERR value is not an integer`) is a *complete, fully drained* reply. The
//!   connection is [`Pool::release`]d and stays usable. Discarding here would be a
//!   silent bug: a script that catches a `WRONGTYPE` error in a loop would churn
//!   a new TCP connection per iteration and eventually exhaust the pool.
//!
//! That split is implemented once, in [`super::handler_exec`], so no command
//! helper can get it wrong individually.
//!
//! # Examples
//!
//! ```rust,no_run
//! use tetherscript::redis::{Config, RedisHandler};
//! use tetherscript::value::Value;
//!
//! # fn main() -> Result<(), String> {
//! // `default()` targets 127.0.0.1:6379 with explicit timeouts; TLS is not implemented.
//! let handler = RedisHandler::connect(&Config::default())?;
//!
//! handler.set("greeting", b"hello", None)?;
//! assert_eq!(handler.get("greeting")?, Value::Str("hello".to_string().into()));
//! # Ok(())
//! # }
//! ```

use super::config::Config;
use super::pool::Pool;

/// Connections opened by default.
///
/// Small on purpose: the runtime is single-threaded, so the pool prevents
/// head-of-line blocking across nested handlers rather than exploiting parallelism
/// that does not exist.
const DEFAULT_POOL_SIZE: usize = 4;

/// A pooled, script-facing Redis client.
///
/// Command helpers live in the sibling `handler_*` modules; this type owns only the
/// pool. See [`super::handler_strings`], [`super::handler_expiry`], and
/// [`super::handler_command`].
pub struct RedisHandler {
    pool: Pool,
}

impl RedisHandler {
    /// Connect and authenticate, returning a handler ready to grant to a script.
    ///
    /// Opens one connection immediately so a bad address, password, or database
    /// index fails here rather than at the script's first command, then keeps that
    /// connection for reuse.
    ///
    /// # Arguments
    ///
    /// * `config` — Address, credentials, and logical database.
    ///
    /// # Returns
    ///
    /// A handler owning exactly one live connection.
    ///
    /// # Errors
    ///
    /// Returns the connection error when the first connection cannot be
    /// established.
    pub fn connect(config: &Config) -> Result<Self, String> {
        Self::with_pool_size(config, DEFAULT_POOL_SIZE)
    }

    /// Connect with an explicit maximum pool size.
    ///
    /// # Arguments
    ///
    /// * `config` — Address, credentials, and logical database.
    /// * `max_size` — Ceiling on concurrently owned connections; clamped to 1.
    ///
    /// # Returns
    ///
    /// A handler owning exactly one live connection, able to grow to `max_size`.
    ///
    /// # Errors
    ///
    /// Returns the connection error when the first connection cannot be
    /// established.
    pub fn with_pool_size(config: &Config, max_size: usize) -> Result<Self, String> {
        let pool = Pool::new(config.clone(), max_size);
        pool.release(pool.acquire()?);
        Ok(Self { pool })
    }

    /// Connections currently owned by the pool, idle or leased, for diagnostics.
    ///
    /// # Returns
    ///
    /// A count that should stay flat across sequential commands: a climbing value
    /// means connections are being discarded and reopened.
    pub fn pool_size(&self) -> usize {
        self.pool.size()
    }

    /// Pool access for the split-out command modules.
    pub(super) fn pool(&self) -> &Pool {
        &self.pool
    }
}
