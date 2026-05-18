//! Click-result navigation dispatch.

use crate::browser_agent::navigation::DocumentRequest;
use crate::browser_agent::page::BrowserPage;
use crate::browser_agent::resolve::Resolved;
use crate::js::JsValue;

pub(crate) fn after_click(
    page: &mut BrowserPage,
    resolved: &Resolved,
    value: &JsValue,
) -> Result<(), String> {
    if value == &JsValue::Bool(false) {
        return Ok(());
    }
    let action = navigation_target(page, &resolved.dom.path);
    if let Some((request, action)) = action {
        super::commit::document(page, request, action)?;
    }
    Ok(())
}

fn navigation_target(
    page: &BrowserPage,
    path: &[usize],
) -> Option<(DocumentRequest, &'static str)> {
    let current = page.session.url.as_str();
    let document = &page.session.document;
    if let Some(href) = super::anchor::href(document, path) {
        let url = super::url::resolve(current, &href);
        return Some((DocumentRequest::get(url), "anchor_click"));
    }
    super::form::submit_target(document, path, current).map(|request| (request, "form_submit"))
}
