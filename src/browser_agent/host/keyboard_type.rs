//! Incremental typing against the host's focused locator.

use crate::browser_agent::keyboard::{keyboard_script, KeyboardKey};
use crate::browser_agent::{prepare, retry, ActionReport, Locator};
use crate::browser_session::TraceEvent;
use crate::value::Value;

use super::super::state::HostState;

pub(super) fn invoke(
    state: &mut HostState,
    locator: &Locator,
    payload: &Value,
) -> Result<ActionReport, String> {
    let text = super::super::value::string_field(payload, "text")?;
    let (resolved, checks) = retry::stable(&mut state.page, "keyboard_type", locator, |page| {
        prepare::fill(page, locator)
    })?;
    if text.is_empty() {
        let node = crate::browser_agent::keyboard_escape::node(&resolved.dom.path);
        state.page.eval_js(&format!("let n={node};n.focus();"))?;
    }
    for character in text.chars() {
        let key = KeyboardKey::try_from(character)
            .map_err(|error| format!("browser.keyboard_type: {error}"))?;
        state
            .page
            .eval_js(&keyboard_script::press(&resolved.dom.path, &key, None))?;
    }
    state.page.session.trace.push(TraceEvent::new(
        "keyboard_type",
        format!("{:?}", locator.kind),
    ));
    Ok(ActionReport::new(
        "keyboard_type",
        format!("{:?}", locator.kind),
        resolved.bounds,
        checks,
    ))
}
