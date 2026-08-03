//! Row decoding for the simple-query protocol.
//!
//! `RowDescription` (`T`) names the columns and gives each one's type OID; each `DataRow` (`D`)
//! carries field lengths and text bytes, with -1 meaning SQL NULL.
//!
//! The type OID is carried through rather than discarded. Without it, decoding had to guess from
//! the text alone, which made a `timestamptz` an opaque string: a script asking for `created_at`
//! got `2026-03-03 23:00:45.517441+00` and had no way to compare or format it. Worse, the guess
//! was actively wrong for some values — a `varchar` holding `"0123"` parsed as the integer 123,
//! losing the leading zero, and a numeric-looking product code silently stopped being a string.
//!
//! With the OID in hand, a column declared textual stays text no matter what it contains, and a
//! temporal column becomes Unix seconds, which is what every date built-in in the language takes.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::cursor::Cursor;
use crate::value::Value;

/// One column's name and declared type.
pub(super) struct Column {
    /// Column name as the server reported it.
    pub(super) name: String,
    /// PostgreSQL type OID, which decides how the text is interpreted.
    pub(super) type_oid: u32,
}

/// Column names and type OIDs from a `RowDescription` message.
pub(super) fn row_description(body: &[u8]) -> Result<Vec<Column>, String> {
    let mut cursor = Cursor::new(body);
    let count = cursor.i16()?;
    let mut columns = Vec::with_capacity(count.max(0) as usize);
    for _ in 0..count.max(0) {
        let name = cursor.cstr()?;
        // table OID and column index precede the type OID.
        cursor.take(6)?;
        let type_oid = cursor.i32()? as u32;
        // size, type modifier, and format code follow.
        cursor.take(8)?;
        columns.push(Column { name, type_oid });
    }
    Ok(columns)
}

/// One `DataRow` as a map keyed by column name.
pub(super) fn data_row(body: &[u8], columns: &[Column]) -> Result<Value, String> {
    let mut cursor = Cursor::new(body);
    let count = cursor.i16()?;
    let mut row = HashMap::new();
    for index in 0..count.max(0) as usize {
        let len = cursor.i32()?;
        let column = columns.get(index);
        let name = column
            .map(|column| column.name.clone())
            .unwrap_or_else(|| format!("column{index}"));
        if len < 0 {
            row.insert(name, Value::Nil);
            continue;
        }
        let raw = cursor.take(len as usize)?;
        let text = String::from_utf8_lossy(raw);
        // An unknown column falls back to inference, which is what the decoder did for every
        // column before type OIDs were carried through.
        let oid = column.map(|column| column.type_oid).unwrap_or(0);
        row.insert(name, super::rows_typed::typed(oid, &text));
    }
    Ok(Value::Map(Rc::new(RefCell::new(row))))
}
