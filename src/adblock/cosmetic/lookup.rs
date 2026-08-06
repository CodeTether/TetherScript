//! Domain scoping and selector collection.

use super::Entry;
use std::collections::HashSet;

pub(super) fn domain_ok(entry: &Entry, domain: &str) -> bool {
    if entry.excluded.contains(domain) {
        return false;
    }
    entry.domains.is_empty() || entry.domains.contains(domain)
}

pub(super) fn collect_selectors(
    hide: &[Entry],
    allow: &HashSet<String>,
    domain: &str,
) -> Vec<String> {
    let d = domain.to_lowercase();
    let mut out: Vec<String> = hide
        .iter()
        .filter(|e| domain_ok(e, &d))
        .map(|e| e.selector.clone())
        .collect();
    out.retain(|s| !allow.contains(&s.to_lowercase()));
    out.sort();
    out.dedup();
    out
}
