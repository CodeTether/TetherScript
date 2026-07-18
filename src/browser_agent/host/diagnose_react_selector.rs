//! Selector-scoped React diagnostic views.

use crate::value::Value;

use super::super::super::super::state::HostState;

pub(super) fn handles(kind: &str) -> bool {
    matches!(
        kind,
        "react.component_for_selector"
            | "react.props"
            | "react.state"
            | "react.hooks"
            | "react.owner_stack"
    )
}

pub(super) fn invoke(state: &HostState, kind: &str, query: Option<&str>) -> Result<Value, String> {
    let selector = required(kind, query)?;
    match kind {
        "react.component_for_selector" => super::super::element::summary(&state.page, selector),
        "react.props" => super::super::element::attributes(&state.page, selector),
        "react.state" => metadata(state, selector, "data-react-state"),
        "react.hooks" => metadata(state, selector, "data-react-hooks"),
        "react.owner_stack" => metadata(state, selector, "data-react-owner"),
        _ => unreachable!(),
    }
}

pub(super) fn boundaries(items: Vec<crate::browser_agent::VisualElementEvidence>) -> Value {
    super::super::value::strings(items.into_iter().filter_map(|item| {
        item.selector_candidates
            .into_iter()
            .find(|selector| selector.contains("suspense"))
    }))
}

fn metadata(state: &HostState, selector: &str, name: &str) -> Result<Value, String> {
    super::super::element::metadata(&state.page, selector, name)
}

fn required<'a>(kind: &str, query: Option<&'a str>) -> Result<&'a str, String> {
    query.ok_or_else(|| format!("browser.{kind}: selector argument is required"))
}
