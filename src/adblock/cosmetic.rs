//! Cosmetic (element-hiding) filter matching.

use super::rule::{FilterType, Rule};
use std::collections::HashSet;

use self::lookup::collect_selectors;

mod lookup;

pub struct CosmeticMatcher {
    hide: Vec<Entry>,
    allow: HashSet<String>,
}

struct Entry {
    selector: String,
    domains: HashSet<String>,
    excluded: HashSet<String>,
}

impl CosmeticMatcher {
    pub fn new() -> Self {
        Self {
            hide: Vec::new(),
            allow: HashSet::new(),
        }
    }

    pub fn add(&mut self, rules: &[Rule]) {
        for rule in rules {
            match rule.filter_type {
                FilterType::CosmeticHide => self.push_hide(rule),
                FilterType::CosmeticAllow => {
                    self.allow.insert(rule.selector.to_lowercase());
                }
                _ => {}
            }
        }
    }

    pub fn selectors_for(&self, domain: &str) -> Vec<String> {
        collect_selectors(&self.hide, &self.allow, domain)
    }
}

impl CosmeticMatcher {
    fn push_hide(&mut self, rule: &Rule) {
        if !rule.selector.is_empty() {
            self.hide.push(Entry {
                selector: rule.selector.clone(),
                domains: rule.domains.iter().cloned().collect(),
                excluded: rule.excluded_domains.iter().cloned().collect(),
            });
        }
    }
}
