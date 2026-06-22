//! Minimal HTTP URL parser for Vault/provider endpoints.

use super::url_scheme;

#[derive(Clone, PartialEq)]
pub(crate) enum Scheme {
    Http,
    Https,
}

pub(crate) struct Url {
    pub scheme: Scheme,
    pub host: String,
    pub port: u16,
    pub path: String,
}

pub(crate) fn parse(input: &str) -> Result<Url, String> {
    let (scheme, rest) = url_scheme::strip(input)?;
    let (authority, raw_path) = rest.split_once('/').unwrap_or((rest, ""));
    if authority.is_empty() {
        return Err("url: missing host".into());
    }
    let default_port = default_port(&scheme);
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (host, parse_port_text(port)?),
        None => (authority, default_port),
    };
    if host.is_empty() {
        return Err("url: missing host".into());
    }
    Ok(Url {
        scheme,
        host: host.into(),
        port,
        path: format!("/{raw_path}"),
    })
}

fn parse_port_text(raw: &str) -> Result<u16, String> {
    raw.parse()
        .map_err(|_| format!("url: invalid port {raw:?}"))
}

pub(crate) fn default_port(scheme: &Scheme) -> u16 {
    match scheme {
        Scheme::Https => 443,
        Scheme::Http => 80,
    }
}
