//! Comment rule constructor.

use super::super::rule::{FilterType, Rule};
use super::super::ResourceType;

pub(super) fn comment(text: &str) -> Rule {
    Rule {
        filter_type: FilterType::Comment,
        exception: false,
        pattern: text.to_string(),
        domains: Vec::new(),
        excluded_domains: Vec::new(),
        third_party: None,
        resource_types: ResourceType::default(),
        selector: String::new(),
    }
}
