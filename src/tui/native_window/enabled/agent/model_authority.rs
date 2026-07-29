//! Capability validation for native agent workflows.

use crate::value::Value;

pub(crate) fn require(value: &Value, kind: &str) -> Result<(), String> {
    match value {
        Value::Capability(cap) if cap.kind == kind => Ok(()),
        _ => Err(format!("tui_native_agent: missing capability: {kind}")),
    }
}
