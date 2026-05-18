//! Redirect-following subresource fetches.

use crate::browser_agent::page::BrowserPage;

use super::super::{discover::ResourceReference, url};
use super::store::LoadedResource;

const MAX_REDIRECTS: usize = 20;

pub(super) fn resource(
    page: &mut BrowserPage,
    reference: &ResourceReference,
) -> Result<Option<LoadedResource>, String> {
    let requested_url = url::resolve(&page.session.url, &reference.url);
    let mut current_url = requested_url.clone();
    for redirects in 0..=MAX_REDIRECTS {
        let Some(response) = super::request::send(page, &current_url)? else {
            return Ok(None);
        };
        super::headers::apply_set_cookie(page, &current_url, &response.headers);
        if let Some(location) =
            super::headers::redirect_location(response.status, &response.headers)
        {
            if redirects == MAX_REDIRECTS {
                return Err(format!(
                    "external resource redirect limit exceeded: {requested_url}"
                ));
            }
            current_url = url::resolve(&current_url, &location);
            continue;
        }
        super::headers::validate_cors(page, &current_url, &response.headers)?;
        if (200..300).contains(&response.status) {
            return Ok(Some(LoadedResource {
                requested_url,
                final_url: current_url,
                body: response.body,
            }));
        }
        return Err(failed(reference, &current_url, response.status));
    }
    Err(format!(
        "external resource redirect limit exceeded: {requested_url}"
    ))
}

fn failed(reference: &ResourceReference, url: &str, status: u16) -> String {
    format!(
        "failed external {} resource: {} (resolved {}) status {}",
        super::kind::name(reference.kind),
        reference.url,
        url,
        status
    )
}
