//! Execute parameterized SQLx queries for the database capability.

use tetherscript::value::Value;

use crate::db_pool::DbPool;

pub(crate) async fn query(pool: &DbPool, arguments: &[Value]) -> Result<Value, String> {
    let (sql, parameters) = parse_arguments(arguments)?;
    let mut query = sqlx::query(&sql);
    for parameter in &parameters {
        query = crate::db_bind::value(query, parameter)?;
    }
    let rows = query
        .fetch_all(pool)
        .await
        .map_err(|error| format!("db.query: SQL execution failed: {error}"))?;
    crate::db_row::rows(rows)
}

fn parse_arguments(arguments: &[Value]) -> Result<(String, Vec<Value>), String> {
    let [Value::Str(sql), Value::List(parameters)] = arguments else {
        return Err("db.query: expected a SQL string and parameter list".into());
    };
    Ok((sql.to_string(), parameters.borrow().clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_requires_sql_and_parameter_list() {
        let error = parse_arguments(&[Value::Int(1)]).unwrap_err();
        assert_eq!(error, "db.query: expected a SQL string and parameter list");
    }
}
