//! Shared helper for the variadic, integer-answering key commands.
//!
//! `DEL` and `EXISTS` differ only in their name, so the arity check and reply
//! extraction live here once.

use super::connection::Connection;
use super::error::RedisError;

impl Connection {
    /// Send `name` with `keys` and read an integer reply.
    ///
    /// # Errors
    ///
    /// [`RedisError::Protocol`] when `keys` is empty; the command's own errors
    /// otherwise.
    pub(super) fn keyed_count(
        &mut self,
        name: &'static str,
        keys: &[&[u8]],
    ) -> Result<i64, RedisError> {
        if keys.is_empty() {
            return Err(RedisError::Protocol(format!(
                "{name}: at least one key is required"
            )));
        }
        let mut args: Vec<&[u8]> = Vec::with_capacity(keys.len() + 1);
        args.push(name.as_bytes());
        args.extend_from_slice(keys);
        self.command(&args)?.integer(name)
    }
}
