use std::rc::Rc;

use super::QueryHandler;

/// Host-neutral authority exposed to scripts as the `db` capability.
///
/// # Examples
///
/// ```
/// use tetherscript::database::{DatabaseAuthority, QueryHandler};
/// use tetherscript::value::Value;
///
/// struct Handler;
/// impl QueryHandler for Handler {
///     fn query(&self, _sql: &str, _parameters: &[Value]) -> Result<Value, String> {
///         Ok(Value::Nil)
///     }
/// }
/// let _database = DatabaseAuthority::new(Handler);
/// ```
pub struct DatabaseAuthority {
    pub(super) handler: Rc<dyn QueryHandler>,
}

impl DatabaseAuthority {
    /// Create an authority backed by a host database handler.
    ///
    /// # Arguments
    ///
    /// * `handler` — Adapter responsible for parameter binding and execution.
    ///
    /// # Returns
    ///
    /// A framework-independent database authority ready for [`PluginHost::grant`](crate::plugin::PluginHost::grant).
    ///
    /// # Examples
    ///
    /// See the [`DatabaseAuthority`] example.
    pub fn new(handler: impl QueryHandler) -> Self {
        Self {
            handler: Rc::new(handler),
        }
    }
}
