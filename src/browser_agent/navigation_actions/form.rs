//! Form navigation target lookup.

use crate::browser::Document;
use crate::browser_agent::navigation::DocumentRequest;

pub(crate) fn submit_target(
    document: &Document,
    submitter_path: &[usize],
    current_url: &str,
) -> Option<DocumentRequest> {
    let submitter = super::dom::element_at_path(document, submitter_path)?;
    let (_, form) = super::dom::closest_form(document, submitter_path)?;
    super::form_request::target_for_form(form, Some(submitter), current_url)
}

pub(crate) fn form_target(
    document: &Document,
    form_path: &[usize],
    current_url: &str,
) -> Option<DocumentRequest> {
    let form = super::dom::element_at_path(document, form_path)?;
    if form.tag.eq_ignore_ascii_case("form") {
        super::form_request::target_for_form(form, None, current_url)
    } else {
        None
    }
}
