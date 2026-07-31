//! SASL/SCRAM-SHA-256 message exchange over the wire.
//!
//! Split from [`super::auth`] because the exchange spans several round trips:
//! SASLInitialResponse, SASLContinue, then SASLResponse.

use std::io::{Read, Write};

use super::cursor::Cursor;
use super::encode::Builder;
use super::{auth, decode, scram};

/// Drive the client half of the SCRAM exchange to the point of sending the proof.
pub(super) fn exchange<S: Read + Write>(stream: &mut S, password: &str) -> Result<(), String> {
    let client_nonce = scram::client_nonce();
    let client_first_bare = format!("n=,r={client_nonce}");

    // The GS2 header `n,,` precedes the bare message and is counted in the length.
    let payload = format!("n,,{client_first_bare}");
    let mut message = Builder::tagged(b'p');
    message
        .cstr("SCRAM-SHA-256")
        .i32(payload.len() as i32)
        .bytes(payload.as_bytes());
    stream
        .write_all(&message.finish())
        .map_err(|error| format!("postgres: send SASLInitialResponse: {error}"))?;

    let server_first = read_continue(stream)?;
    let parsed = scram::parse_server_first(&server_first)?;
    let final_message = scram::client_final(
        password,
        &client_nonce,
        &parsed,
        &client_first_bare,
        &server_first,
    )?;

    let mut reply = Builder::tagged(b'p');
    reply.bytes(final_message.as_bytes());
    stream
        .write_all(&reply.finish())
        .map_err(|error| format!("postgres: send SASLResponse: {error}"))
}

/// Read the `SASLContinue` challenge and return its server-first payload.
fn read_continue<S: Read>(stream: &mut S) -> Result<String, String> {
    let challenge =
        decode::read(stream).map_err(|error| format!("postgres: read SASLContinue: {error}"))?;
    let mut cursor = Cursor::new(&challenge.body);
    if challenge.tag != b'R' {
        return Err(format!(
            "postgres: expected an authentication message during SASL, got tag `{}`",
            challenge.tag as char
        ));
    }
    if cursor.i32()? != auth::SASL_CONTINUE {
        return Err("postgres: expected SASLContinue after SASLInitialResponse".into());
    }
    Ok(String::from_utf8_lossy(cursor.rest()).into_owned())
}
