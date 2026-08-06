//! uBlock Origin / Adblock-Plus filter-list parser.

use super::rule::Rule;

use self::comment::comment;
use self::cosmetic::cosmetic;
use self::network::network;

mod comment;
mod cosmetic;
mod network;

/// Parse a complete filter-list string into rules.
pub(crate) fn parse_list(text: &str) -> Vec<Rule> {
    text.lines().map(parse_line).collect()
}

pub(super) fn parse_line(line: &str) -> Rule {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('!') || trimmed.starts_with('[') {
        return comment(trimmed);
    }
    if let Some(rule) = cosmetic(trimmed) {
        return rule;
    }
    if let Some(rest) = trimmed.strip_prefix("@@") {
        return network(rest, true);
    }
    network(trimmed, false)
}
