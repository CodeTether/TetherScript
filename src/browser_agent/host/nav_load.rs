//! Native document fetch, load, and script execution.

use crate::value::Value;

use super::state::HostState;

pub(super) fn navigate(state: &mut HostState, url: &str) -> Result<Value, String> {
    let loaded = super::fetch::load(url)?;
    state.page.goto_html(loaded.url, loaded.body);
    state.page.run_scripts()?;
    state.started = true;
    Ok(super::snapshot::value(&state.page))
}
