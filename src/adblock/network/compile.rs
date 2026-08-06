//! Rule compilation from [`Rule`] into [`CompiledRule`].

use super::super::rule::Rule;
use super::helpers::normalize;
use super::CompiledRule;
use std::collections::HashSet;

/// Compile a single network rule, returning `None` for non-network rules.
pub(super) fn compile_rule(rule: &Rule) -> Option<CompiledRule> {
    let (pattern, anchor_start, anchor_end) = normalize(&rule.pattern);
    Some(CompiledRule {
        exception: rule.exception,
        anchor_start,
        anchor_end,
        pattern_lower: pattern.to_lowercase(),
        domains: rule.domains.iter().cloned().collect::<HashSet<_>>(),
        excluded_domains: rule
            .excluded_domains
            .iter()
            .cloned()
            .collect::<HashSet<_>>(),
        third_party: rule.third_party,
    })
}
