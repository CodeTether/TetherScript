//! Minimal blocking HTTP GET for Vault.

use std::io::{Read, Write};
use std::net::TcpStream;

use crate::tls::TlsConnector;

use super::url::{self, Scheme, Url};
use super::{request, response};

trait NetStream: Read + Write {}
impl NetStream for TcpStream {}
#[cfg(feature = "openssl-tls")]
impl NetStream for crate::tls::TlsStream {}

pub(super) fn get(target: &str, token: &str) -> Result<String, String> {
    let parsed = url::parse(target)?;
    let mut stream = connect(&parsed)?;
    request::write(&mut *stream, &parsed, token)?;
    response::read(&mut *stream)
}

fn connect(url: &Url) -> Result<Box<dyn NetStream>, String> {
    match url.scheme {
        Scheme::Http => {
            let stream = TcpStream::connect((url.host.as_str(), url.port))
                .map_err(|error| format!("vault: connect failed: {error}"))?;
            Ok(Box::new(stream))
        }
        Scheme::Https => {
            let connector = TlsConnector::new()
                .map_err(|error| format!("vault: TLS connector failed: {error}"))?;
            let stream = connector
                .connect(&url.host, url.port)
                .map_err(|error| format!("vault: TLS handshake failed: {error}"))?;
            Ok(Box::new(stream))
        }
    }
}
