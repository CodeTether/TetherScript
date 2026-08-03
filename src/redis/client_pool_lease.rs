//! Leasing: `new`, `acquire`, `release`, `discard`, `size`.
//!
//! # Why release and discard are different operations
//!
//! Redis is strictly request/response per connection, so reply *n* is only
//! identifiable by having consumed replies `1..n` first. That makes the state of a
//! connection after a failure the deciding question:
//!
//! - The exchange **completed** — including when the server answered
//!   `-WRONGTYPE`, or answered a type the command did not want. Exactly one reply
//!   was consumed for exactly one request, the stream is aligned, and the
//!   connection goes back to the idle list via [`Pool::release`].
//! - The exchange was **abandoned** — a write failed, a read timed out, the peer
//!   closed mid-reply, or the bytes were not RESP. Unread or unparsable bytes may
//!   still be queued, so the next command on that socket would read this
//!   command's leftovers and every later reply would be off by one, silently
//!   returning the wrong value for the wrong key. The connection is dropped and
//!   its slot freed via [`Pool::discard`], so the pool opens a clean replacement.
//!
//! `ClientError::discards_connection` encodes the rule so no call site has to
//! re-derive it, and [`Pool::with_connection`] applies it automatically.

use std::cell::RefCell;

use super::config::Config;
use super::connection::Connection;
use super::error::ClientError;
use super::pool::{Connector, Pool};

impl Pool {
    /// Create an empty pool. No connection is opened until the first lease.
    ///
    /// # Arguments
    ///
    /// * `config` — Settings every connection is opened with.
    /// * `max_size` — Ceiling on owned connections. Clamped up to `1`, since a
    ///   pool of zero could never serve anything.
    /// * `connect` — Factory that opens one connection.
    pub fn new(config: Config, max_size: usize, connect: Connector) -> Self {
        Self {
            config,
            idle: RefCell::new(Vec::new()),
            live: RefCell::new(0),
            max_size: max_size.max(1),
            connect,
        }
    }

    /// Take an idle connection, or open one while the pool may still grow.
    ///
    /// # Returns
    ///
    /// A leased connection, owned by the caller until it is handed back through
    /// [`Pool::release`] or dropped alongside [`Pool::discard`].
    ///
    /// # Errors
    ///
    /// [`ClientError::PoolExhausted`], naming the limit because the fix is a
    /// larger pool rather than a retry, or whatever the connector returns.
    pub fn acquire(&self) -> Result<Connection, ClientError> {
        if let Some(connection) = self.idle.borrow_mut().pop() {
            return Ok(connection);
        }
        let mut live = self.live.borrow_mut();
        if *live >= self.max_size {
            return Err(ClientError::PoolExhausted {
                in_use: *live,
                max: self.max_size,
            });
        }
        let connection = (self.connect)(&self.config)?;
        *live += 1;
        Ok(connection)
    }

    /// Return a connection whose stream is still aligned.
    ///
    /// # Arguments
    ///
    /// * `connection` — A connection whose last exchange completed.
    pub fn release(&self, connection: Connection) {
        self.idle.borrow_mut().push(connection);
    }

    /// Forget a connection whose protocol state is unknown.
    ///
    /// The connection itself is dropped by the caller; this frees its slot so the
    /// pool may open a replacement.
    pub fn discard(&self) {
        let mut live = self.live.borrow_mut();
        *live = live.saturating_sub(1);
    }

    /// Connections currently owned by the pool, idle or leased.
    pub fn size(&self) -> usize {
        *self.live.borrow()
    }
}
