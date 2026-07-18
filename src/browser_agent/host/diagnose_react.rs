//! React and framework diagnostic method views.

use crate::value::Value;

use super::super::super::state::HostState;

#[path = "diagnose_react_selector.rs"]
mod selector;
#[cfg(test)]
#[path = "diagnose_react_selector_tests.rs"]
mod selector_tests;
#[cfg(test)]
#[path = "diagnose_react_tests.rs"]
mod tests;

pub(super) fn invoke(
    state: &mut HostState,
    kind: &str,
    query: Option<&str>,
) -> Result<Value, String> {
    let report = state.page.production_debug_report();
    match kind {
        "react.detect" => Ok(Value::Bool(report.react.detected)),
        "react.version" => version(state),
        "react.component_tree" => Ok(super::value::strings(report.react.roots)),
        "react.errors" => Ok(super::value::strings(
            report
                .console_errors
                .into_iter()
                .filter(|error| error.to_ascii_lowercase().contains("react")),
        )),
        "react.hydration_warnings" => Ok(super::value::strings(report.react.hydration_warnings)),
        "react.suspense_boundaries" => Ok(selector::boundaries(report.visual_elements)),
        kind if selector::handles(kind) => selector::invoke(state, kind, query),
        _ => Err(format!("browser.diagnose: unsupported diagnostic `{kind}`")),
    }
}

fn version(state: &mut HostState) -> Result<Value, String> {
    let source = "window.React && window.React.version ? window.React.version : null";
    Ok(crate::js::js_to_tether(&state.page.eval_js(source)?))
}
