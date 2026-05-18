//! Header handling for routed document navigations.

use crate::{browser_agent::page::BrowserPage, browser_cookie};

pub(super) fn redirect_location(status: u16, headers: &[(String, String)]) -> Option<String> {
    matches!(status, 301 | 302 | 303 | 307 | 308)
        .then(|| header(headers, "location").map(str::to_string))
        .flatten()
}

pub(super) fn apply_set_cookie(page: &mut BrowserPage, url: &str, headers: &[(String, String)]) {
    for (_, value) in headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
    {
        let _ = browser_cookie::set_server_cookie(&mut page.session.cookies, value, url);
    }
}

fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}
