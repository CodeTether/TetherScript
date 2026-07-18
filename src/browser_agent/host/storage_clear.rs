//! Complete shared browser-context storage clearing.

use crate::browser_agent::BrowserPage;
use crate::value::Value;

use super::super::super::state::HostState;

pub(super) fn invoke(state: &mut HostState) -> Value {
    {
        let mut shared = state.context_state.borrow_mut();
        shared.cookies.clear();
        shared.local_storage.clear();
        shared.indexed_db.clear();
        shared.service_workers.clear();
    }
    clear_page(&mut state.page);
    for page in state.tabs.iter_mut().flatten() {
        clear_page(page);
    }
    Value::Bool(true)
}

fn clear_page(page: &mut BrowserPage) {
    page.sync_context_state_into_session();
    page.session.session_storage.clear();
    page.runtime = None;
}
