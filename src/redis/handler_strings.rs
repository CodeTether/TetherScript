//! String key operations: `GET`, `SET`, `DEL`, `EXISTS`.
//!
//! One concern: reading and writing whole string values. Counters live in
//! [`super::handler_counter`] and lifetimes in [`super::handler_expiry`].

use super::handler::RedisHandler;
use super::handler_command::arg;
use crate::value::Value;

impl RedisHandler {
    /// Read a key.
    ///
    /// # Arguments
    ///
    /// * `key` — Key name.
    ///
    /// # Returns
    ///
    /// [`Value::Str`] for a stored UTF-8 value, or [`Value::Nil`] when the key does
    /// not exist. `Nil` and `Value::Str("")` are distinct: an empty string is a
    /// present key holding zero bytes.
    ///
    /// # Errors
    ///
    /// Returns the server's error (e.g. `WRONGTYPE` for a non-string key), a
    /// transport error, a pool-exhaustion message, or a decode error when the
    /// stored bytes are not valid UTF-8.
    pub fn get(&self, key: &str) -> Result<Value, String> {
        self.command(&[arg("GET"), arg(key)])
    }

    /// Write a key, optionally with a time-to-live.
    ///
    /// # Arguments
    ///
    /// * `key` — Key name.
    /// * `value` — Raw bytes; binary-safe, so NUL and CRLF are preserved.
    /// * `expiry_seconds` — `Some(n)` appends `EX n`; `None` stores without a TTL
    ///   and, per Redis semantics, clears any existing TTL.
    ///
    /// # Returns
    ///
    /// [`Value::Str`] holding `OK`.
    ///
    /// # Errors
    ///
    /// Returns the server's error when `expiry_seconds` is not positive, plus the
    /// usual transport and pool errors.
    pub fn set(
        &self,
        key: &str,
        value: &[u8],
        expiry_seconds: Option<i64>,
    ) -> Result<Value, String> {
        let mut args = vec![arg("SET"), arg(key), value.to_vec()];
        if let Some(seconds) = expiry_seconds {
            args.push(arg("EX"));
            args.push(arg(&seconds.to_string()));
        }
        self.command(&args)
    }

    /// Delete a key.
    ///
    /// # Arguments
    ///
    /// * `key` — Key name.
    ///
    /// # Returns
    ///
    /// [`Value::Int`] holding `1` when the key existed, `0` when it did not.
    ///
    /// # Errors
    ///
    /// Returns transport, pool, or server errors.
    pub fn del(&self, key: &str) -> Result<Value, String> {
        self.command(&[arg("DEL"), arg(key)])
    }

    /// Test whether a key exists.
    ///
    /// # Arguments
    ///
    /// * `key` — Key name.
    ///
    /// # Returns
    ///
    /// [`Value::Int`] holding `1` or `0`. Kept as an integer rather than a bool so
    /// the reply matches Redis and stays consistent with the multi-key form
    /// reachable through [`RedisHandler::command`], which counts matches.
    ///
    /// # Errors
    ///
    /// Returns transport, pool, or server errors.
    pub fn exists(&self, key: &str) -> Result<Value, String> {
        self.command(&[arg("EXISTS"), arg(key)])
    }
}
