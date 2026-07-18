//! Render-aware text extraction for browser snapshots and waits.

use crate::browser::{Document, Node};
use crate::browser_agent::BrowserPage;

mod visibility;

pub(super) fn value(page: &BrowserPage) -> String {
    let css = crate::browser_agent::page::cssom::active_css_for(
        &page.session.css,
        page.viewport_width,
        page.media,
    );
    let mut chunks = Vec::new();
    for (index, node) in page.session.document.children.iter().enumerate() {
        collect(&page.session.document, node, &[index], &css, &mut chunks);
    }
    chunks.join(" ")
}

fn collect(document: &Document, node: &Node, path: &[usize], css: &str, out: &mut Vec<String>) {
    match node {
        Node::Text(text) => push_text(text, out),
        Node::Element(element) if !visibility::hidden(document, element, path, css) => {
            for (index, child) in element.children.iter().enumerate() {
                let mut child_path = path.to_vec();
                child_path.push(index);
                collect(document, child, &child_path, css, out);
            }
        }
        Node::Element(_) => {}
    }
}

fn push_text(text: &str, out: &mut Vec<String>) {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if !text.is_empty() {
        out.push(text);
    }
}
