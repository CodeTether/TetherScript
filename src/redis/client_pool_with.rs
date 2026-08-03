//! [`Pool::with_connection`]: lease, run, and route the outcome correctly.
//!
//! This is the API callers should use. Doing the lease by hand is allowed, but
//! then the discard-versus-release decision documented in `client_pool_lease.rs`
//! becomes the caller's to get right on every error path, and getting it wrong is
//! silent: the pool keeps serving a misaligned connection.

use super::connection::Connection;
use super::error::ClientError;
use super::pool::Pool;

impl Pool {
    /// Run `work` against a leased connection, then release or discard it.
    ///
    /// # Arguments
    ///
    /// * `work` — Closure receiving the leased connection.
    ///
    /// # Returns
    ///
    /// Whatever `work` returns on success, after the connection has been released
    /// for reuse.
    ///
    /// # Errors
    ///
    /// [`ClientError::PoolExhausted`] when no connection is available, a connect
    /// error from the connector, or `work`'s own error. In the last case the
    /// connection is released when the error left the stream aligned and dropped
    /// when it did not, per `ClientError::discards_connection`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let cached = pool.with_connection(|c| c.get(b"render:home"))?;
    /// ```
    pub fn with_connection<T, F>(&self, work: F) -> Result<T, ClientError>
    where
        F: FnOnce(&mut Connection) -> Result<T, ClientError>,
    {
        let mut connection = self.acquire()?;
        match work(&mut connection) {
            Ok(value) => {
                self.release(connection);
                Ok(value)
            }
            Err(error) if error.discards_connection() => {
                drop(connection);
                self.discard();
                Err(error)
            }
            Err(error) => {
                self.release(connection);
                Err(error)
            }
        }
    }
}
