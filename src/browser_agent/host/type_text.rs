//! Incremental keyboard typing for selector-backed host actions.

use crate::browser_agent::keyboard::{keyboard_script, KeyboardKey};
use crate::browser_agent::{prepare, retry, ActionReport, Locator};
use crate::browser_session::TraceEvent;
use crate::value::Value;

use super::super::state::HostState;

pub(super) fn invoke(state: &mut HostState, payload: &Value) -> Result<ActionReport, String> {
    let locator = Locator::css(super::super::value::string_field(payload, "selector")?);
    let text = super::super::value::string_field(payload, "text")?;
    let (resolved, checks) = retry::stable(&mut state.page, "type", &locator, |page| {
        prepare::fill(page, &locator)
    })?;
    if text.is_empty() {
        let node = crate::browser_agent::keyboard_escape::node(&resolved.dom.path);
        state.page.eval_js(&format!("let n={node};n.focus();"))?;
    }
    for character in text.chars() {
        let key =
            KeyboardKey::try_from(character).map_err(|error| format!("browser.type: {error}"))?;
        state
            .page
            .eval_js(&keyboard_script::press(&resolved.dom.path, &key, None))?;
    }
    state
        .page
        .session
        .trace
        .push(TraceEvent::new("type", format!("{:?}", locator.kind)));
    Ok(ActionReport::new(
        "type",
        format!("{:?}", locator.kind),
        resolved.bounds,
        checks,
    ))
}
