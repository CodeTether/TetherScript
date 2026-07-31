//! [`QueryHandler`] adapter over the native client.
//!
//! Bridges the two halves that already existed separately: the `db` capability
//! contract in [`crate::database`] and the wire client in [`crate::postgres`].
//! Granting this to a [`PluginHost`](crate::plugin::PluginHost) is what lets a
//! `.tether` script call `db.query(..)` with no driver dependency anywhere.
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
use crate::database::QueryHandler;
use crate::value::Value;

/// A [`QueryHandler`] backed by one native PostgreSQL connection.
///
/// `QueryHandler::query` takes `&self`, so the connection sits behind a
/// [`RefCell`]: the protocol is stateful and every exchange needs mutable access
/// to the socket.
pub struct PostgresHandler {
    connection: RefCell<Connection>,
}

impl PostgresHandler {
    /// Connect and authenticate, returning a handler ready to grant as `db`.
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
        Ok(Self {
            connection: RefCell::new(Connection::connect(config)?),
        })
    }
}

impl QueryHandler for PostgresHandler {
    /// Execute `sql` with bound `parameters` through the extended protocol.
    ///
    /// Always parameterized, even for an empty list, so a script cannot opt into
    /// the unparameterized path by accident.
    fn query(&self, sql: &str, parameters: &[Value]) -> Result<Value, String> {
        // A reentrant db.query from inside a handler would alias the socket; a
        // named error beats a panic from the RefCell.
        let mut connection = self
            .connection
            .try_borrow_mut()
            .map_err(|_| "db.query: connection is already in use by an outer query".to_string())?;
        connection.query(sql, parameters)
    }
}
