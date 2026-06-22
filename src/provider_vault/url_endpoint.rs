//! Endpoint formatting from parsed URLs.

use super::url::{default_port, Scheme, Url};

pub(crate) fn from_url(url: &Url) -> String {
    let scheme = scheme_name(&url.scheme);
    let port = if url.port == default_port(&url.scheme) {
        String::new()
    } else {
        format!(":{}", url.port)
    };
    format!("{scheme}://{}{port}", url.host)
}

fn scheme_name(scheme: &Scheme) -> &'static str {
    match scheme {
        Scheme::Https => "https",
        Scheme::Http => "http",
    }
}
