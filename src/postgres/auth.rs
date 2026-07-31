//! Authentication request dispatch for a PostgreSQL connection.
//!
//! Handles the `Authentication*` backend messages: trust (`Ok`), cleartext
//! `password`, `md5`, and SASL/SCRAM-SHA-256. Unsupported mechanisms fail with a
//! message naming the method rather than hanging the connection.

use std::io::{Read, Write};

use super::cursor::Cursor;
use super::encode::Builder;
use super::{md5_password, sasl};

/// Authentication request codes carried in the `R` message.
pub(super) const OK: i32 = 0;
pub(super) const CLEARTEXT: i32 = 3;
pub(super) const MD5: i32 = 5;
pub(super) const SASL: i32 = 10;
pub(super) const SASL_CONTINUE: i32 = 11;
pub(super) const SASL_FINAL: i32 = 12;

/// Respond to one `R` message. Returns true once authentication is complete.
pub(super) fn step<S: Read + Write>(
    stream: &mut S,
    body: &[u8],
    user: &str,
    password: &str,
) -> Result<bool, String> {
    let mut cursor = Cursor::new(body);
    match cursor.i32()? {
        OK => Ok(true),
        CLEARTEXT => {
            send_password(stream, password)?;
            Ok(false)
        }
        MD5 => {
            let salt = cursor.take(4)?;
            send_password(
                stream,
                &md5_password::postgres_password(user, password, salt),
            )?;
            Ok(false)
        }
        SASL => {
            sasl::exchange(stream, password)?;
            Ok(false)
        }
        // `sasl::exchange` consumes the SASLContinue challenge itself, so the
        // only in-band message left here is SASLFinal, which carries the server
        // signature and is followed by AuthenticationOk.
        SASL_FINAL => Ok(false),
        SASL_CONTINUE => Err("postgres: unexpected SASLContinue outside the SASL exchange".into()),
        other => Err(format!(
            "postgres: unsupported authentication method (code {other})"
        )),
    }
}

pub(super) fn send_password<S: Write>(stream: &mut S, secret: &str) -> Result<(), String> {
    let mut message = Builder::tagged(b'p');
    message.cstr(secret);
    stream
        .write_all(&message.finish())
        .map_err(|error| format!("postgres: send password: {error}"))
}
