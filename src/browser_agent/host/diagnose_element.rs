//! DOM-backed component evidence for framework diagnostics.

use crate::browser::{query_selector, text_content, Element, Node};
use crate::browser_agent::BrowserPage;
use crate::value::Value;

#[path = "diagnose_element_attrs.rs"]
mod attrs;

pub(super) fn summary(page: &BrowserPage, selector: &str) -> Result<Value, String> {
    let element = find(page, selector)?;
    Ok(super::super::super::value::map(vec![
        ("selector", super::super::super::value::string(selector)),
        ("tag", super::super::super::value::string(&element.tag)),
        (
            "component",
            super::super::super::value::string(
                element.attrs.get("data-component").unwrap_or(&element.tag),
            ),
        ),
        (
            "text",
            super::super::super::value::string(text_content(&Node::Element(element.clone()))),
        ),
    ]))
}

pub(super) fn attributes(page: &BrowserPage, selector: &str) -> Result<Value, String> {
    attrs::attributes(page, selector)
}

pub(super) fn metadata(page: &BrowserPage, selector: &str, name: &str) -> Result<Value, String> {
    attrs::metadata(page, selector, name)
}

fn find(page: &BrowserPage, selector: &str) -> Result<Element, String> {
    query_selector(&page.session.document, selector)
        .into_iter()
        .find_map(|node| match node {
            Node::Element(element) => Some(element),
            _ => None,
        })
        .ok_or_else(|| format!("browser.diagnose: selector `{selector}` not found"))
}
