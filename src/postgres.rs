//! # Native PostgreSQL client
//!
//! Speaks the PostgreSQL v3 frontend/backend protocol directly over TCP, so the
//! core build gains a real database client with **no driver dependency**. This
//! keeps the promise in [`crate::database`] honest: hosts can now grant a `db`
//! capability without pulling SQLx or `tokio-postgres` into the build.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use tetherscript::postgres::{Config, Connection};
//! use tetherscript::value::Value;
//!
//! # fn main() -> Result<(), String> {
//! let mut connection = Connection::connect(&Config {
//!     host: "127.0.0.1".into(),
//!     port: 5432,
//!     user: "tsuser".into(),
//!     password: "pencil".into(),
//!     database: "tsdb".into(),
//! })?;
//!
//! // Rows are a list of maps keyed by column name. Prefer `query` whenever a
//! // value comes from outside: it binds parameters instead of splicing SQL.
//! let rows = connection.query(
//!     "SELECT id, name FROM users WHERE id = $1",
//!     &[Value::Int(1)],
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! Each concern is one module, matching the shape of the protocol itself:
//!
//! | Module | Responsibility |
//! |---|---|
//! | `encode` / `decode` / `cursor` | Length-prefixed message framing |
//! | `startup` | StartupMessage through `ReadyForQuery` |
//! | `auth` / `sasl` / `scram` | Authentication negotiation |
//! | `hmac` / `md5` | Crypto primitives for SCRAM and legacy `md5` |
//! | `extended` / `params` | `Parse`/`Bind` and parameter encoding |
//! | `query` / `collect` / `rows` / `error` | Execution and row decoding |
//! | `handler` | `QueryHandler` adapter for the `db` capability |
//!
//! Crypto is built on the in-tree SHA-256 in [`crate::system`]. HMAC-SHA-256,
//! PBKDF2, and MD5 are verified against the published vectors in RFC 4231,
//! RFC 6070, RFC 1321, and RFC 7677 respectively, since a silently wrong digest
//! would surface only as an unexplained authentication failure.
//!
//! ## Scope and limits
//!
//! Read these before relying on the client:
//!
//! - **No TLS.** Connections are cleartext, so credentials and rows cross the
//!   network unprotected. Use it on a trusted network or through a tunnel until
//!   TLS is wired through the optional `openssl-tls` transport.
//! - **Parameters bind, but only as text.** [`Connection::query`] uses the
//!   extended protocol, so values never enter the SQL string. Types are inferred
//!   by the server; str, int, float, bool, and nil are supported.
//!   [`Connection::simple_query`] takes no parameters, so anything untrusted
//!   belongs in `query`.
//! - **Text-format decoding.** Values arrive as text and are converted
//!   heuristically, so exact SQL types need an explicit cast in the query.
//! - **One connection, synchronous.** Pooling belongs to the host, matching how
//!   [`crate::database::DatabaseAuthority`] is granted per request.
//!
//! ## Reaching it from a script
//!
//! [`PostgresHandler`] implements [`crate::database::QueryHandler`], so a host can
//! grant it as the `db` capability and a `.tether` script can call
//! `db.query(sql, [params])`. Scripts have no ambient database access: `db` is
//! undefined unless granted. See `examples/db_capability.rs` and
//! `examples/db_capability.tether`.
//!
//! ## Testing
//!
//! Protocol correctness cannot be proven against a mock, so the wire-level tests
//! in `tests/postgres_live.rs` run against a real server and are skipped unless
//! `TETHERSCRIPT_PG_TEST_URL` is set:
//!
//! ```text
//! docker run -d --rm --name ts_pg_test -e POSTGRES_PASSWORD=pencil \
//!   -e POSTGRES_USER=tsuser -e POSTGRES_DB=tsdb -p 55432:5432 postgres:16
//! TETHERSCRIPT_PG_TEST_URL=127.0.0.1:55432 cargo test --test postgres_live
//! ```

mod auth;
mod collect;
mod connection;
mod cursor;
mod decode;
mod encode;
mod error;
mod extended;
mod handler;
mod hmac;
mod md5;
mod md5_block;
mod md5_constants;
mod md5_password;
mod nonce;
mod params;
mod query;
mod rows;
mod sasl;
mod scram;
mod startup;

// `Connection` is the embedding surface and is unused by the CLI binary itself,
// which reaches the client only through `PostgresHandler`.
#[allow(unused_imports)]
pub use connection::{Config, Connection};
pub use handler::PostgresHandler;

#[cfg(test)]
mod hmac_tests;

#[cfg(test)]
mod extended_tests;

#[cfg(test)]
mod md5_tests;

#[cfg(test)]
mod scram_tests;
