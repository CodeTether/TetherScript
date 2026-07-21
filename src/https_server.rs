//! Blocking HTTPS listener backed by an in-process OpenSSL acceptor.

use std::net::TcpListener;
use std::time::Duration;

use crate::tls::TlsAcceptor;
use crate::value::{Runtime, Value};

use super::{http_server_args as args, http_server_connection as connection};

pub(crate) fn serve(
    runtime: &mut dyn Runtime,
    port: &Value,
    certificates: &Value,
    private_key: &Value,
    handler: &Value,
) -> Result<Value, String> {
    let port = args::port(port, "https_serve")?;
    let certificates = args::pem(certificates, "https_serve", "certificate")?;
    let private_key = args::pem(private_key, "https_serve", "private key")?;
    args::handler(handler, "https_serve")?;
    let acceptor = TlsAcceptor::from_pem(certificates.as_bytes(), private_key.as_bytes())
        .map_err(|error| format!("https_serve: invalid TLS identity: {error}"))?;
    let listener = TcpListener::bind(("0.0.0.0", port))
        .map_err(|error| format!("https_serve: bind to 0.0.0.0:{port} failed: {error}"))?;
    eprintln!(
        "tetherscript https: listening on https://0.0.0.0:{port} (try https://localhost:{port})"
    );
    for incoming in listener.incoming() {
        let tcp = match incoming {
            Ok(tcp) => tcp,
            Err(error) => {
                eprintln!("tetherscript https: accept error: {error}");
                continue;
            }
        };
        let stream = match acceptor.accept(tcp) {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("tetherscript https: TLS handshake failed: {error}");
                continue;
            }
        };
        stream
            .get_ref()
            .set_read_timeout(Some(Duration::from_millis(2)))
            .map_err(|error| format!("https_serve: set keep-alive timeout: {error}"))?;
        if let Err(error) = connection::handle(runtime, handler, stream) {
            eprintln!("tetherscript https: {error}");
        }
    }
    Ok(Value::Nil)
}
