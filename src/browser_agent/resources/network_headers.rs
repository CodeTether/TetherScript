//! Header helpers for routed subresources.

use crate::browser_agent::page::BrowserPage;
use crate::browser_cookie;

pub(super) fn redirect_location(status: u16, headers: &[(String, String)]) -> Option<String> {
    matches!(status, 301 | 302 | 303 | 307 | 308)
        .then(|| header(headers, "location").map(str::to_string))
        .flatten()
}

pub(super) fn apply_set_cookie(page: &mut BrowserPage, url: &str, headers: &[(String, String)]) {
    if !page.request_security_metadata(url).same_origin {
        return;
    }
    for (_, value) in headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
    {
        let _ = browser_cookie::set_server_cookie(&mut page.session.cookies, value, url);
    }
}

pub(super) fn validate_cors(
    page: &BrowserPage,
    url: &str,
    headers: &[(String, String)],
) -> Result<(), String> {
    let metadata = page.request_security_metadata(url);
    if metadata.same_origin || metadata.target_origin.is_opaque() {
        return Ok(());
    }
    let origin = metadata.request_origin.serialized();
    match header(headers, "access-control-allow-origin") {
        Some("*") => Ok(()),
        Some(value) if value.eq_ignore_ascii_case(&origin) => Ok(()),
        _ => Err(format!(
            "CORS blocked external resource from {} to {}",
            origin,
            metadata.target_origin.serialized()
        )),
    }
}

fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}
