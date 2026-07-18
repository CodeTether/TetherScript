//! Closing native host tabs without losing active page state.

use super::super::super::state::HostState;

pub(super) fn close(state: &mut HostState, index: usize) -> Result<(), String> {
    if state.tabs.len() == 1 {
        return Err("browser.tabs_close: cannot close the last tab".into());
    }
    if index >= state.tabs.len() {
        return Err(format!("browser.tabs_close: tab {index} does not exist"));
    }
    if index != state.active_tab {
        state.tabs.remove(index);
        if index < state.active_tab {
            state.active_tab -= 1;
        }
        return Ok(());
    }
    let replacement = if index + 1 < state.tabs.len() {
        index + 1
    } else {
        index - 1
    };
    state.page = state.tabs[replacement]
        .take()
        .ok_or_else(|| "browser.tabs_close: replacement tab is unavailable".to_string())?;
    state.tabs.remove(index);
    state.active_tab = if replacement > index {
        replacement - 1
    } else {
        replacement
    };
    state.focused = None;
    Ok(())
}
