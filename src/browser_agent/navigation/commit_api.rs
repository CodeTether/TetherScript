//! Internal navigation commits from default actions.

use crate::browser_agent::navigation::load_api;
use crate::browser_agent::navigation::request::DocumentRequest;
use crate::browser_agent::navigation::result::{NavigationKind, NavigationResult};
use crate::browser_agent::page::BrowserPage;

pub(crate) fn commit_document(
    page: &mut BrowserPage,
    request: DocumentRequest,
    action: &str,
) -> Result<NavigationResult, String> {
    let from = page.session.url.clone();
    if let Some(hash) = same_document_hash(page, &request) {
        page.session.set_hash(hash);
        let result = load_api::push(page, action, NavigationKind::SameDocument);
        super::lifecycle::finish_commit(page, action, result.kind, &from, &result.navigation.url);
        return Ok(result);
    }
    super::commit_network::document(page, request, action)
}

fn same_document_hash(page: &BrowserPage, request: &DocumentRequest) -> Option<String> {
    if request.method == "GET" && request.body.is_none() {
        super::url_kind::same_document_hash(&page.session.url, &request.url).map(str::to_string)
    } else {
        None
    }
}
