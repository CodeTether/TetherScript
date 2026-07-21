use crate::value::Value;

/// Executes SQL for a host-granted [`DatabaseAuthority`](super::DatabaseAuthority).
///
/// Implementors adapt any database client, such as SQLx or a synchronous
/// repository, without introducing that client into tetherscript itself.
///
/// # Examples
///
/// ```
/// use tetherscript::database::QueryHandler;
/// use tetherscript::value::Value;
///
/// struct Noop;
/// impl QueryHandler for Noop {
///     fn query(&self, _sql: &str, _parameters: &[Value]) -> Result<Value, String> {
///         Ok(Value::Nil)
///     }
/// }
/// ```
pub trait QueryHandler: 'static {
    /// Execute parameterized SQL and return script-facing rows.
    ///
    /// # Arguments
    ///
    /// * `sql` — SQL text owned by the calling script.
    /// * `parameters` — separately supplied tetherscript parameter values.
    ///
    /// # Returns
    ///
    /// A tetherscript value, normally a list of row maps.
    ///
    /// # Errors
    ///
    /// Returns a database-qualified message when binding, execution, or row
    /// decoding fails.
    fn query(&self, sql: &str, parameters: &[Value]) -> Result<Value, String>;
}
