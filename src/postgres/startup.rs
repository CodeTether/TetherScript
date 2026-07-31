//! Startup handshake: StartupMessage, authentication, then `ReadyForQuery`.

use std::io::Write;

use super::connection::{Config, Connection};
use super::encode::Builder;
use super::{auth, decode, error};

/// Send the startup packet and drive authentication to completion.
pub(super) fn run(connection: &mut Connection, config: &Config) -> Result<(), String> {
    let mut startup = Builder::untagged();
    startup
        .i32(196_608) // protocol version 3.0
        .cstr("user")
        .cstr(&config.user)
        .cstr("database")
        .cstr(&config.database)
        .bytes(&[0]);
    connection
        .stream
        .write_all(&startup.finish())
        .map_err(|error| format!("postgres: send startup packet: {error}"))?;
    drain_until_ready(connection, &config.user, &config.password)
}

/// Consume authentication and parameter messages up to `ReadyForQuery`.
fn drain_until_ready(
    connection: &mut Connection,
    user: &str,
    password: &str,
) -> Result<(), String> {
    loop {
        let message = decode::read(&mut connection.stream)
            .map_err(|error| format!("postgres: read startup response: {error}"))?;
        match message.tag {
            b'R' => {
                auth::step(&mut connection.stream, &message.body, user, password)?;
            }
            b'E' => return Err(error::describe(&message.body)),
            b'Z' => return Ok(()),
            // S/K/N: parameter status, cancellation key, notice.
            _ => {}
        }
    }
}
