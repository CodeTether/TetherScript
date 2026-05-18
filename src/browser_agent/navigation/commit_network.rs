//! Commit document navigations loaded through the native network model.

use crate::browser_agent::navigation::request::DocumentRequest;
use crate::browser_agent::navigation::result::{NavigationKind, NavigationResult};
use crate::browser_agent::page::BrowserPage;

pub(super) fn document(
    page: &mut BrowserPage,
    request: DocumentRequest,
    action: &str,
) -> Result<NavigationResult, String> {
    let from = page.session.url.clone();
    super::lifecycle::leave(
        page,
        action,
        NavigationKind::DocumentReplacement,
        &from,
        &request.url,
    );
    let load = super::network_fetch::load(page, request)?;
    commit_session(page, load.final_url, load.html);
    page.runtime = None;
    let result = super::load_api::push(page, action, NavigationKind::DocumentReplacement);
    super::lifecycle::finish_commit(page, action, result.kind, &from, &result.navigation.url);
    Ok(result)
}

fn commit_session(page: &mut BrowserPage, final_url: String, html: String) {
    let network_len = page.session.network.len();
    page.session.goto_html(final_url, html);
    page.session.network.truncate(network_len);
}
