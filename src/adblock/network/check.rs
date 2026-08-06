//! Matching loop over compiled rules.

use super::helpers::{domain_ok, matches, third_party_ok};
use super::CompiledRule;

/// Test all rules and return whether the request should be blocked.
pub(super) fn check_rules(
    rules: &[CompiledRule],
    url_lower: &str,
    source: &str,
    third: bool,
) -> bool {
    let mut blocked = false;
    for rule in rules {
        if !domain_ok(rule, source) || !third_party_ok(rule, third) {
            continue;
        }
        if !matches(
            &rule.pattern_lower,
            rule.anchor_start,
            rule.anchor_end,
            url_lower,
        ) {
            continue;
        }
        if rule.exception {
            return false;
        }
        blocked = true;
    }
    blocked
}
