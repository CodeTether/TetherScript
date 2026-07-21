//! Blocking plain-HTTP listener for tetherscript handlers.

use std::net::TcpListener;
use std::time::Duration;

use crate::value::{Runtime, Value};

use super::{http_server_args as args, http_server_connection as connection};

/// Bind to `0.0.0.0:port` and serve HTTP requests via a TetherScript handler.
///
/// Blocks the calling thread indefinitely. Each incoming connection is
/// handled synchronously with keep-alive support.
pub fn serve(rt: &mut dyn Runtime, port: &Value, handler: &Value) -> Result<Value, String> {
    let port = args::port(port, "http_serve")?;
    args::handler(handler, "http_serve")?;
    let listener = TcpListener::bind(("0.0.0.0", port))
        .map_err(|e| format!("http_serve: bind to 0.0.0.0:{} failed: {}", port, e))?;
    eprintln!(
        "tetherscript http: listening on http://0.0.0.0:{} (try http://localhost:{})",
        port, port
    );
    for conn in listener.incoming() {
        let stream = match conn {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("tetherscript http: accept error: {}", e);
                continue;
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_millis(2)))
            .map_err(|e| format!("http_serve: set keep-alive timeout: {e}"))?;
        if let Err(e) = connection::handle(rt, handler, stream) {
            eprintln!("tetherscript http: {}", e);
        }
    }
    Ok(Value::Nil)
}
