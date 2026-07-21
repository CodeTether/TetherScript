//! Per-connection HTTP lifecycle over any readable and writable transport.

use std::io::{BufReader, Read, Write};

use crate::value::{Runtime, Value};

use super::{http_headers, http_response, http_server_request};

pub(super) fn handle<S: Read + Write>(
    runtime: &mut dyn Runtime,
    handler: &Value,
    stream: S,
) -> Result<(), String> {
    let mut reader = BufReader::new(stream);
    loop {
        let request = match http_server_request::parse(&mut reader) {
            Ok(request) => request,
            Err(error) if error == "empty request" => return Ok(()),
            Err(error) => return parse_error(reader.get_mut(), error),
        };
        let keep_alive = !http_headers::wants_close(&request);
        let response = match runtime.invoke(handler, &[request]) {
            Ok(response) => response,
            Err(error) => return handler_error(reader.get_mut(), error),
        };
        http_response::write_response(reader.get_mut(), &response, keep_alive)?;
        if !keep_alive {
            return Ok(());
        }
    }
}

fn parse_error(stream: &mut impl Write, error: String) -> Result<(), String> {
    let _ = http_response::write_simple(
        stream,
        400,
        "text/plain; charset=utf-8",
        error.as_bytes(),
        false,
    );
    Err(format!("parse: {error}"))
}

fn handler_error(stream: &mut impl Write, error: String) -> Result<(), String> {
    let body = format!("handler error: {error}");
    let _ = http_response::write_simple(
        stream,
        500,
        "text/plain; charset=utf-8",
        body.as_bytes(),
        false,
    );
    Err(format!("handler: {error}"))
}
