//! Convert width-specific PostgreSQL numbers to tetherscript numeric values.

use sqlx::postgres::PgRow;
use sqlx::Row;
use tetherscript::value::Value;

use crate::db_decode::error;

pub(crate) fn integer16(row: &PgRow, index: usize) -> Result<Value, String> {
    row.try_get::<i16, _>(index)
        .map(|value| Value::Int(value.into()))
        .map_err(error)
}

pub(crate) fn integer32(row: &PgRow, index: usize) -> Result<Value, String> {
    row.try_get::<i32, _>(index)
        .map(|value| Value::Int(value.into()))
        .map_err(error)
}

pub(crate) fn float32(row: &PgRow, index: usize) -> Result<Value, String> {
    row.try_get::<f32, _>(index)
        .map(|value| Value::Float(value.into()))
        .map_err(error)
}
