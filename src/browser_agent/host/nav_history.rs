//! Native host history traversal and page-runtime restoration.

use crate::browser_agent::{NavigationKind, NavigationStatus};
use crate::value::Value;

use super::super::state::HostState;

#[cfg(test)]
#[path = "nav_history_tests.rs"]
mod tests;

pub(super) fn navigate(state: &mut HostState, action: &str) -> Result<Value, String> {
    state.focused = None;
    let result = if action == "back" {
        state.page.go_back()
    } else {
        state.page.go_forward()
    };
    if result.status == NavigationStatus::Committed && result.kind != NavigationKind::SameDocument {
        state.page.run_scripts()?;
    }
    Ok(super::super::snapshot::value(&state.page))
}
