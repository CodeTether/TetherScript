use crate::value::Value;

use super::QueryHandler;

pub(super) fn call(
    handler: &dyn QueryHandler,
    arguments: &[Value],
) -> Result<Value, String> {
    let [Value::Str(sql), Value::List(parameters)] = arguments else {
        return Err("db.query: expected a SQL string and parameter list".into());
    };
    handler.query(sql, &parameters.borrow())
}
