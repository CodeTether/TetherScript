//! Redirect-following document navigation fetches.

use crate::browser_agent::navigation::request::DocumentRequest;
use crate::browser_agent::page::BrowserPage;

const MAX_REDIRECTS: usize = 20;

pub(super) struct DocumentLoad {
    pub(super) final_url: String,
    pub(super) html: String,
}

pub(super) fn load(
    page: &mut BrowserPage,
    mut request: DocumentRequest,
) -> Result<DocumentLoad, String> {
    let requested_url = request.url.clone();
    for redirects in 0..=MAX_REDIRECTS {
        let response = match super::network_request::send(page, &request)? {
            super::network_request::DocumentOutcome::Continue => return Ok(empty(request.url)),
            super::network_request::DocumentOutcome::Response(response) => response,
        };
        super::network_headers::apply_set_cookie(page, &request.url, &response.headers);
        if let Some(location) =
            super::network_headers::redirect_location(response.status, &response.headers)
        {
            if redirects == MAX_REDIRECTS {
                return Err(format!("document redirect limit exceeded: {requested_url}"));
            }
            request = super::network_redirect::follow(request, response.status, &location);
            continue;
        }
        if (200..300).contains(&response.status) {
            return Ok(DocumentLoad {
                final_url: request.url,
                html: response.body,
            });
        }
        return Err(format!(
            "document navigation failed: {} status {}",
            request.url, response.status
        ));
    }
    Err(format!("document redirect limit exceeded: {requested_url}"))
}

fn empty(final_url: String) -> DocumentLoad {
    DocumentLoad {
        final_url,
        html: String::new(),
    }
}
