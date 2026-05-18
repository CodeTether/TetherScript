//! Resource registry writes for routed subresources.

use crate::browser_agent::page::BrowserPage;

use super::super::{discover::ResourceReference, url, ResourceKind};

pub(super) struct LoadedResource {
    pub requested_url: String,
    pub final_url: String,
    pub body: String,
}

pub(super) fn has(page: &BrowserPage, reference: &ResourceReference) -> bool {
    url::candidates(&page.session.url, &reference.url)
        .iter()
        .any(|candidate| page.resources.has(reference.kind, candidate))
}

pub(super) fn save(
    page: &mut BrowserPage,
    reference: &ResourceReference,
    resource: LoadedResource,
) {
    match reference.kind {
        ResourceKind::Image => save_image(page, resource),
        kind => save_text(page, kind, resource),
    }
}

fn save_text(page: &mut BrowserPage, kind: ResourceKind, resource: LoadedResource) {
    let body = resource.body.clone();
    page.resources
        .register_text(resource.requested_url.clone(), kind, body);
    if resource.final_url != resource.requested_url {
        page.resources
            .register_text(resource.final_url, kind, resource.body);
    }
    page.runtime = None;
}

fn save_image(page: &mut BrowserPage, resource: LoadedResource) {
    page.resources.register_image(
        resource.requested_url.clone(),
        resource.body.clone().into_bytes(),
    );
    if resource.final_url != resource.requested_url {
        page.resources
            .register_image(resource.final_url, resource.body.into_bytes());
    }
}
