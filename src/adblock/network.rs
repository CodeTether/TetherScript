//! Network request matching against compiled rules.

use super::rule::{FilterType, Rule};
use std::collections::HashSet;

use self::check::check_rules;
use self::compile::compile_rule;
use self::helpers::{domain_ok, matches, normalize, third_party_ok};
use self::host::host_of;

mod check;
mod compile;
mod helpers;
mod host;

/// Compiles network rules for fast matching.
pub struct NetworkMatcher {
    rules: Vec<CompiledRule>,
}

pub(super) struct CompiledRule {
    pub(super) exception: bool,
    pub(super) anchor_start: bool,
    pub(super) anchor_end: bool,
    pub(super) pattern_lower: String,
    pub(super) domains: HashSet<String>,
    pub(super) excluded_domains: HashSet<String>,
    pub(super) third_party: Option<bool>,
}

impl NetworkMatcher {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add(&mut self, rules: &[Rule]) {
        for rule in rules {
            if rule.filter_type != FilterType::Network {
                continue;
            }
            if let Some(compiled) = compile_rule(rule) {
                self.rules.push(compiled);
            }
        }
    }

    pub fn is_blocked(&self, url: &str, source_domain: &str) -> bool {
        let url_lower = url.to_lowercase();
        let source = source_domain.to_lowercase();
        let third = host_of(&url_lower) != source;
        check_rules(&self.rules, &url_lower, &source, third)
    }
}
