//! A single PostgreSQL connection over TCP.
//!
//! Deliberately synchronous and single-connection: the `db` capability is
//! request-scoped, so pooling belongs to the host, matching how
//! `DatabaseAuthority` is granted.

use std::net::TcpStream;

use super::{query, startup};
use crate::value::Value;

/// Connection settings resolved by the caller.
///
/// Nothing here is read from the ambient environment: the host decides where
/// credentials come from, which keeps this type usable from a capability grant
/// without the client reaching for `DATABASE_URL` on its own.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::Config;
///
/// let config = Config {
///     host: "127.0.0.1".into(),
///     port: 5432,
///     user: "tsuser".into(),
///     password: "pencil".into(),
///     database: "tsdb".into(),
/// };
/// assert_eq!(config.port, 5432);
/// ```
///
/// `Clone` is derived so a connection pool can reconnect from the same settings.
#[derive(Clone)]
pub struct Config {
    /// Hostname or IP address of the server.
    pub host: String,
    /// TCP port; PostgreSQL's default is 5432.
    pub port: u16,
    /// Role to authenticate as.
    pub user: String,
    /// Password for `user`. Never logged, and omitted from `Debug` output.
    pub password: String,
    /// Database to attach to after authentication.
    pub database: String,
}

/// An authenticated connection ready to accept queries.
///
/// Created by [`Connection::connect`], which completes the startup handshake and
/// authentication before returning, so a value of this type is always ready for
/// [`Connection::simple_query`].
///
/// # Examples
///
/// ```rust,no_run
/// use tetherscript::postgres::{Config, Connection};
///
/// # fn main() -> Result<(), String> {
/// let config = Config {
///     host: "127.0.0.1".into(),
///     port: 5432,
///     user: "tsuser".into(),
///     password: "pencil".into(),
///     database: "tsdb".into(),
/// };
/// let mut connection = Connection::connect(&config)?;
/// let rows = connection.simple_query("SELECT id, name FROM users")?;
/// println!("{rows:?}");
/// # Ok(())
/// # }
/// ```
pub struct Connection {
    pub(super) stream: TcpStream,
}

/// Prints only the peer address: never credentials, which a panic could leak.
impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let peer = self
            .stream
            .peer_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|_| "disconnected".into());
        write!(f, "Connection({peer})")
    }
}

impl Connection {
    /// Connect, authenticate, and wait for `ReadyForQuery`.
    ///
    /// Negotiates whichever authentication method the server requests: trust,
    /// cleartext `password`, legacy `md5`, or SCRAM-SHA-256.
    ///
    /// # Arguments
    ///
    /// * `config` — Address, credentials, and target database.
    ///
    /// # Returns
    ///
    /// A connection that has completed startup and is ready for queries.
    ///
    /// # Errors
    ///
    /// Returns a `postgres:`-prefixed message when the TCP connect fails, the
    /// credentials are rejected, the server requests an unsupported
    /// authentication method, or the startup sequence is malformed.
    ///
    /// # Examples
    ///
    /// See the [`Connection`] example; connecting requires a live server, so the
    /// snippet there is `no_run`.
    pub fn connect(config: &Config) -> Result<Self, String> {
        let stream = TcpStream::connect((config.host.as_str(), config.port)).map_err(|error| {
            format!(
                "postgres: connect to {}:{}: {error}",
                config.host, config.port
            )
        })?;
        let mut connection = Self { stream };
        startup::run(&mut connection, config)?;
        Ok(connection)
    }

    /// Execute SQL through the simple-query protocol and collect rows.
    ///
    /// Rows come back as a [`Value::List`] of [`Value::Map`]s keyed by column
    /// name. Fields arrive in text format, so `t`/`f` become [`Value::Bool`],
    /// numeric strings become [`Value::Int`] or [`Value::Float`], SQL `NULL`
    /// becomes [`Value::Nil`], and anything else stays a [`Value::Str`].
    ///
    /// # Arguments
    ///
    /// * `sql` — One or more statements. Because this is the *simple* query
    ///   protocol, values cannot be bound as parameters yet; any untrusted input
    ///   must not be concatenated into this string.
    ///
    /// # Returns
    ///
    /// The decoded rows. Statements returning no rows yield an empty list.
    ///
    /// # Errors
    ///
    /// Returns the server's `ErrorResponse` text, including severity and
    /// SQLSTATE, or a transport error. The connection stays usable afterwards
    /// because the reply is drained through `ReadyForQuery` before returning.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use tetherscript::postgres::{Config, Connection};
    /// # fn run(connection: &mut Connection) -> Result<(), String> {
    /// let rows = connection.simple_query("SELECT 1 AS one")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn simple_query(&mut self, sql: &str) -> Result<Value, String> {
        query::run(self, sql)
    }

    /// Execute SQL with bound parameters through the extended query protocol.
    ///
    /// This is the safe path for untrusted values. The statement is parsed once
    /// with `$1`-style placeholders and the values are sent separately, so a
    /// parameter can never change the shape of the SQL. Prefer it over
    /// [`Connection::simple_query`] whenever a value comes from outside.
    ///
    /// # Arguments
    ///
    /// * `sql` — A single statement using `$1`, `$2`, … placeholders.
    /// * `parameters` — Values to bind, positionally. Supported types are str,
    ///   int, float, bool, and nil (SQL NULL).
    ///
    /// # Returns
    ///
    /// The decoded rows, in the same shape as [`Connection::simple_query`].
    ///
    /// # Errors
    ///
    /// Returns an error naming the position and type of any parameter that has no
    /// text-format encoding, the server's `ErrorResponse`, or a transport error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use tetherscript::postgres::Connection;
    /// # use tetherscript::value::Value;
    /// # fn run(connection: &mut Connection) -> Result<(), String> {
    /// let rows = connection.query("SELECT name FROM users WHERE id = $1", &[Value::Int(1)])?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn query(&mut self, sql: &str, parameters: &[Value]) -> Result<Value, String> {
        query::run_params(self, sql, parameters)
    }
}
