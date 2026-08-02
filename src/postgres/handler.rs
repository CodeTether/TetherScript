//! [`QueryHandler`] adapter over the native client.
//!
//! Bridges the `db` capability contract in [`crate::database`] to the wire client
//! in [`crate::postgres`], so a `.tether` script can call `db.query(..)` with no
//! driver dependency anywhere.
//!
//! Queries are served from a [`Pool`], because `http_serve` is single-threaded and
//! a single connection would serialize every request behind the slowest statement.
//! A transaction pins one connection for its whole lifetime, since statements sent
//! to a different connection would silently fall outside it.
//!
//! # Examples
//!
//! ```rust,no_run
//! use std::rc::Rc;
//! use tetherscript::database::DatabaseAuthority;
//! use tetherscript::plugin::PluginHost;
//! use tetherscript::postgres::{Config, PostgresHandler};
//!
//! # fn main() -> Result<(), String> {
//! let handler = PostgresHandler::connect(&Config {
//!     host: "127.0.0.1".into(),
//!     port: 5432,
//!     user: "tsuser".into(),
//!     password: "pencil".into(),
//!     database: "tsdb".into(),
//! })?;
//!
//! let mut host = PluginHost::new();
//! host.grant("db", Rc::new(DatabaseAuthority::new(handler)));
//! # Ok(())
//! # }
//! ```

use std::cell::RefCell;

use super::connection::{Config, Connection};
use super::pool::Pool;
use crate::database::QueryHandler;
use crate::value::Value;

/// Connections opened by default. Small because the runtime is single-threaded:
/// the pool exists to avoid head-of-line blocking across nested handlers, not to
/// exploit parallelism the runtime does not have.
const DEFAULT_POOL_SIZE: usize = 4;

/// A [`QueryHandler`] backed by a pool of native PostgreSQL connections.
pub struct PostgresHandler {
    pool: Pool,
    /// Connection pinned by an open transaction, if any.
    transaction: RefCell<Option<Connection>>,
}

impl PostgresHandler {
    /// Connect and authenticate, returning a handler ready to grant as `db`.
    ///
    /// Opens one connection immediately so a bad address or credential fails here
    /// rather than at the script's first query, then keeps it for reuse.
    ///
    /// # Arguments
    ///
    /// * `config` — Address, credentials, and target database.
    ///
    /// # Errors
    ///
    /// Returns a `postgres:`-prefixed message when connecting or authenticating
    /// fails.
    pub fn connect(config: &Config) -> Result<Self, String> {
        Self::with_pool_size(config, DEFAULT_POOL_SIZE)
    }

    /// Connect with an explicit maximum pool size.
    ///
    /// # Errors
    ///
    /// Returns an error when the first connection cannot be established.
    pub fn with_pool_size(config: &Config, max_size: usize) -> Result<Self, String> {
        let pool = Pool::new(config.clone(), max_size);
        pool.release(pool.acquire()?);
        Ok(Self {
            pool,
            transaction: RefCell::new(None),
        })
    }
}

impl QueryHandler for PostgresHandler {
    /// Execute `sql` with bound `parameters` through the extended protocol.
    ///
    /// Always parameterized, even for an empty list, so a script cannot reach the
    /// unparameterized path by accident.
    fn query(&self, sql: &str, parameters: &[Value]) -> Result<Value, String> {
        super::handler_exec::query(self, sql, parameters)
    }

    fn begin(&self) -> Result<(), String> {
        super::handler_tx::begin(self)
    }

    fn commit(&self) -> Result<(), String> {
        super::handler_tx::finish(self, "COMMIT")
    }

    fn rollback(&self) -> Result<(), String> {
        super::handler_tx::finish(self, "ROLLBACK")
    }

    fn pool_size(&self) -> usize {
        self.pool.size()
    }
}

impl PostgresHandler {
    /// Pool access for the split-out execution and transaction modules.
    pub(super) fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Pinned transaction slot for the split-out modules.
    pub(super) fn transaction(&self) -> &RefCell<Option<Connection>> {
        &self.transaction
    }
}
