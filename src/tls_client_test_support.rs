//! Test-only trusted-root and preconnected-stream helpers.

use std::io;
use std::net::TcpStream;

use openssl::x509::X509;

use super::{client, TlsConnector, TlsStream};

impl TlsConnector {
    pub(crate) fn trusting(certificate_pem: &[u8]) -> io::Result<Self> {
        let mut builder = client::builder()?;
        let certificate = X509::from_pem(certificate_pem)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        builder
            .cert_store_mut()
            .add_cert(certificate)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        Ok(Self(builder.build()))
    }

    pub(crate) fn connect_tcp(&self, domain: &str, tcp: TcpStream) -> io::Result<TlsStream> {
        self.0
            .connect(domain, tcp)
            .map_err(|error| io::Error::other(error.to_string()))
    }
}
