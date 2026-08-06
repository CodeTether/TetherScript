//! Conversions between [`Rule`] structs and TetherScript [`Value`]s.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::adblock::rule::{FilterType, Rule};
use crate::value::Value;

use super::accessors::{list_of, new_map, set};

/// Convert compiled rules into a TetherScript list of rule maps.
pub(super) fn rules_to_value(rules: &[Rule]) -> Value {
    let list: Vec<Value> = rules.iter().filter_map(rule_to_map).collect();
    Value::List(Rc::new(RefCell::new(list)))
}

fn rule_to_map(rule: &Rule) -> Option<Value> {
    if rule.filter_type == FilterType::Comment {
        return None;
    }
    let map = new_map();
    set(
        &map,
        "type",
        Value::Str(Rc::new(type_name(&rule.filter_type).into())),
    );
    set(&map, "exception", Value::Bool(rule.exception));
    if !rule.pattern.is_empty() {
        set(&map, "pattern", Value::Str(Rc::new(rule.pattern.clone())));
    }
    if !rule.selector.is_empty() {
        set(&map, "selector", Value::Str(Rc::new(rule.selector.clone())));
    }
    set(&map, "domains", list_of(&rule.domains));
    set(&map, "excluded_domains", list_of(&rule.excluded_domains));
    if let Some(tp) = rule.third_party {
        set(&map, "third_party", Value::Bool(tp));
    }
    Some(Value::Map(map))
}

fn type_name(ft: &FilterType) -> &'static str {
    match ft {
        FilterType::Network => "network",
        FilterType::CosmeticHide => "cosmetic_hide",
        FilterType::CosmeticAllow => "cosmetic_allow",
        FilterType::Comment => "comment",
    }
}
