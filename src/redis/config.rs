//! Connection settings for the Redis client.
//!
//! Nothing here is read from the ambient environment. As in
//! [`crate::postgres::Config`], the host decides where the address and password
//! come from, which keeps the client usable from a capability grant instead of
//! reaching for `REDIS_URL` on its own.

use std::time::Duration;

/// Where to connect, how to authenticate, and how long to wait.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
/// use tetherscript::redis::Config;
///
/// let config = Config {
///     host: "127.0.0.1".into(),
///     port: 6379,
///     password: None,
///     username: None,
///     database: 0,
///     connect_timeout: Duration::from_secs(5),
///     read_timeout: Duration::from_secs(5),
///     write_timeout: Duration::from_secs(5),
/// };
/// assert_eq!(config.port, 6379);
/// ```
///
/// `Clone` is derived so a pool can reconnect from the same settings.
#[derive(Clone)]
pub struct Config {
    /// Hostname or IP address of the server.
    pub host: String,
    /// TCP port; Redis' default is 6379.
    pub port: u16,
    /// Password for `AUTH`. `None` skips authentication entirely, which is
    /// correct for a server without `requirepass`.
    pub password: Option<String>,
    /// Username for ACL-style two-argument `AUTH`. Ignored when `password` is
    /// `None`; when `None` the legacy one-argument form is sent.
    pub username: Option<String>,
    /// Logical database to `SELECT` after authenticating. `0` is the default and
    /// sends no `SELECT`, since selecting 0 is a no-op.
    pub database: u32,
    /// Deadline for the TCP connect itself.
    pub connect_timeout: Duration,
    /// Socket read timeout, so a lost server surfaces as an error rather than a
    /// permanent hang.
    pub read_timeout: Duration,
    /// Socket write timeout, for the same reason.
    pub write_timeout: Duration,
}
