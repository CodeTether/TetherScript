//! External script application for page resources.

use super::discover::ResourceReference;
use super::{inline, url, ResourceKind, ResourceRegistry};

pub(crate) fn inline_scripts(
    mut html: String,
    refs: &[ResourceReference],
    registry: &ResourceRegistry,
    base_url: &str,
) -> Result<String, String> {
    for reference in refs.iter().filter(|item| item.kind == ResourceKind::Script) {
        let (url, source) = text_resource(registry, ResourceKind::Script, base_url, &reference.url)
            .ok_or_else(|| missing("script", base_url, &reference.url))?;
        html = inline::append_script(html, &url, source);
    }
    Ok(html)
}

fn text_resource<'a>(
    registry: &'a ResourceRegistry,
    kind: ResourceKind,
    base_url: &str,
    reference: &str,
) -> Option<(String, &'a str)> {
    url::candidates(base_url, reference)
        .into_iter()
        .find_map(|url| registry.text(kind, &url).map(|text| (url, text)))
}

fn missing(kind: &str, base_url: &str, reference: &str) -> String {
    let resolved = url::resolve(base_url, reference);
    format!(
        "missing external {} resource: {} (resolved {})",
        kind, reference, resolved
    )
}
