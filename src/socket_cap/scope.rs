//! Host-and-port scope rules for the `socket` capability.
//!
//! One concern: decide whether a grant permits a given `host:port`. Shared by
//! the TCP and UDP operations so both enforce the same rule.

/// One granted endpoint pattern.
///
/// `host` is matched literally after lowercasing, except that `*` matches any
/// host. `port` of `None` means any port on that host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope {
    host: String,
    port: Option<u16>,
}

impl Scope {
    /// Build a scope from a `host`, `host:port`, or `*` grant string.
    ///
    /// # Errors
    ///
    /// Returns an error naming the argument when the port is not a number in
    /// `0..=65535`, or when the host is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::socket_cap::scope::Scope;
    ///
    /// assert!(Scope::parse("127.0.0.1:8080").is_ok());
    /// assert!(Scope::parse("example.com").is_ok());
    /// assert!(Scope::parse("*").is_ok());
    /// assert!(Scope::parse("host:notaport").is_err());
    /// ```
    pub fn parse(grant: &str) -> Result<Self, String> {
        let (host, port) = split_host_port(grant)?;
        if host.is_empty() {
            return Err(format!("socket grant `{grant}` has an empty host"));
        }
        Ok(Self {
            host: host.to_ascii_lowercase(),
            port,
        })
    }

    /// True when this scope permits `host:port`.
    pub fn permits(&self, host: &str, port: u16) -> bool {
        let host_ok = self.host == "*" || self.host == host.to_ascii_lowercase();
        host_ok && self.port.is_none_or(|granted| granted == port)
    }
}

/// Split `host`, `host:port`, or `*` into its parts.
fn split_host_port(grant: &str) -> Result<(&str, Option<u16>), String> {
    match grant.rsplit_once(':') {
        Some((host, port)) => {
            let parsed = port
                .parse::<u16>()
                .map_err(|_| format!("socket grant `{grant}` has an invalid port `{port}`"))?;
            Ok((host, Some(parsed)))
        }
        None => Ok((grant, None)),
    }
}
