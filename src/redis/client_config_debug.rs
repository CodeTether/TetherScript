//! Defaults and password-safe `Debug` for [`Config`].
//!
//! Split from the struct so the field documentation stays readable, and because
//! redaction is its own concern.
//!
//! # How the password is kept out of logs
//!
//! [`Config`] does **not** derive `Debug`. The implementation below is the only
//! one that exists, and it never touches the secret's contents: it prints the
//! literal `Some(<redacted>)` or `None`. Because the derive is absent, a later
//! `{config:?}` in a log line, a `panic!` payload, or an `unwrap` on a
//! `Result<_, Config>` cannot leak it — there is no code path that formats it.
//! [`Connection`](super::connection::Connection) and [`Pool`](super::pool::Pool)
//! are opaque for the same reason: both hold a `Config`, so a derived `Debug` on
//! either would reintroduce the leak transitively.

use std::fmt;
use std::time::Duration;

use super::config::Config;

/// Timeout applied to connect, read, and write by [`Config::default`].
pub(super) const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

impl Default for Config {
    /// Localhost, port 6379, no auth, database 0, five-second timeouts.
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 6379,
            password: None,
            database: None,
            connect_timeout: DEFAULT_TIMEOUT,
            read_timeout: DEFAULT_TIMEOUT,
            write_timeout: DEFAULT_TIMEOUT,
        }
    }
}

/// Prints every field except the password, which is replaced by a fixed marker.
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("password", &self.password.as_ref().map(|_| "<redacted>"))
            .field("database", &self.database)
            .finish_non_exhaustive()
    }
}
