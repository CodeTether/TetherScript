//! Route-table loading for static module imports.

#[path = "network_module_sources.rs"]
mod sources;

use std::collections::HashSet;

use crate::browser_agent::page::BrowserPage;

use super::super::{discover::ResourceReference, script_import, script_resolve, url, ResourceKind};

pub(super) fn load_missing(page: &mut BrowserPage) -> Result<(), String> {
    let mut seen = HashSet::new();
    for (module_url, source) in sources::collect(page) {
        load_from_source(page, &module_url, &source, &mut seen)?;
    }
    Ok(())
}

fn load_from_source(
    page: &mut BrowserPage,
    current_url: &str,
    source: &str,
    seen: &mut HashSet<String>,
) -> Result<(), String> {
    if !seen.insert(current_url.into()) {
        return Ok(());
    }
    for import in script_import::split(source).0 {
        load_import(page, current_url, &import.url)?;
        if let Some((url, source)) = module_source(page, current_url, &import.url) {
            load_from_source(page, &url, &source, seen)?;
        }
    }
    Ok(())
}

fn load_import(page: &mut BrowserPage, base_url: &str, reference: &str) -> Result<(), String> {
    let reference = ResourceReference {
        kind: ResourceKind::Script,
        url: url::resolve(base_url, reference),
    };
    if !super::store::has(page, &reference) {
        if let Some(resource) = super::fetch::resource(page, &reference)? {
            super::store::save(page, &reference, resource);
        }
    }
    Ok(())
}

fn module_source(page: &BrowserPage, base_url: &str, reference: &str) -> Option<(String, String)> {
    script_resolve::text(&page.resources, base_url, reference)
        .map(|(url, source)| (url::resolve(base_url, &url), source.to_string()))
}
