//! Deterministic page navigation commits.

use crate::browser_agent::navigation::DocumentRequest;
use crate::browser_agent::page::BrowserPage;

pub(crate) fn document(
    page: &mut BrowserPage,
    request: DocumentRequest,
    action: &str,
) -> Result<(), String> {
    crate::browser_agent::navigation::commit_document(page, request, action).map(|_| ())
}
