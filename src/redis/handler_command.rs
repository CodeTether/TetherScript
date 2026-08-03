//! The general `command` escape hatch and argument encoding.
//!
//! Redis has hundreds of commands and this adapter wraps a handful. Rather than
//! grow a wrapper per command, everything else is reachable through
//! [`RedisHandler::command`], which sends an argument list verbatim. The typed
//! helpers in the sibling modules are thin conveniences over it.
//!
//! Arguments are carried as `Vec<u8>` all the way to the socket so a value
//! containing NUL or CRLF round-trips byte-for-byte: RESP is length-prefixed, so
//! only a client that assumes text can corrupt such a payload.

use super::handler::RedisHandler;
use super::handler_exec;
use crate::value::Value;

impl RedisHandler {
    /// Send an arbitrary command and return its reply.
    ///
    /// # Arguments
    ///
    /// * `args` — Command name first, then its arguments, each as raw bytes. An
    ///   empty list is refused rather than sent, because Redis would answer an
    ///   unhelpful protocol error.
    ///
    /// # Returns
    ///
    /// The reply mapped to a [`Value`]: bulk strings become `Value::Str`, a null
    /// bulk becomes `Value::Nil`, integers become `Value::Int`, arrays become
    /// `Value::List`.
    ///
    /// # Errors
    ///
    /// Returns a transport error (the connection is discarded), the server's error
    /// reply (the connection is released and stays usable), or a pool-exhaustion
    /// message naming the limit.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use tetherscript::redis::{Config, RedisHandler};
    /// # fn main() -> Result<(), String> {
    /// # let handler = RedisHandler::connect(&Config::default())?;
    /// let pong = handler.command(&[b"PING".to_vec()])?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn command(&self, args: &[Vec<u8>]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("redis.command: needs at least a command name".into());
        }
        handler_exec::command(self, args)
    }
}

/// Encode a textual argument for the wire.
pub(super) fn arg(text: &str) -> Vec<u8> {
    text.as_bytes().to_vec()
}
