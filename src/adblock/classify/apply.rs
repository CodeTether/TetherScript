//! Option application helpers.

use super::{Options, ResourceType};

pub(super) fn set_type(opts: &mut Options, _any_type: bool, f: impl Fn(&mut ResourceType)) {
    if opts.resource_types.is_all() {
        opts.resource_types = ResourceType::default();
    }
    f(&mut opts.resource_types);
}

pub(super) fn collect_domains(opts: &mut Options, raw: &str) {
    for dom in raw.split('|') {
        let dom = dom.trim();
        if let Some(rest) = dom.strip_prefix('~') {
            opts.excluded_domains.push(rest.to_lowercase());
        } else if !dom.is_empty() {
            opts.domains.push(dom.to_lowercase());
        }
    }
}
