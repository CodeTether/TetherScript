//! High-level ad-blocking engine combining network and cosmetic rules.
//!
//! Scripts reach ad blocking through the stateless `adblock_*` built-ins, which
//! compile a rule list per call. This engine is the stateful alternative, but
//! nothing constructs it yet, so `super::engine_tests` is its only consumer.
#![allow(dead_code, reason = "engine layer is not wired to a caller yet")]

use super::cosmetic::CosmeticMatcher;
use super::network::NetworkMatcher;
use super::parse::parse_list;
use super::rule::FilterType;

/// Combined engine holding compiled network + cosmetic rules and counters.
pub struct Engine {
    network: NetworkMatcher,
    cosmetic: CosmeticMatcher,
    total_rules: usize,
    pub blocked_count: u64,
    pub allowed_count: u64,
}

impl Engine {
    /// Create an empty engine.
    pub fn new() -> Self {
        Self {
            network: NetworkMatcher::new(),
            cosmetic: CosmeticMatcher::new(),
            total_rules: 0,
            blocked_count: 0,
            allowed_count: 0,
        }
    }

    /// Compile and add a filter-list text blob.
    pub fn add_list(&mut self, text: &str) {
        let rules = parse_list(text);
        self.total_rules += rules
            .iter()
            .filter(|rule| rule.filter_type != FilterType::Comment)
            .count();
        self.network.add(&rules);
        self.cosmetic.add(&rules);
    }

    /// Test whether a request URL should be blocked.
    pub fn should_block(&mut self, url: &str, source_domain: &str) -> bool {
        if self.network.is_blocked(url, source_domain) {
            self.blocked_count += 1;
            return true;
        }
        self.allowed_count += 1;
        false
    }

    /// Return cosmetic selectors for a given page domain.
    pub fn cosmetic_selectors(&self, domain: &str) -> Vec<String> {
        self.cosmetic.selectors_for(domain)
    }

    /// Number of non-comment rules compiled.
    pub fn rule_count(&self) -> usize {
        self.total_rules
    }
}
