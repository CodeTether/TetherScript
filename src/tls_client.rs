//! Verified OpenSSL client connector construction.

use std::io;
use std::net::TcpStream;
use std::time::Duration;

use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode, SslVersion};

use super::TlsStream;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);

/// Builds verified TLS streams using native platform trust anchors.
///
/// # Examples
///
/// ```no_run
/// use tetherscript::tls::TlsConnector;
///
/// let connector = TlsConnector::new()?;
/// let stream = connector.connect("example.com", 443)?;
/// # drop(stream);
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct TlsConnector(pub(super) SslConnector);

impl TlsConnector {
    /// Create a connector that verifies certificate chains and hostnames.
    ///
    /// # Errors
    ///
    /// Returns an error if OpenSSL cannot initialize or the native trust store
    /// contains no usable CA certificates.
    pub fn new() -> io::Result<Self> {
        let mut builder = builder()?;
        super::roots::load(builder.cert_store_mut())?;
        Ok(Self(builder.build()))
    }

    /// Connect to `domain:port` with a two-minute I/O timeout.
    ///
    /// # Errors
    ///
    /// Returns an error for DNS, TCP, TLS negotiation, certificate-chain, or
    /// hostname-validation failures.
    pub fn connect(&self, domain: &str, port: u16) -> io::Result<TlsStream> {
        self.connect_with_timeout(domain, port, DEFAULT_TIMEOUT)
    }

    /// Connect to `domain:port`, verify its identity, and apply `timeout`.
    ///
    /// # Errors
    ///
    /// Returns an error for DNS, TCP, timeout configuration, TLS negotiation,
    /// certificate-chain, or hostname-validation failures.
    pub fn connect_with_timeout(
        &self,
        domain: &str,
        port: u16,
        timeout: Duration,
    ) -> io::Result<TlsStream> {
        let tcp = TcpStream::connect((domain, port))?;
        tcp.set_read_timeout(Some(timeout))?;
        tcp.set_write_timeout(Some(timeout))?;
        self.0.connect(domain, tcp).map_err(error)
    }
}

pub(super) fn builder() -> io::Result<openssl::ssl::SslConnectorBuilder> {
    let mut builder = SslConnector::builder(SslMethod::tls_client()).map_err(error)?;
    builder.set_verify(SslVerifyMode::PEER);
    builder
        .set_min_proto_version(Some(SslVersion::TLS1_2))
        .map_err(error)?;
    Ok(builder)
}

fn error(error: impl std::fmt::Display) -> io::Error {
    io::Error::other(error.to_string())
}
