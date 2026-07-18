//! Native console, production-debug, and framework diagnostics.

use crate::value::Value;

use super::super::state::HostState;

#[path = "diagnose_console.rs"]
mod console;
#[path = "diagnose_element.rs"]
mod element;
#[path = "diagnose_har.rs"]
mod har;
#[path = "diagnose_react.rs"]
mod react;
#[path = "diagnose_report.rs"]
mod report;
#[cfg(test)]
#[path = "diagnose_tests.rs"]
mod tests;
#[path = "diagnose_value.rs"]
mod value;

pub(super) fn invoke(state: &mut HostState, payload: &Value) -> Result<Value, String> {
    let kind =
        super::super::value::optional_string(payload, "kind")?.unwrap_or_else(|| "diagnose".into());
    let query = super::super::value::optional_string(payload, "query")?;
    match kind.as_str() {
        "console_logs" => Ok(console::logs(&state.page)),
        "console_errors" => Ok(console::errors(&state.page)),
        "unhandled_rejections" => Ok(console::rejections(&state.page)),
        "runtime_exceptions" => Ok(report::exceptions(&state.page)),
        "source_mapped_stack_traces" => Ok(report::stacks(&state.page)),
        "network_har" => Ok(har::value(&state.page)),
        "frameworks" => Ok(value::strings(
            state.page.production_debug_report().frameworks,
        )),
        "diagnose" => Ok(report::summary(&state.page)),
        kind if kind.starts_with("react.") => react::invoke(state, kind, query.as_deref()),
        _ => Err(format!("browser.diagnose: unsupported diagnostic `{kind}`")),
    }
}
