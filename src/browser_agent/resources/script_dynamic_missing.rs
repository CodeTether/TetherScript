//! Browser-shaped fallback for missing dynamic imports.

use super::url;

pub(crate) fn rejection(base_url: &str, reference: &str) -> String {
    let resolved = url::resolve(base_url, reference);
    format!(
        "Promise.reject(TypeError({}))",
        quoted(&format!(
            "Failed to fetch dynamically imported module: {resolved}"
        ))
    )
}

fn quoted(value: &str) -> String {
    let mut out = String::from("'");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out.push('\'');
    out
}
