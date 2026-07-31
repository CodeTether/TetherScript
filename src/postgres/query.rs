//! Simple-query execution and row collection.

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use super::connection::Connection;
use super::encode::Builder;
use super::{decode, error, rows};
use crate::value::Value;

/// Send `Query` and gather rows until `ReadyForQuery`.
pub(super) fn run(connection: &mut Connection, sql: &str) -> Result<Value, String> {
    let mut query = Builder::tagged(b'Q');
    query.cstr(sql);
    connection
        .stream
        .write_all(&query.finish())
        .map_err(|error| format!("postgres: send query: {error}"))?;
    collect(connection)
}

fn collect(connection: &mut Connection) -> Result<Value, String> {
    let mut columns: Vec<String> = Vec::new();
    let mut collected: Vec<Value> = Vec::new();
    let mut failure: Option<String> = None;
    loop {
        let message = decode::read(&mut connection.stream)
            .map_err(|error| format!("postgres: read query response: {error}"))?;
        match message.tag {
            b'T' => columns = rows::row_description(&message.body)?,
            b'D' => collected.push(rows::data_row(&message.body, &columns)?),
            b'E' => failure = Some(error::describe(&message.body)),
            // Surface the error only after ReadyForQuery, so the connection is
            // left reusable rather than abandoned mid-stream.
            b'Z' => break,
            _ => {}
        }
    }
    match failure {
        Some(message) => Err(message),
        None => Ok(Value::List(Rc::new(RefCell::new(collected)))),
    }
}
