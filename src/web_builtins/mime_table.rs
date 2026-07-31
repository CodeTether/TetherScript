//! Extension-to-content-type mapping.
//!
//! The mappings for html, htm, css, js, mjs, json, xml, svg, png, jpg, jpeg,
//! gif, ico, wasm, and txt are copied verbatim from
//! [`crate::http_static`]'s `content_type::for_path`, so the script-visible
//! built-in and the native static server can never disagree about a file. The
//! remaining types extend that set rather than redefining it.

/// Look up a content type by file extension.
///
/// # Arguments
///
/// * `path` — File name or path. Only the final extension is considered.
///
/// # Returns
///
/// The content type, or `application/octet-stream` when the extension is unknown
/// or absent. Textual types carry `; charset=utf-8`, matching the static server.
pub(super) fn for_path(path: &str) -> &'static str {
    match extension(path).as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "xml" => "application/xml",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        "txt" => "text/plain; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        _ => "application/octet-stream",
    }
}

/// Extract the lowercase final extension, if any.
///
/// A leading-dot name such as `.gitignore` has no extension: splitting on the
/// last dot would otherwise treat `gitignore` as one and guess a type for a
/// dotfile.
fn extension(path: &str) -> String {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => ext.to_ascii_lowercase(),
        _ => String::new(),
    }
}
