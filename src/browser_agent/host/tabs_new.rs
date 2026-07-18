//! Creation of native host tabs.

use crate::browser_agent::BrowserPage;

use super::super::super::state::HostState;

pub(super) fn open(state: &mut HostState, url: &str) -> Result<(), String> {
    let loaded = super::super::super::fetch::load(url)?;
    let mut page = BrowserPage::new(Default::default());
    page.goto_html(loaded.url, loaded.body);
    page.run_scripts()?;
    let old = std::mem::replace(&mut state.page, page);
    state.tabs[state.active_tab] = Some(old);
    state.tabs.push(None);
    state.active_tab = state.tabs.len() - 1;
    state.focused = None;
    state.started = true;
    Ok(())
}
