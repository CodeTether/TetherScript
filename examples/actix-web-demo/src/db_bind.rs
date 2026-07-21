//! Bind tetherscript scalar values to a SQLx PostgreSQL query.

use sqlx::postgres::PgArguments;
use sqlx::query::Query;
use sqlx::Postgres;
use tetherscript::value::Value;

type PgQuery<'query> = Query<'query, Postgres, PgArguments>;

pub(crate) fn value<'query>(
    query: PgQuery<'query>,
    value: &Value,
) -> Result<PgQuery<'query>, String> {
    match value {
        Value::Nil => Ok(query.bind(Option::<String>::None)),
        Value::Bool(value) => Ok(query.bind(*value)),
        Value::Int(value) => Ok(query.bind(*value)),
        Value::Float(value) => Ok(query.bind(*value)),
        Value::Str(value) => Ok(query.bind(value.to_string())),
        Value::Bytes(value) => Ok(query.bind(value.borrow().clone())),
        value => Err(format!(
            "db.query: cannot bind parameter of type `{}`",
            value.type_name()
        )),
    }
}
