//! Database capability method dispatch.

use std::rc::Rc;

use tetherscript::value::Value;

use crate::database;
use crate::db_pool::DbPool;

pub fn invoke(pool: &DbPool, method: &str, arguments: &[Value]) -> Result<Value, String> {
    match method {
        "country" => country(pool, arguments),
        _ => Err(format!("db: unsupported method `{method}`")),
    }
}

fn country(pool: &DbPool, arguments: &[Value]) -> Result<Value, String> {
    let Some(Value::Str(code)) = arguments.first() else {
        return Err("db.country: expected a country-code string".into());
    };
    let record = database::country(pool, code)?
        .ok_or_else(|| format!("db.country: country `{code}` not found"))?;
    let json = serde_json::to_string(&record).map_err(|error| error.to_string())?;
    Ok(Value::Str(Rc::new(json)))
}
