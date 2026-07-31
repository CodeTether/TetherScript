//! Row decoding for the simple-query protocol.
//!
//! `RowDescription` (`T`) names the columns; each `DataRow` (`D`) carries field
//! lengths and text bytes, with -1 meaning SQL NULL. Values arrive as text, so
//! numeric strings are converted opportunistically and everything else stays a
//! string, which keeps decoding free of a type-OID table.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::cursor::Cursor;
use crate::value::Value;

/// Column names from a `RowDescription` message.
pub(super) fn row_description(body: &[u8]) -> Result<Vec<String>, String> {
    let mut cursor = Cursor::new(body);
    let count = cursor.i16()?;
    let mut names = Vec::with_capacity(count.max(0) as usize);
    for _ in 0..count.max(0) {
        names.push(cursor.cstr()?);
        // table OID, column index, type OID, size, modifier, format code
        cursor.take(18)?;
    }
    Ok(names)
}

/// One `DataRow` as a map keyed by column name.
pub(super) fn data_row(body: &[u8], columns: &[String]) -> Result<Value, String> {
    let mut cursor = Cursor::new(body);
    let count = cursor.i16()?;
    let mut row = HashMap::new();
    for index in 0..count.max(0) as usize {
        let len = cursor.i32()?;
        let name = columns
            .get(index)
            .cloned()
            .unwrap_or_else(|| format!("column{index}"));
        if len < 0 {
            row.insert(name, Value::Nil);
            continue;
        }
        let raw = cursor.take(len as usize)?;
        row.insert(name, scalar(&String::from_utf8_lossy(raw)));
    }
    Ok(Value::Map(Rc::new(RefCell::new(row))))
}

/// Convert a text-format field to the closest tetherscript scalar.
///
/// Text format gives no type information, so `t`/`f` become booleans and numeric
/// strings become numbers. Callers needing exact SQL types should cast in SQL.
fn scalar(text: &str) -> Value {
    match text {
        "t" => return Value::Bool(true),
        "f" => return Value::Bool(false),
        _ => {}
    }
    if let Ok(int) = text.parse::<i64>() {
        return Value::Int(int);
    }
    if let Ok(float) = text.parse::<f64>() {
        return Value::Float(float);
    }
    Value::Str(Rc::new(text.to_string()))
}
