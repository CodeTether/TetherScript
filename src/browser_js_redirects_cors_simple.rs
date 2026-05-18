//! CORS simple-request classification.

use super::super::super::FetchRequest;

pub(super) fn needs_preflight(request: &FetchRequest) -> bool {
    !simple_method(&request.method) || !requested_headers(request).is_empty()
}

pub(super) fn requested_headers(request: &FetchRequest) -> Vec<String> {
    let mut names = Vec::new();
    for (name, value) in &request.headers {
        let name = name.to_ascii_lowercase();
        if non_simple_header(&name, value) {
            push_unique(&mut names, name);
        }
    }
    names
}

fn simple_method(method: &str) -> bool {
    matches!(method, "GET" | "HEAD" | "POST")
}

fn non_simple_header(name: &str, value: &str) -> bool {
    if matches!(name, "origin" | "cookie") {
        return false;
    }
    match name {
        "accept" | "accept-language" | "content-language" => false,
        "content-type" => !simple_content_type(value),
        _ => true,
    }
}

fn simple_content_type(value: &str) -> bool {
    let mime = value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    matches!(
        mime.as_str(),
        "application/x-www-form-urlencoded" | "multipart/form-data" | "text/plain"
    )
}

fn push_unique(names: &mut Vec<String>, name: String) {
    if !names.iter().any(|existing| existing == &name) {
        names.push(name);
    }
}
