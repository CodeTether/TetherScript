//! Routed loading for missing page subresources.

#[path = "network_fetch.rs"]
mod fetch;
#[path = "network_headers.rs"]
mod headers;
#[path = "network_kind.rs"]
mod kind;
#[path = "network_modules.rs"]
mod modules;
#[path = "network_request.rs"]
mod request;
#[path = "network_request_build.rs"]
mod request_build;
#[path = "network_request_event.rs"]
mod request_event;
#[path = "network_source_maps.rs"]
mod source_maps;
#[path = "network_store.rs"]
mod store;

use crate::browser_agent::page::BrowserPage;

use super::{discover, preload};

pub(crate) fn load_missing(
    page: &mut BrowserPage,
    refs: &[discover::ResourceReference],
) -> Result<(), String> {
    let mut refs = refs.to_vec();
    refs.extend(preload::collect(&page.session.document));
    for reference in refs {
        load_one(page, &reference)?;
    }
    modules::load_missing(page)?;
    source_maps::load_missing(page)
}

fn load_one(page: &mut BrowserPage, reference: &discover::ResourceReference) -> Result<(), String> {
    if store::has(page, reference) {
        return Ok(());
    }
    if let Some(resource) = fetch::resource(page, reference)? {
        store::save(page, reference, resource);
    }
    Ok(())
}
