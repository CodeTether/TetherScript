//! Connection pool for the native PostgreSQL client.
//!
//! `http_serve` runs a single-threaded accept loop, so a one-connection handler
//! serializes every request behind the slowest query. The pool keeps a small set of
//! authenticated connections and hands one to each query.
//!
//! Connections are created lazily up to `max_size`: a script that never queries
//! pays for nothing, and a burst grows the pool only as far as it actually needs.

use std::cell::RefCell;

use super::connection::{Config, Connection};

/// A lazily-grown set of authenticated connections.
pub(super) struct Pool {
    config: Config,
    idle: RefCell<Vec<Connection>>,
    live: RefCell<usize>,
    max_size: usize,
}

impl Pool {
    /// Create an empty pool. No connection is opened until the first lease.
    pub(super) fn new(config: Config, max_size: usize) -> Self {
        Self {
            config,
            idle: RefCell::new(Vec::new()),
            live: RefCell::new(0),
            max_size: max_size.max(1),
        }
    }

    /// Take an idle connection, or open one when the pool may still grow.
    ///
    /// # Errors
    ///
    /// Returns an error when the pool is exhausted or a new connection fails. The
    /// exhaustion message names the limit, because the fix is a larger pool rather
    /// than a retry.
    pub(super) fn acquire(&self) -> Result<Connection, String> {
        if let Some(connection) = self.idle.borrow_mut().pop() {
            return Ok(connection);
        }
        let mut live = self.live.borrow_mut();
        if *live >= self.max_size {
            return Err(format!(
                "db: connection pool exhausted ({} in use, max {})",
                *live, self.max_size
            ));
        }
        let connection = Connection::connect(&self.config)?;
        *live += 1;
        Ok(connection)
    }

    /// Return a healthy connection for reuse.
    pub(super) fn release(&self, connection: Connection) {
        self.idle.borrow_mut().push(connection);
    }

    /// Drop a connection whose protocol state is unknown after a failure.
    ///
    /// A connection abandoned mid-exchange may still have unread bytes queued, so
    /// reusing it would misalign every later reply. Forgetting it lets the pool
    /// open a clean replacement.
    pub(super) fn discard(&self) {
        let mut live = self.live.borrow_mut();
        *live = live.saturating_sub(1);
    }

    /// Connections currently owned by the pool, idle or leased.
    pub(super) fn size(&self) -> usize {
        *self.live.borrow()
    }
}
