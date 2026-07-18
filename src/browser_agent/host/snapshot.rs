//! Compact native page snapshot values.

use crate::browser_agent::BrowserPage;
use crate::value::Value;

pub(super) fn value(page: &BrowserPage) -> Value {
    let viewport = super::value::map(vec![
        ("width", Value::Int(page.viewport_width)),
        ("height", Value::Int(page.viewport_height)),
    ]);
    super::value::map(vec![
        ("url", super::value::string(page.session.url.clone())),
        ("html", super::value::string(page.session.html.clone())),
        ("visible_text", super::value::string(document_text(page))),
        ("viewport", viewport),
    ])
}

pub(super) fn document_text(page: &BrowserPage) -> String {
    super::visible_text::value(page)
}
