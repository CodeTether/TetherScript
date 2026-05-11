//! HTTP URL parsing for `http://` scheme (no TLS support).

/// Parsed components of an `http://` URL.
#[derive(Debug, PartialEq)]
pub(crate) struct ParsedHttpUrl {
    pub host: String,
    pub port: u16,
    pub host_header: String,
    pub target: String,
}

impl ParsedHttpUrl {
    /// Parse an `http://` URL into its components.
    ///
    /// # Errors
    ///
    /// Returns a descriptive error for invalid URLs, HTTPS URLs,
    /// missing hosts, bad ports, and other malformed input.
    pub(crate) fn parse(input: &str) -> Result<Self, String> {
        if input.starts_with("https://") {
            return Err(
                "http_request: https:// requires TLS; std-only TetherScript supports http://"
                    .into(),
            );
        }
        let rest = input
            .strip_prefix("http://")
            .ok_or_else(|| "http_request: URL must start with http://".to_string())?;
        if rest.is_empty() {
            return Err("http_request: missing URL authority".into());
        }
        if rest.contains('#') {
            return Err("http_request: URL fragments are not sent over HTTP".into());
        }

        let split_at = rest
            .char_indices()
            .find_map(|(idx, ch)| matches!(ch, '/' | '?').then_some(idx))
            .unwrap_or(rest.len());
        let authority = &rest[..split_at];
        let suffix = &rest[split_at..];
        if authority.is_empty() {
            return Err("http_request: missing URL host".into());
        }
        if authority.contains('@') {
            return Err("http_request: userinfo in URLs is not supported".into());
        }

        let (host, port, host_header) = parse_authority(authority)?;
        let target = if suffix.is_empty() {
            "/".to_string()
        } else if suffix.starts_with('?') {
            format!("/{}", suffix)
        } else {
            suffix.to_string()
        };

        Ok(Self {
            host,
            port,
            host_header,
            target,
        })
    }
}

fn parse_authority(authority: &str) -> Result<(String, u16, String), String> {
    if authority.starts_with('[') {
        return parse_ipv6_authority(authority);
    }
    if authority.matches(':').count() > 1 {
        return Err("http_request: IPv6 hosts must be bracketed".into());
    }
    let (host, port) = match authority.split_once(':') {
        Some((host, port)) => (host.to_string(), parse_port(port)?),
        None => (authority.to_string(), 80),
    };
    if host.is_empty() {
        return Err("http_request: missing URL host".into());
    }
    if host.chars().any(char::is_whitespace) {
        return Err("http_request: URL host must not contain whitespace".into());
    }
    let host_header = if port == 80 {
        host.clone()
    } else {
        format!("{}:{}", host, port)
    };
    Ok((host, port, host_header))
}

fn parse_ipv6_authority(authority: &str) -> Result<(String, u16, String), String> {
    let end = authority
        .find(']')
        .ok_or_else(|| "http_request: invalid bracketed IPv6 host".to_string())?;
    let host = authority[1..end].to_string();
    if host.is_empty() {
        return Err("http_request: empty IPv6 host".into());
    }
    let rest = &authority[end + 1..];
    let port = if rest.is_empty() {
        80
    } else {
        let port = rest
            .strip_prefix(':')
            .ok_or_else(|| "http_request: invalid authority after IPv6 host".to_string())?;
        parse_port(port)?
    };
    let host_header = if port == 80 {
        format!("[{}]", host)
    } else {
        format!("[{}]:{}", host, port)
    };
    Ok((host, port, host_header))
}

fn parse_port(port: &str) -> Result<u16, String> {
    if port.is_empty() {
        return Err("http_request: empty URL port".into());
    }
    let port: u16 = port
        .parse()
        .map_err(|_| format!("http_request: invalid URL port {}", port))?;
    if port == 0 {
        return Err("http_request: URL port must be greater than zero".into());
    }
    Ok(port)
}
