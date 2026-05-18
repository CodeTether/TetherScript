//! CORS actual-response validation.

use super::super::super::{request_host, FetchRequest};

pub(super) fn response(request: &FetchRequest, headers: &[(String, String)]) -> Result<(), String> {
    let origin = request_host::origin(&request.initiator_url);
    let Some(value) = header(headers, "access-control-allow-origin") else {
        return Err(blocked(request, "missing access-control-allow-origin"));
    };
    let value = value.trim();
    if value == "*" && request.credentials == "include" {
        return Err(blocked(request, "wildcard origin with credentials"));
    }
    if value != "*" && !value.eq_ignore_ascii_case(&origin) {
        return Err(blocked(request, "origin not allowed by response"));
    }
    if request.credentials != "include" {
        return Ok(());
    }
    match header(headers, "access-control-allow-credentials") {
        Some(value) if value.trim().eq_ignore_ascii_case("true") => Ok(()),
        _ => Err(blocked(request, "missing access-control-allow-credentials")),
    }
}

pub(super) fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

pub(super) fn blocked(request: &FetchRequest, reason: &str) -> String {
    format!(
        "CORS blocked cross-origin response from {} to {}: {}",
        request_host::origin(&request.initiator_url),
        request_host::origin(&request.url),
        reason
    )
}
