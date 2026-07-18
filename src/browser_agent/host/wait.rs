//! Native browser wait action dispatch.

use crate::value::Value;

use super::state::HostState;

pub(super) fn invoke(state: &mut HostState, payload: &Value) -> Result<Value, String> {
    let timeout = timeout(payload)?;
    if let Some(selector) = super::value::optional_string(payload, "selector")? {
        let desired =
            super::value::optional_string(payload, "state")?.unwrap_or_else(|| "visible".into());
        return super::wait_selector::until(state, &selector, &desired, timeout);
    }
    if let Some(expected) = super::value::optional_string(payload, "text")? {
        let label = format!("text `{expected}`");
        return super::wait_poll::until(state, timeout, &label, |host| {
            Ok(super::snapshot::document_text(&host.page).contains(&expected))
        });
    }
    if let Some(expected) = super::value::optional_string(payload, "url_contains")? {
        let label = format!("URL containing `{expected}`");
        return super::wait_poll::until(state, timeout, &label, |host| {
            Ok(host.page.session.url.contains(&expected))
        });
    }
    Err("browser.wait: expected selector, text, or url_contains".into())
}

fn timeout(payload: &Value) -> Result<u64, String> {
    let value = super::value::optional_int(payload, "timeout_ms")?.unwrap_or(30_000);
    u64::try_from(value).map_err(|_| "browser.wait: timeout_ms cannot be negative".into())
}
