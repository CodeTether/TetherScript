//! Scope and action policy checks for computer capabilities.

use super::authority::ComputerAuthority;

pub(crate) fn all_scopes() -> Vec<String> {
    [
        "computer.snapshot",
        "computer.window_snapshot",
        "computer.click",
        "computer.type",
        "computer.key",
        "computer.scroll",
        "computer.apps",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

impl ComputerAuthority {
    pub(crate) fn require_scope(&self, scope: &str) -> Result<(), String> {
        self.allowed_scopes
            .contains(scope)
            .then_some(())
            .ok_or_else(|| format!("computer: scope `{}` not granted", scope))
    }
}
