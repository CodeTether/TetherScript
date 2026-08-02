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

    /// Begin a transaction, pinning one connection until it resolves.
    ///
    /// Every subsequent [`QueryHandler::query`] must run on the pinned connection
    /// until [`QueryHandler::commit`] or [`QueryHandler::rollback`], because a
    /// pooled handler could otherwise send the statements to a different
    /// connection and silently drop them from the transaction.
    ///
    /// # Errors
    ///
    /// Returns an error when a transaction is already open or the adapter does not
    /// support them. The default implementation refuses, so an adapter that has not
    /// opted in cannot appear to honour a transaction it is ignoring.
    fn begin(&self) -> Result<(), String> {
        Err("db.begin: this database adapter does not support transactions".into())
    }

    /// Commit the open transaction.
    ///
    /// # Errors
    ///
    /// Returns an error when no transaction is open or the commit fails.
    fn commit(&self) -> Result<(), String> {
        Err("db.commit: this database adapter does not support transactions".into())
    }

    /// Roll back the open transaction.
    ///
    /// # Errors
    ///
    /// Returns an error when no transaction is open or the rollback fails.
    fn rollback(&self) -> Result<(), String> {
        Err("db.rollback: this database adapter does not support transactions".into())
    }

    /// Number of connections the adapter currently holds, for diagnostics.
    ///
    /// Defaults to 1, which is correct for a single-connection adapter.
    fn pool_size(&self) -> usize {
        1
    }
}
