//! Native scroll action envelope.

use crate::browser_session::TraceEvent;
use crate::value::Value;

use super::state::HostState;

pub(super) fn invoke(state: &mut HostState, payload: &Value) -> Result<Value, String> {
    let selector = super::value::optional_string(payload, "selector")?;
    let coordinates = coordinates(payload)?;
    let target = super::scroll_target::resolve(&state.page, selector.as_deref(), coordinates)?;
    state
        .page
        .eval_js(&format!("scrollTo({},{})", target.x, target.y))?;
    state.page.session.scroll = target.clone();
    state.page.session.trace.push(TraceEvent::new(
        "scroll",
        format!("{},{}", target.x, target.y),
    ));
    Ok(super::value::map(vec![
        ("x", Value::Int(target.x)),
        ("y", Value::Int(target.y)),
    ]))
}

fn coordinates(payload: &Value) -> Result<Option<(i64, i64)>, String> {
    let x = super::value::optional_int(payload, "x")?;
    let y = super::value::optional_int(payload, "y")?;
    match (x, y) {
        (Some(x), Some(y)) => Ok(Some((x, y))),
        (None, None) => Ok(None),
        _ => Err("browser.scroll: x and y must be provided together".into()),
    }
}
