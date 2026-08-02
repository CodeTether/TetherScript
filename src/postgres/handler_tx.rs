//! Transaction lifecycle for the pooled handler.
//!
//! A transaction pins one connection from `BEGIN` until `COMMIT` or `ROLLBACK`.
//! Without pinning, a pooled handler could send the statements to a different
//! connection, where they would commit independently and survive a rollback —
//! silent data corruption rather than a visible error.

use super::handler::PostgresHandler;

/// Open a transaction and pin its connection.
///
/// # Errors
///
/// Returns an error when a transaction is already open, the pool is exhausted, or
/// the `BEGIN` itself fails. Nested transactions are refused rather than silently
/// flattened, because a caller that believes it has an inner scope would otherwise
/// have its rollback discard the outer work too.
pub(super) fn begin(handler: &PostgresHandler) -> Result<(), String> {
    if handler.transaction().borrow().is_some() {
        return Err("db.begin: a transaction is already open".into());
    }
    let mut connection = handler.pool().acquire()?;
    connection.query("BEGIN", &[])?;
    *handler.transaction().borrow_mut() = Some(connection);
    Ok(())
}

/// Finish the open transaction with `COMMIT` or `ROLLBACK`, then unpin.
///
/// # Arguments
///
/// * `verb` — Either `COMMIT` or `ROLLBACK`.
///
/// # Errors
///
/// Returns an error when no transaction is open, or when the statement fails. The
/// connection is returned to the pool either way: leaving it pinned after a failed
/// commit would leak it and eventually exhaust the pool.
pub(super) fn finish(handler: &PostgresHandler, verb: &str) -> Result<(), String> {
    let Some(mut connection) = handler.transaction().borrow_mut().take() else {
        return Err(format!(
            "db.{}: no transaction is open",
            verb.to_ascii_lowercase()
        ));
    };
    let outcome = connection.query(verb, &[]);
    match &outcome {
        // A failed COMMIT/ROLLBACK leaves protocol state unclear, so the
        // connection is dropped rather than reused.
        Err(_) => handler.pool().discard(),
        Ok(_) => handler.pool().release(connection),
    }
    outcome.map(|_| ())
}
