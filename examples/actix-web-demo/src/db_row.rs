//! Convert SQLx PostgreSQL rows into structured tetherscript values.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use sqlx::postgres::PgRow;
use sqlx::{Column, Row};
use tetherscript::value::Value;

pub fn rows(rows: Vec<PgRow>) -> Result<Value, String> {
    let values = rows.into_iter().map(row).collect::<Result<Vec<_>, _>>()?;
    Ok(Value::List(Rc::new(RefCell::new(values))))
}

fn row(row: PgRow) -> Result<Value, String> {
    let mut values = HashMap::new();
    for (index, column) in row.columns().iter().enumerate() {
        values.insert(
            column.name().into(),
            crate::db_decode::column_value(&row, index)?,
        );
    }
    Ok(Value::Map(Rc::new(RefCell::new(values))))
}
