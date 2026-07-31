//! Extended query protocol: `Parse`/`Bind`/`Execute`.
//!
//! This is the safe path for untrusted values. The statement is parsed once with
//! `$n` placeholders, then values are bound out-of-band, so a parameter can never
//! change the shape of the SQL.

use super::encode::Builder;
use super::params::Parameter;

/// Build the `Parse` message for an unnamed prepared statement.
///
/// The zero parameter-type count asks the server to infer every type.
pub(super) fn parse(sql: &str) -> Vec<u8> {
    let mut message = Builder::tagged(b'P');
    message
        .cstr("") // unnamed statement
        .cstr(sql)
        .i16(0);
    message.finish()
}

/// Build the `Bind` message that binds `parameters` to the unnamed statement.
pub(super) fn bind(parameters: &[Parameter]) -> Vec<u8> {
    let mut message = Builder::tagged(b'B');
    message
        .cstr("") // unnamed destination portal
        .cstr("") // unnamed source statement
        .i16(0) // all parameters use the default text format
        .i16(parameters.len() as i16);
    for parameter in parameters {
        match parameter {
            // -1 length signals SQL NULL and carries no value bytes.
            None => {
                message.i32(-1);
            }
            Some(bytes) => {
                message.i32(bytes.len() as i32).bytes(bytes);
            }
        }
    }
    // Request text format for all result columns.
    message.i16(0);
    message.finish()
}

/// Build `Describe`, so the reply carries a `RowDescription` with column names.
pub(super) fn describe() -> Vec<u8> {
    let mut message = Builder::tagged(b'D');
    message.bytes(b"P").cstr("");
    message.finish()
}

/// Build `Execute` with no row limit.
pub(super) fn execute() -> Vec<u8> {
    let mut message = Builder::tagged(b'E');
    message.cstr("").i32(0);
    message.finish()
}

/// Build `Sync`, which closes the exchange and triggers `ReadyForQuery`.
pub(super) fn sync() -> Vec<u8> {
    Builder::tagged(b'S').finish()
}
