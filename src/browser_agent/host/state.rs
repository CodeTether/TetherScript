//! Mutable state owned by one native browser host.

use crate::browser_agent::BrowserPage;
use crate::browser_agent::Locator;

pub(super) struct HostState {
    pub(super) page: BrowserPage,
    pub(super) started: bool,
    pub(super) focused: Option<Locator>,
    pub(super) tabs: Vec<Option<BrowserPage>>,
    pub(super) active_tab: usize,
}

impl HostState {
    pub(super) fn new() -> Self {
        Self {
            page: BrowserPage::new(Default::default()),
            started: false,
            focused: None,
            tabs: vec![None],
            active_tab: 0,
        }
    }
}
