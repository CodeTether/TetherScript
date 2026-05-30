//! HTTP bridge URL parser for computer capability transports.

pub(crate) struct BridgeUrl {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) path: String,
}

impl BridgeUrl {
    pub(crate) fn parse(url: &str) -> Result<Self, String> {
        let rest = url
            .strip_prefix("http://")
            .ok_or("computer bridge endpoint must start with http://")?;
        let (host_port, path) = rest.split_once('/').unwrap_or((rest, "computer"));
        let (host, port) = match host_port.rsplit_once(':') {
            Some((host, port)) => (host.to_string(), parse_port(port)?),
            None => (host_port.to_string(), 80),
        };
        Ok(Self {
            host,
            port,
            path: format!("/{}", path),
        })
    }
}

fn parse_port(port: &str) -> Result<u16, String> {
    port.parse()
        .map_err(|_| "computer bridge endpoint port is invalid".into())
}
