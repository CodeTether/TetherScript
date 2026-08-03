//! Single point where a Redis command is executed against a pooled connection.
//!
//! Every script-facing helper funnels through [`command`], so the failure
//! classification that keeps the reply stream aligned is written exactly once.
//!
//! # Why the two failure paths differ
//!
//! [`Connection::command`](super::connection::Connection::command) distinguishes the
//! two cases structurally, which is what makes this safe to centralize:
//!
//! * `Err(_)` is a **transport** failure. The exchange was abandoned mid-stream, so
//!   an unknown number of bytes may still be queued on the socket. The connection
//!   is discarded; reusing it would answer the next command with this command's
//!   leftovers and desynchronize the connection permanently.
//! * `Ok(Resp::Error(_))` is a **server-side** rejection. The reply was framed and
//!   read to completion, so the connection is perfectly aligned. It is released
//!   back to the pool and the error is surfaced to the script. Discarding here
//!   would churn a socket per caught error and eventually exhaust the pool.

use super::handler::RedisHandler;
use super::handler_value;
use crate::value::Value;

/// Send one command on a leased connection and convert its reply.
///
/// # Arguments
///
/// * `handler` — Handler owning the pool to lease from.
/// * `args` — Command name followed by its arguments, each already encoded as
///   bytes so binary-safe values survive unchanged.
///
/// # Returns
///
/// The reply mapped into a tetherscript [`Value`] by [`handler_value::from_resp`].
///
/// # Errors
///
/// Returns a `redis:`-prefixed transport error (connection discarded), the server's
/// own error reply (connection released), a pool-exhaustion message, or a decode
/// error for a reply that has no faithful `Value` representation.
pub(super) fn command(handler: &RedisHandler, args: &[Vec<u8>]) -> Result<Value, String> {
    let mut connection = handler.pool().acquire()?;
    match connection.command(args) {
        Err(error) => {
            handler.pool().discard();
            Err(format!("redis: transport failure: {error}"))
        }
        Ok(reply) => {
            // Released before conversion: a `Resp::Error` reply is already fully
            // drained, so the connection is reusable even though the call fails.
            handler.pool().release(connection);
            handler_value::from_resp(reply)
        }
    }
}
