//! Native text, HTML, JavaScript, and wait queries.

use crate::browser::{query_selector, text_content};
use crate::value::Value;

use super::state::HostState;

#[path = "network_log.rs"]
mod network_log;

pub(super) fn invoke(
    state: &mut HostState,
    action: &str,
    payload: &Value,
) -> Result<Value, String> {
    match action {
        "text" => text(state, super::value::optional_string(payload, "selector")?),
        "html" => html(state, super::value::optional_string(payload, "selector")?),
        "eval" => eval(state, &super::value::string_field(payload, "expression")?),
        "network_log" => network_log::invoke(state, payload),
        _ => unreachable!(),
    }
}

pub(super) fn text(state: &HostState, selector: Option<String>) -> Result<Value, String> {
    let value = match selector {
        Some(selector) => query_selector(&state.page.session.document, &selector)
            .first()
            .map(text_content)
            .ok_or_else(|| format!("browser.text: selector `{}` not found", selector))?,
        None => super::snapshot::document_text(&state.page),
    };
    Ok(super::value::string(value))
}

fn html(state: &mut HostState, selector: Option<String>) -> Result<Value, String> {
    let Some(selector) = selector else {
        return Ok(super::value::string(state.page.session.html.clone()));
    };
    let quoted = crate::json::encode_to_string(&super::value::string(selector))?;
    eval(
        state,
        &format!("document.querySelector({}).outerHTML", quoted),
    )
}

fn eval(state: &mut HostState, expression: &str) -> Result<Value, String> {
    let value = state.page.eval_js(expression)?;
    Ok(crate::js::js_to_tether(&value))
}
