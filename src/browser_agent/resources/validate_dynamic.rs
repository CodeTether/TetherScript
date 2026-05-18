//! Dynamic import validation for deterministic module graphs.

use std::collections::HashSet;

use crate::browser_agent::page::BrowserPage;

use super::{script_dynamic_ref, script_import, script_module_refs, script_resolve, url};

pub(crate) fn missing(page: &BrowserPage) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for reference in script_module_refs::collect(&page.session.document) {
        scan(page, &page.session.url, &reference, &mut seen, &mut out);
    }
    out
}

fn scan(
    page: &BrowserPage,
    base_url: &str,
    reference: &str,
    seen: &mut HashSet<String>,
    out: &mut Vec<String>,
) {
    let Some((url, source)) = script_resolve::text(&page.resources, base_url, reference) else {
        return;
    };
    let module_url = url::resolve(base_url, &url);
    if !seen.insert(module_url.clone()) {
        return;
    }
    for item in script_dynamic_ref::collect(source) {
        if script_resolve::text(&page.resources, &module_url, &item.url).is_none() {
            out.push(script_resolve::missing(&module_url, &item.url));
        }
    }
    for import in script_import::split(source).0 {
        scan(page, &module_url, &import.url, seen, out);
    }
}
