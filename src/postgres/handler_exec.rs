//! Query execution against a pooled or pinned connection.
//!
//! Split from [`super::handler`] to keep each file within the line budget and to
//! isolate the lease/return discipline, which is where a leak or a misaligned
//! connection would come from.

use super::handler::PostgresHandler;
use crate::value::Value;

/// Run `sql` on the pinned transaction connection, or on a leased one.
///
/// # Errors
///
/// Returns the server's error, a transport error, or a pool-exhaustion message.
pub(super) fn query(
    handler: &PostgresHandler,
    sql: &str,
    parameters: &[Value],
) -> Result<Value, String> {
    if let Some(connection) = handler.transaction().borrow_mut().as_mut() {
        // Inside a transaction every statement must use the pinned connection, or
        // it would run outside the transaction and survive a rollback.
        return connection.query(sql, parameters);
    }
    let mut connection = handler.pool().acquire()?;
    match connection.query(sql, parameters) {
        Ok(rows) => {
            handler.pool().release(connection);
            Ok(rows)
        }
        Err(error) if is_protocol_failure(&error) => {
            // The exchange was abandoned mid-stream, so unread bytes may remain
            // queued. Dropping the connection is cheaper than misreading every
            // later reply on it.
            handler.pool().discard();
            Err(error)
        }
        Err(error) => {
            // A server-side SQL error leaves the connection drained and reusable,
            // because the reply was read through ReadyForQuery.
            handler.pool().release(connection);
            Err(error)
        }
    }
}

/// Whether an error means the connection's protocol state is unknown.
fn is_protocol_failure(error: &str) -> bool {
    error.contains("send") || error.contains("read")
}
