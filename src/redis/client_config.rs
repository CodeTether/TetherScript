//! Connection settings for the Redis client.
//!
//! Nothing is read from the ambient environment, mirroring
//! [`crate::postgres::Config`]: the host decides where the address and the
//! password come from, which keeps this type usable from a capability grant
//! rather than reaching for `REDIS_URL` on its own.
//!
//! The password is deliberately *not* covered by a derived `Debug`; see
//! `client_config_debug.rs` for the redacting implementation and why it exists.

use std::time::Duration;

/// Where to connect, how to authenticate, and how long to wait.
///
/// `Clone` is derived so a pool can reconnect from the same settings. `Debug` is
/// hand-written and redacts [`Config::password`].
///
/// # Examples
///
/// ```rust,ignore
/// use tetherscript::redis::client::Config;
///
/// let config = Config::new("127.0.0.1", 6379);
/// assert_eq!(config.database, 0);
/// assert!(config.password.is_none());
/// ```
#[derive(Clone)]
pub struct Config {
    /// Hostname or IP address of the server.
    pub host: String,
    /// TCP port; Redis' registered default is 6379.
    pub port: u16,
    /// Password for `AUTH`. `None` sends no `AUTH` at all, which is required
    /// against a server without `requirepass`: an unconditional `AUTH` there is
    /// an error reply, not a no-op.
    pub password: Option<String>,
    /// Logical database to `SELECT` after authenticating. `None` and `Some(0)`
    /// both send no `SELECT`, since every connection already starts on 0.
    pub database: Option<u32>,
    /// Deadline for the TCP connect itself.
    pub connect_timeout: Duration,
    /// Socket read timeout. Set, never left infinite: a server that accepts a
    /// connection and never answers must fail the request, not wedge the process.
    pub read_timeout: Duration,
    /// Socket write timeout, for the same reason on a full send buffer.
    pub write_timeout: Duration,
}

impl Config {
    /// Settings for `host:port` with no auth, database 0, and default timeouts.
    ///
    /// # Arguments
    ///
    /// * `host` — Hostname or IP address.
    /// * `port` — TCP port.
    ///
    /// # Returns
    ///
    /// A config whose remaining fields come from [`Config::default`].
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            ..Self::default()
        }
    }
}
