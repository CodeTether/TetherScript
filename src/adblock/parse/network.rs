//! Network rule constructor.

use super::super::classify::parse_options;
use super::super::rule::{FilterType, Rule};

pub(super) fn network(expr: &str, exception: bool) -> Rule {
    let (pattern, options_str) = match expr.split_once('$') {
        Some((p, o)) => (p.to_string(), o),
        None => (expr.to_string(), ""),
    };
    let opts = parse_options(options_str);
    Rule {
        filter_type: FilterType::Network,
        exception,
        pattern,
        domains: opts.domains,
        excluded_domains: opts.excluded_domains,
        third_party: opts.third_party,
        resource_types: opts.resource_types,
        selector: String::new(),
    }
}
