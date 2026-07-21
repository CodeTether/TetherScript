//! Native trust-anchor loading for the OpenSSL client context.

use std::io;

use openssl::x509::{store::X509StoreBuilderRef, X509};

pub(super) fn load(store: &mut X509StoreBuilderRef) -> io::Result<()> {
    let result = rustls_native_certs::load_native_certs();
    let mut loaded = 0usize;
    for certificate in result.certs {
        let certificate = X509::from_der(certificate.as_ref()).map_err(error)?;
        if store.add_cert(certificate).is_ok() {
            loaded += 1;
        }
    }
    if loaded == 0 {
        let details = result
            .errors
            .first()
            .map(ToString::to_string)
            .unwrap_or_else(|| "platform certificate store is empty".into());
        return Err(io::Error::other(format!(
            "no trusted CA certificates could be loaded: {details}"
        )));
    }
    Ok(())
}

fn error(error: openssl::error::ErrorStack) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
