//! Routed source-map loading for fetched scripts.

use crate::browser_agent::page::resources::{ResourceKind, ResourcePayload};
use crate::browser_agent::page::BrowserPage;

use super::super::{discover::ResourceReference, url};

pub(super) fn load_missing(page: &mut BrowserPage) -> Result<(), String> {
    for reference in references(page) {
        if super::store::has(page, &reference) {
            continue;
        }
        if let Some(resource) = super::fetch::resource(page, &reference)? {
            super::store::save(page, &reference, resource);
        }
    }
    Ok(())
}

fn references(page: &BrowserPage) -> Vec<ResourceReference> {
    page.resources
        .entries()
        .iter()
        .filter_map(|resource| match &resource.payload {
            ResourcePayload::Text(source) if resource.kind == ResourceKind::Script => {
                mapping_url(source).map(|map| ResourceReference {
                    kind: ResourceKind::SourceMap,
                    url: url::resolve(&resource.url, &map),
                })
            }
            _ => None,
        })
        .collect()
}

fn mapping_url(source: &str) -> Option<String> {
    source.lines().rev().find_map(|line| {
        line.split_once("sourceMappingURL=")
            .map(|(_, value)| value.trim().to_string())
    })
}
