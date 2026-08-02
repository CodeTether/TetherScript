use crate::value::Value;

use super::QueryHandler;

/// Dispatch a `db` method that takes no arguments.
///
/// Transaction control is exposed as three separate methods rather than a callback
/// so a script can decide to roll back partway through, based on a query result.
///
/// # Errors
///
/// Returns an error when arguments are supplied, or when the handler rejects the
/// operation.
pub(super) fn call_unit(
    handler: &dyn QueryHandler,
    method: &str,
    arguments: &[Value],
) -> Result<Value, String> {
    if !arguments.is_empty() {
        return Err(format!("db.{method}: takes no arguments"));
    }
    match method {
        "begin" => handler.begin().map(|()| Value::Nil),
        "commit" => handler.commit().map(|()| Value::Nil),
        "rollback" => handler.rollback().map(|()| Value::Nil),
        "pool_size" => Ok(Value::Int(handler.pool_size() as i64)),
        other => Err(format!("db: unsupported method `{other}`")),
    }
}
