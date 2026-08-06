//! Pattern normalization and URL matching utilities.

use super::CompiledRule;

pub(super) fn normalize(pattern: &str) -> (&str, bool, bool) {
    let mut p = pattern;
    let start = p.starts_with("||");
    if start {
        p = &p[2..];
    }
    let end = p.ends_with('^');
    if end {
        p = &p[..p.len() - 1];
    }
    (p, start, end)
}

pub(super) fn matches(pattern: &str, anchor_start: bool, _anchor_end: bool, url: &str) -> bool {
    if pattern.is_empty() {
        return true;
    }
    if anchor_start {
        url_boundary(url, pattern)
    } else {
        url.contains(pattern)
    }
}

fn url_boundary(url: &str, pattern: &str) -> bool {
    if let Some(at) = url.find("://") {
        let after = &url[at + 3..];
        host_match(after, pattern)
    } else {
        url.contains(pattern)
    }
}

fn host_match(after: &str, pattern: &str) -> bool {
    if let Some(slash) = after.find('/') {
        let host = &after[..slash];
        host == pattern || host.ends_with(&format!(".{pattern}")) || after.contains(pattern)
    } else {
        after == pattern || after.ends_with(&format!(".{pattern}"))
    }
}

pub(super) fn domain_ok(rule: &CompiledRule, source: &str) -> bool {
    !rule.excluded_domains.contains(source)
        && (rule.domains.is_empty() || rule.domains.contains(source))
}

pub(super) fn third_party_ok(rule: &CompiledRule, third: bool) -> bool {
    match rule.third_party {
        Some(req) => req == third,
        None => true,
    }
}
