//! PEM certificate and private-key handling for HTTPS listeners.

use std::io;

use openssl::pkey::{PKey, Private};
use openssl::ssl::{SslAcceptor, SslMethod, SslStream, SslVersion};
use openssl::x509::X509;

pub(crate) struct TlsAcceptor(SslAcceptor);

impl TlsAcceptor {
    pub(crate) fn from_pem(certificates: &[u8], private_key: &[u8]) -> io::Result<Self> {
        let chain = X509::stack_from_pem(certificates).map_err(invalid)?;
        let (leaf, intermediates) = chain
            .split_first()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty certificate PEM"))?;
        let key = PKey::<Private>::private_key_from_pem(private_key).map_err(invalid)?;
        let mut builder =
            SslAcceptor::mozilla_intermediate_v5(SslMethod::tls_server()).map_err(invalid)?;
        builder
            .set_min_proto_version(Some(SslVersion::TLS1_2))
            .map_err(invalid)?;
        builder.set_certificate(leaf).map_err(invalid)?;
        builder.set_private_key(&key).map_err(invalid)?;
        builder.check_private_key().map_err(invalid)?;
        for certificate in intermediates {
            builder
                .add_extra_chain_cert(certificate.to_owned())
                .map_err(invalid)?;
        }
        Ok(Self(builder.build()))
    }

    pub(crate) fn accept(
        &self,
        stream: std::net::TcpStream,
    ) -> io::Result<SslStream<std::net::TcpStream>> {
        self.0
            .accept(stream)
            .map_err(|error| io::Error::other(error.to_string()))
    }
}

fn invalid(error: openssl::error::ErrorStack) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, error)
}
