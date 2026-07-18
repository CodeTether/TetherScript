//! Mutable state owned by one native browser host.

use crate::browser_agent::BrowserPage;

pub(super) struct HostState {
    pub(super) page: BrowserPage,
    pub(super) started: bool,
}

impl HostState {
    pub(super) fn new() -> Self {
        Self {
            page: BrowserPage::new(Default::default()),
            started: false,
        }
    }
}
