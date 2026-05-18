//! Commit JavaScript-initiated location navigations.

use crate::browser_agent::page::BrowserPage;

pub(crate) fn commit_if_changed(
    source: String,
    page: &mut BrowserPage,
    from: &str,
) -> Result<(), String> {
    let target = page.session.url.clone();
    if target == from || !looks_like_location_navigation(&source) {
        return Ok(());
    }
    page.session.url = from.to_string();
    let url = super::url::resolve(from, &target);
    super::commit_api::commit_document(
        page,
        super::request::DocumentRequest::get(url),
        "script_navigation",
    )?;
    Ok(())
}

fn looks_like_location_navigation(source: &str) -> bool {
    source.contains("location.href")
        || source.contains("location.assign")
        || source.contains("location.replace")
        || source.contains("window.location")
}
