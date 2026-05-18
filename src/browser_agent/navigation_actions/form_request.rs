//! Form navigation request construction.

use crate::browser::Element;
use crate::browser_agent::navigation::DocumentRequest;

pub(crate) fn target_for_form(
    form: &Element,
    submitter: Option<&Element>,
    current_url: &str,
) -> Option<DocumentRequest> {
    let method = method(form);
    if !matches!(method.as_str(), "get" | "post") {
        return None;
    }
    let action = form.attrs.get("action").map_or(current_url, String::as_str);
    let url = super::url::resolve(current_url, action);
    let entries = super::entries::collect(form, submitter);
    if method == "post" {
        Some(DocumentRequest::form_post(
            url,
            super::encode::pairs(&entries),
        ))
    } else {
        Some(DocumentRequest::get(super::query::with_query(
            &url, &entries,
        )))
    }
}

fn method(form: &Element) -> String {
    form.attrs
        .get("method")
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_else(|| "get".into())
}
