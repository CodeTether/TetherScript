//! CORS preflight-response validation.

use super::super::super::{FetchRequest, FetchResponseParts};

pub(super) fn response(
    request: &FetchRequest,
    parts: &FetchResponseParts,
    request_headers: &[String],
) -> Result<(), String> {
    if !(200..300).contains(&parts.status) {
        return Err(super::validate::blocked(
            request,
            "preflight did not succeed",
        ));
    }
    super::validate::response(request, &parts.headers)?;
    allow_method(request, &parts.headers)?;
    allow_headers(request, &parts.headers, request_headers)
}

fn allow_method(request: &FetchRequest, headers: &[(String, String)]) -> Result<(), String> {
    let Some(value) = super::validate::header(headers, "access-control-allow-methods") else {
        return Err(super::validate::blocked(request, "missing allowed methods"));
    };
    if contains_token(value, &request.method) {
        Ok(())
    } else {
        Err(super::validate::blocked(request, "method not allowed"))
    }
}

fn allow_headers(
    request: &FetchRequest,
    headers: &[(String, String)],
    requested: &[String],
) -> Result<(), String> {
    let Some(value) = super::validate::header(headers, "access-control-allow-headers") else {
        return requested
            .is_empty()
            .then_some(())
            .ok_or_else(|| super::validate::blocked(request, "missing allowed headers"));
    };
    if requested.iter().all(|name| contains_token(value, name)) {
        Ok(())
    } else {
        Err(super::validate::blocked(request, "header not allowed"))
    }
}

fn contains_token(value: &str, token: &str) -> bool {
    value.trim() == "*"
        || value
            .split(',')
            .any(|part| part.trim().eq_ignore_ascii_case(token))
}
