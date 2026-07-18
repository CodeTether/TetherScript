//! Activation of an existing native host tab.

use super::super::super::state::HostState;

pub(super) fn activate(state: &mut HostState, index: usize) -> Result<(), String> {
    if index == state.active_tab {
        return Ok(());
    }
    let slot = state
        .tabs
        .get_mut(index)
        .ok_or_else(|| format!("browser.tabs_select: tab {index} does not exist"))?;
    let target = slot
        .take()
        .ok_or_else(|| format!("browser.tabs_select: tab {index} is unavailable"))?;
    let old = std::mem::replace(&mut state.page, target);
    state.tabs[state.active_tab] = Some(old);
    state.active_tab = index;
    state.focused = None;
    Ok(())
}
