//! Lease discipline for the Redis connection pool: acquire, release, discard.
//!
//! Split from [`super::pool`] so the state layout and the lease rules can be read
//! independently. This is where a connection leak or a misaligned reply stream
//! would come from, so it is one file with one concern.
//!
//! The three operations are not interchangeable:
//!
//! | Operation | When | Effect on `live` |
//! |---|---|---|
//! | [`acquire`] | Command needs a connection | `+1` only when a new socket opens |
//! | [`release`] | Reply fully drained | unchanged; connection goes back to `idle` |
//! | [`discard`] | Reply state unknown | `-1`; connection is dropped |

use super::connection::Connection;
use super::pool::Pool;

/// Take an idle connection, or open one when the pool may still grow.
///
/// Reuses an idle connection first, because an already-authenticated connection
/// costs nothing to hand out while a new one costs a TCP handshake plus `AUTH`.
///
/// # Arguments
///
/// * `pool` — The pool to lease from.
///
/// # Returns
///
/// An owned [`Connection`]. The caller owns it until it calls
/// [`release`] or [`discard`]; dropping it without either leaks a slot.
///
/// # Errors
///
/// Returns an error when the pool is exhausted, or when opening a new connection
/// fails. The exhaustion message names both the in-use count and the configured
/// maximum, because the fix is a larger pool rather than a retry: every leased
/// connection is held by a caller on this same single thread, so blocking or
/// retrying here could never make one available.
pub(super) fn acquire(pool: &Pool) -> Result<Connection, String> {
    if let Some(connection) = pool.idle.borrow_mut().pop() {
        return Ok(connection);
    }
    let mut live = pool.live.borrow_mut();
    if *live >= pool.max_size {
        return Err(format!(
            "redis: connection pool exhausted ({} in use, max {}); increase the pool size",
            *live, pool.max_size
        ));
    }
    // `Connection::connect` reports a typed RedisError; the pool's callers work in
    // strings, so it is rendered here rather than widening the pool's error type.
    let connection =
        Connection::connect(&pool.config).map_err(|error| format!("redis: {error}"))?;
    *live += 1;
    Ok(connection)
}

/// Return a healthy connection for reuse.
///
/// # Arguments
///
/// * `pool` — The owning pool.
/// * `connection` — A connection whose last reply was read to completion and whose
///   per-connection state is still the pool default. A connection left in a
///   `SELECT`ed database or in subscribe mode must **not** be released; see the
///   statefulness notes on [`super::pool`].
pub(super) fn release(pool: &Pool, connection: Connection) {
    pool.idle.borrow_mut().push(connection);
}

/// Drop a connection whose protocol state is unknown after a transport failure.
///
/// A command abandoned mid-exchange may leave unread bytes queued on the socket —
/// a partially written reply, or a reply that arrived after the read timed out.
/// Reusing such a connection would return the *previous* command's tail as the
/// next command's answer and misalign every reply after it. Forgetting it lets
/// [`acquire`] open a clean replacement.
///
/// # Arguments
///
/// * `pool` — The owning pool. Its `live` count is decremented, saturating at
///   zero so a double-discard cannot underflow into a pool that never grows again.
pub(super) fn discard(pool: &Pool) {
    let mut live = pool.live.borrow_mut();
    *live = live.saturating_sub(1);
}
