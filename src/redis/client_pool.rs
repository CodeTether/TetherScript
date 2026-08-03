//! Connection pool for the Redis client.
//!
//! Shaped after `src/postgres/pool.rs`, for the same reason it exists there: the
//! HTTP server runs a single-threaded accept loop, so one shared connection
//! serialises every request behind the slowest command. Connections are created
//! lazily up to `max_size`, so a script that never touches Redis pays nothing and
//! a burst grows the pool only as far as it actually needs.
//!
//! `RefCell`, not a mutex, again matching the PostgreSQL pool: the runtime is
//! single-threaded, and a lock here would be a claim of thread safety the rest of
//! the client does not make.

use std::cell::RefCell;

use super::config::Config;
use super::connection::Connection;
use super::error::ClientError;

/// Opens one new connection from the pool's settings.
///
/// Injected rather than hard-coded so the pool is testable without a server and
/// so the codec choice stays the integrator's, not the pool's.
pub type Connector = Box<dyn Fn(&Config) -> Result<Connection, ClientError>>;

/// A lazily grown set of ready connections.
///
/// # Examples
///
/// ```rust,ignore
/// use tetherscript::redis::client::{Config, Pool};
///
/// let pool = Pool::new(Config::default(), 4, connector);
/// assert_eq!(pool.size(), 0); // nothing is opened until the first lease
/// ```
pub struct Pool {
    pub(super) config: Config,
    pub(super) idle: RefCell<Vec<Connection>>,
    pub(super) live: RefCell<usize>,
    pub(super) max_size: usize,
    pub(super) connect: Connector,
}

/// Opaque: the pool holds a [`Config`] carrying a password, so a derived `Debug`
/// would risk printing it. [`Config`]'s own `Debug` redacts, and this one prints
/// nothing but the counts.
impl std::fmt::Debug for Pool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pool({} live, max {})", self.size(), self.max_size)
    }
}
