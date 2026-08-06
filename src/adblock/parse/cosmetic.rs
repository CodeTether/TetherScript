//! Cosmetic rule constructor.

use self::domains::split_domains;
use super::super::rule::{FilterType, Rule};
use super::super::ResourceType;

mod domains;

pub(super) fn cosmetic(line: &str) -> Option<Rule> {
    let idx = line.find("##")?;
    let allow = line[..idx].ends_with('#');
    let hash = if allow { "#@#" } else { "##" };
    let at = line.find(hash)?;
    let (domains, excluded) = split_domains(&line[..at]);
    Some(Rule {
        filter_type: if allow {
            FilterType::CosmeticAllow
        } else {
            FilterType::CosmeticHide
        },
        exception: false,
        pattern: String::new(),
        domains,
        excluded_domains: excluded,
        third_party: None,
        resource_types: ResourceType::default(),
        selector: line[at + hash.len()..].to_string(),
    })
}
