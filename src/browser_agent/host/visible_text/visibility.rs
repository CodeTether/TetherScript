//! Non-rendered subtree classification for visible-text extraction.

use crate::browser::{Document, Element};

pub(super) fn hidden(document: &Document, element: &Element, path: &[usize], css: &str) -> bool {
    if matches!(
        element.tag.as_str(),
        "script" | "style" | "title" | "template"
    ) || element.attrs.contains_key("hidden")
    {
        return true;
    }
    let Some(style) =
        crate::browser_agent::page::cssom::computed_style_at_path(document, css, path)
    else {
        return false;
    };
    style.get("display") == Some("none")
        || matches!(style.get("visibility"), Some("hidden" | "collapse"))
}
