//! Module source snapshots for recursive route loading.

use crate::browser_agent::page::{
    resources::{ResourceKind, ResourcePayload},
    BrowserPage,
};

pub(super) fn collect(page: &BrowserPage) -> Vec<(String, String)> {
    page.resources
        .entries()
        .iter()
        .filter_map(|entry| match &entry.payload {
            ResourcePayload::Text(source) if entry.kind == ResourceKind::Script => {
                Some((entry.url.clone(), source.clone()))
            }
            _ => None,
        })
        .collect()
}
