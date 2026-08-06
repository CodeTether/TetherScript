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
    pub resource_types: super::ResourceType,
    pub selector: String,
}
