//! Conversions from TetherScript [`Value`]s back into [`Rule`] structs.

use super::super::adblock::rule::{FilterType, Rule};
use super::super::adblock::ResourceType;
use crate::value::Value;

use super::accessors::{get_bool, get_list, get_opt_bool, get_str};

/// Decode a TetherScript list of rule maps back into compiled rules.
pub(super) fn value_to_rules(value: &Value) -> Result<Vec<Rule>, String> {
    let Value::List(list) = value else {
        return Err(format!(
            "adblock: rules must be list, got {}",
            value.type_name()
        ));
    };
    Ok(list.borrow().iter().map(map_to_rule).collect())
}

fn map_to_rule(value: &Value) -> Rule {
    let Value::Map(map) = value else {
        return blank_rule();
    };
    let map = map.borrow();
    Rule {
        filter_type: parse_type(&get_str(&map, "type")),
        exception: get_bool(&map, "exception"),
        pattern: get_str(&map, "pattern"),
        domains: get_list(&map, "domains"),
        excluded_domains: get_list(&map, "excluded_domains"),
        third_party: get_opt_bool(&map, "third_party"),
        resource_types: ResourceType::all(),
        selector: get_str(&map, "selector"),
    }
}

fn blank_rule() -> Rule {
    Rule {
        filter_type: FilterType::Comment,
        exception: false,
        pattern: String::new(),
        domains: Vec::new(),
        excluded_domains: Vec::new(),
        third_party: None,
        resource_types: ResourceType::all(),
        selector: String::new(),
    }
}

fn parse_type(name: &str) -> FilterType {
    match name {
        "network" => FilterType::Network,
        "cosmetic_hide" => FilterType::CosmeticHide,
        "cosmetic_allow" => FilterType::CosmeticAllow,
        _ => FilterType::Comment,
    }
}
