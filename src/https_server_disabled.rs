//! Dependency-free HTTPS server diagnostic.

use crate::value::{Runtime, Value};

pub(crate) fn serve(
    _runtime: &mut dyn Runtime,
    _port: &Value,
    _certificates: &Value,
    _private_key: &Value,
    _handler: &Value,
) -> Result<Value, String> {
    Err("https_serve: TLS support requires the `openssl-tls` feature".into())
}
