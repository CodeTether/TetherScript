//! Encoding, sending, and error-promoting a single command.
//!
//! This is the one place where a top-level [`RespValue::Error`] becomes a
//! [`RedisError::Server`]. Doing it here, rather than in the decoder, keeps the
//! two ideas separate: the decoder's job is to say faithfully what arrived, and a
//! nested error inside an array must survive as a value. Only a *reply-level*
//! error is a failed command.

use super::connection::Connection;
use super::encode_command::encode_command;
use super::error::RedisError;
use super::value::RespValue;

impl Connection {
    /// Encode `args`, send it, and return the reply, promoting error replies.
    ///
    /// # Arguments
    ///
    /// * `args` — Command name followed by arguments, as raw bytes. Values are
    ///   never spliced into a command string; see [`encode_command`].
    ///
    /// # Returns
    ///
    /// The decoded reply, guaranteed not to be a top-level [`RespValue::Error`].
    ///
    /// # Errors
    ///
    /// * [`RedisError::Server`] when the reply was `-<kind> <message>`. The
    ///   connection stays usable.
    /// * [`RedisError::Transport`] or [`RedisError::Protocol`] otherwise; after a
    ///   protocol error the connection must be discarded, since its framing
    ///   position is unknown.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tetherscript::redis::{Config, Connection};
    ///
    /// # fn main() -> Result<(), tetherscript::redis::RedisError> {
    /// let mut connection = Connection::connect(&Config::default())?;
    /// // Any command, including ones without a typed helper here.
    /// let reply = connection.command(&[&b"ECHO"[..], &b"hi"[..]])?;
    /// assert_eq!(reply.bulk("ECHO")?, b"hi".as_slice());
    /// # Ok(())
    /// # }
    /// ```
    pub fn command(&mut self, args: &[&[u8]]) -> Result<RespValue, RedisError> {
        let request = encode_command(args)?;
        match self.round_trip(&request)? {
            RespValue::Error { kind, message } => Err(RedisError::Server { kind, message }),
            other => Ok(other),
        }
    }
}
