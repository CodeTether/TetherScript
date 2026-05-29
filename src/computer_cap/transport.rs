//! Blocking HTTP transport for CodeTether computer-use bridges.

use std::net::TcpStream;
use std::time::Duration;

#[path = "http_request.rs"]
mod http_request;
#[path = "http_response.rs"]
mod http_response;
#[path = "http_url.rs"]
mod http_url;

pub(crate) fn post_json(
    url: &str,
    body: &str,
    timeout: Duration,
    origin: Option<&str>,
) -> Result<String, String> {
    let parsed = http_url::BridgeUrl::parse(url)?;
    let mut stream = connect(&parsed, timeout)?;
    http_request::write_request(&mut stream, &parsed, body, origin)?;
    http_response::read_response(stream)
}

fn connect(parsed: &http_url::BridgeUrl, timeout: Duration) -> Result<TcpStream, String> {
    let stream = TcpStream::connect((parsed.host.as_str(), parsed.port)).map_err(|e| {
        format!(
            "computer bridge: connect to {}:{} failed: {}",
            parsed.host, parsed.port, e
        )
    })?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| format!("computer bridge: set read timeout failed: {}", e))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|e| format!("computer bridge: set write timeout failed: {}", e))?;
    Ok(stream)
}
