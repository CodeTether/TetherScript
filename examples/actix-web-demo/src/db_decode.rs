//! Decode one PostgreSQL column into a tetherscript scalar value.

use std::cell::RefCell;
use std::rc::Rc;

use sqlx::postgres::PgRow;
use sqlx::{Column, Row, TypeInfo, ValueRef};
use tetherscript::value::Value;

pub(crate) fn column_value(row: &PgRow, index: usize) -> Result<Value, String> {
    let column = &row.columns()[index];
    if row.try_get_raw(index).map_err(error)?.is_null() {
        return Ok(Value::Nil);
    }
    match column.type_info().name() {
        "BOOL" => row.try_get(index).map(Value::Bool).map_err(error),
        "INT2" => crate::db_decode_numeric::integer16(row, index),
        "INT4" => crate::db_decode_numeric::integer32(row, index),
        "INT8" => row.try_get(index).map(Value::Int).map_err(error),
        "FLOAT4" => crate::db_decode_numeric::float32(row, index),
        "FLOAT8" => row.try_get(index).map(Value::Float).map_err(error),
        "TEXT" | "VARCHAR" | "BPCHAR" | "NAME" => row
            .try_get::<String, _>(index)
            .map(|v| Value::Str(Rc::new(v)))
            .map_err(error),
        "BYTEA" => row
            .try_get::<Vec<u8>, _>(index)
            .map(|v| Value::Bytes(Rc::new(RefCell::new(v))))
            .map_err(error),
        kind => Err(format!(
            "db.query: column `{}` has unsupported PostgreSQL type `{kind}`",
            column.name()
        )),
    }
}

pub(crate) fn error(error: sqlx::Error) -> String {
    format!("db.query: row decoding failed: {error}")
}
