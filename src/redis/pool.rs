//! Connection pool state for the native Redis client.
//!
//! The runtime is single-threaded, so this pool does not exist to exploit
//! parallelism: it exists so nested handlers (an `http_serve` route that calls a
//! helper that also touches Redis) do not serialize behind one socket, and so a
//! connection that failed mid-reply can be replaced instead of poisoning every
//! later command.
//!
//! Connections are created lazily up to `max_size`. A script that never issues a
//! command pays for nothing, and a burst grows the pool only as far as it needs.
//!
//! # Statefulness: what must never be pooled
//!
//! Unlike an HTTP connection, a Redis connection carries per-connection state.
//! Two kinds of state make a connection **unsafe to return to a shared pool**:
//!
//! * **`SELECT <n>`** — the selected logical database is a property of the
//!   connection. A connection left on database 3 and released into the pool will
//!   silently serve a later, unrelated command against database 3. Keys appear to
//!   vanish. Either restore the configured database before releasing, or pin the
//!   connection for the whole lifetime of that database's use.
//! * **`SUBSCRIBE` / `PSUBSCRIBE` / `MONITOR`** — these put the connection into a
//!   push mode where the server sends unsolicited frames. A pooled reader would
//!   read a published message where it expected its own reply, and every reply
//!   after that would be off by one. A subscribed connection must stay pinned to
//!   its subscriber for as long as the subscription lives, and must be
//!   [`Pool::discard`]ed rather than released when the subscriber goes away.
//!
//! This module deliberately offers no `select` or `subscribe` helper. Adding one
//! means adding pinning, which is a separate change.
//!
//! Lease mechanics live in [`super::pool_lease`]; this file owns only the state.

use std::cell::RefCell;

use super::config::Config;
use super::connection::Connection;

/// A lazily-grown set of authenticated Redis connections.
///
/// `live` counts every connection the pool owns, idle **or** leased out, which is
/// what the `max_size` limit is measured against.
pub(super) struct Pool {
    pub(super) config: Config,
    pub(super) idle: RefCell<Vec<Connection>>,
    pub(super) live: RefCell<usize>,
    pub(super) max_size: usize,
}

impl Pool {
    /// Create an empty pool. No socket is opened until the first lease.
    ///
    /// # Arguments
    ///
    /// * `config` — Address, credentials, and logical database for new connections.
    /// * `max_size` — Hard ceiling on owned connections; clamped to at least 1, so
    ///   a caller passing `0` gets a usable pool rather than a deadlock.
    ///
    /// # Returns
    ///
    /// An idle pool owning zero connections.
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
    /// # Returns
    ///
    /// An owned [`Connection`] the caller must hand back with [`Pool::release`] or
    /// account for with [`Pool::discard`].
    ///
    /// # Errors
    ///
    /// Returns an error when the pool is exhausted or a new connection fails. See
    /// [`super::pool_lease::acquire`] for why the exhaustion message names the
    /// limit.
    pub(super) fn acquire(&self) -> Result<Connection, String> {
        super::pool_lease::acquire(self)
    }

    /// Return a connection whose reply was fully drained, for reuse.
    ///
    /// # Arguments
    ///
    /// * `connection` — A connection with no unread bytes and default state.
    pub(super) fn release(&self, connection: Connection) {
        super::pool_lease::release(self, connection)
    }

    /// Forget a connection whose protocol state is unknown.
    pub(super) fn discard(&self) {
        super::pool_lease::discard(self)
    }

    /// Connections currently owned by the pool, idle or leased.
    ///
    /// # Returns
    ///
    /// The `live` count, which is the number compared against `max_size`.
    pub(super) fn size(&self) -> usize {
        *self.live.borrow()
    }
}
