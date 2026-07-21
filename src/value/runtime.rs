//! Shared host-callback interface for both execution engines.

use super::Value;

/// Allows native functions and cooperative tasks to invoke language callables.
pub trait Runtime {
    /// Synchronously call `callee` with `args` and return its result.
    ///
    /// # Errors
    /// Returns language errors and panics raised by the callable.
    fn invoke(&mut self, callee: &Value, args: &[Value]) -> Result<Value, String>;

    /// Execute the body of a callable already selected by the scheduler.
    ///
    /// # Errors
    /// Returns language errors and panics raised by the task body.
    fn invoke_scheduled(&mut self, callee: &Value, args: &[Value]) -> Result<Value, String> {
        self.invoke(callee, args)
    }

    /// Return true when a global binding exists in this runtime.
    fn global_defined(&self, _name: &str) -> bool {
        false
    }
}
