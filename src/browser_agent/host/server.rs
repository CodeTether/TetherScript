//! Blocking single-session HTTP server for the native browser host.

use std::net::TcpListener;

use super::state::HostState;

pub(super) fn serve(address: &str) -> Result<(), String> {
    let listener = TcpListener::bind(address)
        .map_err(|error| format!("browser host: bind {} failed: {}", address, error))?;
    let mut state = HostState::new();
    for connection in listener.incoming() {
        let mut stream =
            connection.map_err(|error| format!("browser host: accept failed: {}", error))?;
        let request = match super::request::read(&mut stream) {
            Ok(request) if request.path == "/browser" => request,
            Ok(_) => {
                super::response::write(&mut stream, Err("browser host: unknown path".into()))?;
                continue;
            }
            Err(error) => {
                super::response::write(&mut stream, Err(error))?;
                continue;
            }
        };
        let action = crate::json::parse_str(&request.body)
            .map_err(|error| format!("browser host: invalid JSON: {}", error));
        let (result, stop) = match action {
            Ok(action) => super::dispatch::invoke(&mut state, &action),
            Err(error) => (Err(error), false),
        };
        super::response::write(&mut stream, result)?;
        if stop {
            return Ok(());
        }
    }
    Ok(())
}
