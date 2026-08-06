//! Rule model produced by the filter-list parser.

/// Broad classification of a single filter line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterType {
    Network,
    CosmeticHide,
    CosmeticAllow,
    Comment,
}

/// One compiled rule from a filter list.
#[derive(Clone, Debug)]
pub struct Rule {
    pub filter_type: FilterType,
    pub exception: bool,
    pub pattern: String,
    pub domains: Vec<String>,
    pub excluded_domains: Vec<String>,
    pub third_party: Option<bool>,
    /// Parsed from `$script`, `$image`, and friends.
    ///
    /// Populated by the parser but not yet consulted by `network::check_rules`,
    /// so `$type` modifiers currently widen to "any resource type" at match
    /// time. Enforcing them needs a request-type argument threaded through
    /// `adblock_should_block`, which is a built-in signature change.
    #[allow(
        dead_code,
        reason = "parsed but resource-type matching is not wired yet"
    )]
    pub resource_types: super::ResourceType,
    pub selector: String,
}
