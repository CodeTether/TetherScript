//! Defaults and address parsing for [`Config`].
//!
//! Split from the struct definition so the field documentation stays readable.

use std::time::Duration;

use super::config::Config;

/// Redis' registered default port.
pub(super) const DEFAULT_PORT: u16 = 6379;

/// Timeout applied to connect, read, and write by `Config::default`.
pub(super) const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

impl Default for Config {
    /// Localhost, port 6379, no auth, database 0, five-second timeouts.
    ///
    /// Timeouts are set rather than left infinite on purpose: an unset socket
    /// timeout turns an unreachable server into a hung process.
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: DEFAULT_PORT,
            password: None,
            username: None,
            database: 0,
            connect_timeout: DEFAULT_TIMEOUT,
            read_timeout: DEFAULT_TIMEOUT,
            write_timeout: DEFAULT_TIMEOUT,
        }
    }
}

impl Config {
    /// Build a config from a `host:port` string, keeping every other default.
    ///
    /// # Arguments
    ///
    /// * `target` — `host:port`, or a bare host to accept Redis' default port 6379.
    ///
    /// # Returns
    ///
    /// The parsed config.
    ///
    /// # Errors
    ///
    /// Returns a message naming the unparsable input when the port is not a
    /// number, so a typo in configuration is obvious rather than silently
    /// defaulted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::redis::Config;
    ///
    /// let config = Config::from_address("127.0.0.1:6380").unwrap();
    /// assert_eq!(config.port, 6380);
    /// assert_eq!(Config::from_address("localhost").unwrap().port, 6379);
    /// assert!(Config::from_address("localhost:nope").is_err());
    /// ```
    pub fn from_address(target: &str) -> Result<Self, String> {
        let (host, port) = match target.rsplit_once(':') {
            Some((host, port)) => (
                host,
                port.parse::<u16>()
                    .map_err(|_| format!("redis: `{port}` is not a valid TCP port"))?,
            ),
            None => (target, DEFAULT_PORT),
        };
        Ok(Self {
            host: host.to_string(),
            port,
            ..Self::default()
        })
    }
}
