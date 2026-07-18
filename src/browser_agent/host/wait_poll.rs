//! Wall-clock polling for native host wait predicates.

use std::time::{Duration, Instant};

use crate::value::Value;

use super::state::HostState;

pub(super) fn until(
    state: &mut HostState,
    timeout_ms: u64,
    label: &str,
    mut predicate: impl FnMut(&HostState) -> Result<bool, String>,
) -> Result<Value, String> {
    let duration = Duration::from_millis(timeout_ms);
    let deadline = Instant::now()
        .checked_add(duration)
        .ok_or_else(|| "browser.wait: timeout_ms is too large".to_string())?;
    loop {
        if predicate(state)? {
            return Ok(Value::Bool(true));
        }
        let now = Instant::now();
        if now >= deadline {
            return Err(format!(
                "browser.wait: {label} timed out after {timeout_ms}ms"
            ));
        }
        state
            .page
            .run_scripts()
            .map_err(|error| format!("browser.wait: failed settling {label}: {error}"))?;
        let remaining = deadline.saturating_duration_since(Instant::now());
        if !remaining.is_zero() {
            std::thread::sleep(remaining.min(Duration::from_millis(5)));
        }
    }
}
