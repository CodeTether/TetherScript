//! Response collection, shared by both query protocols.
//!
//! Both paths end with `ReadyForQuery`, so the reply is always drained to that
//! point. An `ErrorResponse` is remembered and reported afterwards, which leaves
//! the connection reusable instead of abandoned mid-stream.

use std::cell::RefCell;
use std::rc::Rc;

use super::connection::Connection;
use super::{decode, error, rows};
use crate::value::Value;

/// Read messages until `ReadyForQuery`, returning the decoded rows.
pub(super) fn rows(connection: &mut Connection) -> Result<Value, String> {
    let mut columns: Vec<String> = Vec::new();
    let mut collected: Vec<Value> = Vec::new();
    let mut failure: Option<String> = None;
    loop {
        let message = decode::read(&mut connection.stream)
            .map_err(|error| format!("postgres: read query response: {error}"))?;
        match message.tag {
            // T: RowDescription, D: DataRow, E: ErrorResponse, Z: ReadyForQuery.
            b'T' => columns = rows::row_description(&message.body)?,
            b'D' => collected.push(rows::data_row(&message.body, &columns)?),
            b'E' => failure = Some(error::describe(&message.body)),
            b'Z' => break,
            _ => {}
        }
    }
    match failure {
        Some(message) => Err(message),
        None => Ok(Value::List(Rc::new(RefCell::new(collected)))),
    }
}
