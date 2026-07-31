//! Query execution for both the simple and extended protocols.

use std::io::Write;

use super::connection::Connection;
use super::encode::Builder;
use super::{collect, extended, params};
use crate::value::Value;

/// Send `Query` and gather rows until `ReadyForQuery`.
pub(super) fn run(connection: &mut Connection, sql: &str) -> Result<Value, String> {
    let mut query = Builder::tagged(b'Q');
    query.cstr(sql);
    connection
        .stream
        .write_all(&query.finish())
        .map_err(|error| format!("postgres: send query: {error}"))?;
    collect::rows(connection)
}

/// Run `sql` with bound `parameters` through the extended query protocol.
///
/// Placeholders are `$1`, `$2`, and so on. Values travel separately from the
/// statement text, so they cannot alter the parsed SQL.
pub(super) fn run_params(
    connection: &mut Connection,
    sql: &str,
    parameters: &[Value],
) -> Result<Value, String> {
    let encoded = params::encode_all(parameters)?;
    let mut batch = extended::parse(sql);
    batch.extend_from_slice(&extended::bind(&encoded));
    batch.extend_from_slice(&extended::describe());
    batch.extend_from_slice(&extended::execute());
    // Sync must ride along, or the server waits and the read below blocks.
    batch.extend_from_slice(&extended::sync());
    connection
        .stream
        .write_all(&batch)
        .map_err(|error| format!("postgres: send parameterized query: {error}"))?;
    collect::rows(connection)
}
