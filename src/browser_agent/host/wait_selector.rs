//! Selector-state predicates for native host waits.

use crate::browser_agent::{query, resolve, Locator};
use crate::value::Value;

use super::state::HostState;

pub(super) fn until(
    state: &mut HostState,
    selector: &str,
    desired: &str,
    timeout_ms: u64,
) -> Result<Value, String> {
    if !matches!(desired, "attached" | "detached" | "visible" | "hidden") {
        return Err(format!(
            "browser.wait: unsupported selector state `{desired}`"
        ));
    }
    let locator = Locator::css(selector).relaxed();
    let label = format!("selector `{selector}` state `{desired}`");
    super::wait_poll::until(state, timeout_ms, &label, move |host| {
        let attached = !query::locate(&host.page.session.document, &locator).is_empty();
        let visible =
            resolve::resolve(&host.page.session, host.page.viewport_width, &locator).is_ok();
        Ok(match desired {
            "attached" => attached,
            "detached" => !attached,
            "visible" => visible,
            "hidden" => !attached || !visible,
            _ => unreachable!(),
        })
    })
}
