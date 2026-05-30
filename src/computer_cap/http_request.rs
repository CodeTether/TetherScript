//! HTTP request writer for computer bridge calls.

use std::io::Write;
use std::net::TcpStream;

use super::http_url::BridgeUrl;

const USER_AGENT: &str = "tetherscript-computer-cap/0.1";

pub(crate) fn write_request(
    stream: &mut TcpStream,
    url: &BridgeUrl,
    body: &str,
    origin: Option<&str>,
) -> Result<(), String> {
    let origin = origin
        .map(|o| format!("X-TetherScript-Origin: {}\r\n", o))
        .unwrap_or_default();
    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}:{}\r\nUser-Agent: {}\r\n{}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        url.path,
        url.host,
        url.port,
        USER_AGENT,
        origin,
        body.len(),
        body
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("computer bridge: write request failed: {}", e))
}
