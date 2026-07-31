//! Textual-type classification.
//!
//! Split from [`super::mime_header`] to respect the 50-line file limit.

/// Report whether a content type carries text.
///
/// # Arguments
///
/// * `content_type` — A full header or a bare media type.
///
/// # Returns
///
/// True for every `text/*` type, and for JSON, XML, JavaScript, and SVG, which
/// are textual despite their `application/` or `image/` prefix. Structured-suffix
/// types such as `application/ld+json` are covered by the `+json`/`+xml` check.
pub(super) fn is_text(content_type: &str) -> bool {
    let media = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if media.starts_with("text/") {
        return true;
    }
    matches!(
        media.as_str(),
        "application/json"
            | "application/xml"
            | "application/javascript"
            | "application/ecmascript"
            | "image/svg+xml"
    ) || media.ends_with("+json")
        || media.ends_with("+xml")
}
