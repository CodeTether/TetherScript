//! Native shared-context storage actions.

use crate::value::Value;

use super::super::state::HostState;

#[path = "storage_clear.rs"]
mod clear;
#[path = "indexed_db_summary.rs"]
mod indexed;
#[path = "storage_read.rs"]
mod read;
#[cfg(test)]
#[path = "storage_tests.rs"]
mod tests;

pub(super) fn invoke(state: &mut HostState, action: &str) -> Result<Value, String> {
    match action {
        "indexed_db_summary" => indexed::invoke(state),
        "cookies" | "local_storage" | "session_storage" => read::invoke(state, action),
        "clear_storage" => Ok(clear::invoke(state)),
        _ => unreachable!(),
    }
}
