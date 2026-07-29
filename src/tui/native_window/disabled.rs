//! Native widget fallback for builds without window support.

use crate::value::Value;

pub(crate) fn builtin(_args: &[Value]) -> Result<Value, String> {
    Err("tui_native requires the `native-window` Cargo feature on Windows or Linux".into())
}

pub(crate) fn agent_builtin(_args: &[Value]) -> Result<Value, String> {
    Err("tui_native_agent requires the `native-window` Cargo feature".into())
}
