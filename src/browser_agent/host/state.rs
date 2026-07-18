//! Mutable state owned by one native browser host.

use crate::browser_agent::context::context_state::SharedContextState;
use crate::browser_agent::BrowserPage;
use crate::browser_agent::Locator;

pub(super) struct HostState {
    pub(super) page: BrowserPage,
    pub(super) started: bool,
    pub(super) focused: Option<Locator>,
    pub(super) tabs: Vec<Option<BrowserPage>>,
    pub(super) active_tab: usize,
    pub(super) context_state: SharedContextState,
}

impl HostState {
    pub(super) fn new() -> Self {
        let context_state = crate::browser_agent::context::context_state::shared_context_state();
        let mut page = BrowserPage::new(Default::default());
        page.attach_context_state(context_state.clone(), true);
        Self {
            page,
            started: false,
            focused: None,
            tabs: vec![None],
            active_tab: 0,
            context_state,
        }
    }
}
