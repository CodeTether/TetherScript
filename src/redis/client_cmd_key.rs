//! `DEL` and `EXISTS`: the key-space commands that count keys.
//!
//! Both are variadic and both answer with a count, so they share one builder.
//! Neither is an error when a key is absent: `DEL` of a missing key is `0`, which
//! is the whole point of an idempotent delete.

use super::connection::Connection;
use super::error::ClientError;

impl Connection {
    /// `DEL key [key …]`.
    ///
    /// # Arguments
    ///
    /// * `keys` — One or more raw keys.
    ///
    /// # Returns
    ///
    /// How many of `keys` existed and were removed; `0` is normal.
    ///
    /// # Errors
    ///
    /// [`ClientError::Protocol`] when `keys` is empty, since Redis would reject
    /// the arity anyway and a local error costs no round trip. Plus the usual
    /// type and transport errors.
    pub fn del(&mut self, keys: &[&[u8]]) -> Result<i64, ClientError> {
        self.counted("DEL", keys)
    }

    /// `EXISTS key [key …]`.
    ///
    /// # Arguments
    ///
    /// * `keys` — One or more raw keys. Repeats are counted repeatedly, which is
    ///   Redis' own behaviour.
    ///
    /// # Returns
    ///
    /// How many of `keys` exist.
    ///
    /// # Errors
    ///
    /// As [`Connection::del`].
    pub fn exists(&mut self, keys: &[&[u8]]) -> Result<i64, ClientError> {
        self.counted("EXISTS", keys)
    }

    /// Issue a variadic key command expecting an integer count.
    fn counted(&mut self, name: &'static str, keys: &[&[u8]]) -> Result<i64, ClientError> {
        if keys.is_empty() {
            return Err(ClientError::Protocol(format!(
                "{name}: at least one key is required"
            )));
        }
        let mut args: Vec<&[u8]> = Vec::with_capacity(keys.len() + 1);
        args.push(name.as_bytes());
        args.extend_from_slice(keys);
        self.command(&args)?.integer(name)
    }
}
