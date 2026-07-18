//! Wall-clock polling for captured network events.

use crate::browser_session::NetworkEvent;
use crate::value::Value;

use super::super::super::state::HostState;

#[cfg(test)]
#[path = "network_log_wait_tests.rs"]
mod tests;

pub(super) fn until(state: &mut HostState, payload: &Value) -> Result<(), String> {
    let Some(kind) = super::super::super::value::optional_string(payload, "wait_kind")? else {
        return Ok(());
    };
    if !matches!(kind.as_str(), "request" | "response") {
        return Err(format!("browser.network_log: invalid wait_kind `{kind}`"));
    }
    let contains = super::super::super::value::optional_string(payload, "url_contains")?
        .ok_or_else(|| "browser.network_log: network wait requires `url_contains`".to_string())?;
    let timeout =
        super::super::super::value::optional_int(payload, "timeout_ms")?.unwrap_or(30_000);
    let timeout = u64::try_from(timeout)
        .map_err(|_| "browser.network_log: timeout_ms cannot be negative".to_string())?;
    let label = format!("network {kind} containing `{contains}`");
    super::super::super::wait_poll::until(state, timeout, &label, |host| {
        Ok(host
            .page
            .network_events()
            .iter()
            .any(|event| matches(event, &contains, &kind)))
    })?;
    Ok(())
}

fn matches(event: &NetworkEvent, contains: &str, kind: &str) -> bool {
    event.url.contains(contains) && (kind == "request" || event.status.is_some())
}
