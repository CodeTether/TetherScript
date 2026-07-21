//! Dependency-free diagnostics for builds without OpenSSL TLS support.

use std::io;
use std::net::TcpStream;

const ERROR: &str = "TLS support requires the `openssl-tls` feature";

/// Connector placeholder used by dependency-free builds.
pub struct TlsConnector;

impl TlsConnector {
    /// Return an error naming the required opt-in feature.
    pub fn new() -> io::Result<Self> {
        Err(io::Error::new(io::ErrorKind::Unsupported, ERROR))
    }

    /// Return an error because TLS was not compiled into this binary.
    pub fn connect(&self, _domain: &str, _port: u16) -> io::Result<TcpStream> {
        Err(io::Error::new(io::ErrorKind::Unsupported, ERROR))
    }

    pub(crate) fn connect_with_timeout(
        &self,
        _domain: &str,
        _port: u16,
        _timeout: std::time::Duration,
    ) -> io::Result<TcpStream> {
        Err(io::Error::new(io::ErrorKind::Unsupported, ERROR))
    }
}
