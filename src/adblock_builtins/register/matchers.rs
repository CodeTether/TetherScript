//! Network and cosmetic matching builtins.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::adblock::{
    cosmetic::CosmeticMatcher, network::NetworkMatcher, rule::Rule,
};
use crate::value::Value;

/// Test whether a request URL should be blocked.
pub(super) fn block_value(rules: &[Rule], url: &str, domain: &str) -> bool {
    let mut net = NetworkMatcher::new();
    net.add(rules);
    net.is_blocked(url, domain)
}

/// Return cosmetic selectors as a Value::List.
pub(super) fn cosmetic_list(rules: &[Rule], domain: &str) -> Value {
    let mut cos = CosmeticMatcher::new();
    cos.add(rules);
    let list = cos
        .selectors_for(domain)
        .into_iter()
        .map(Rc::new)
        .map(Value::Str)
        .collect();
    Value::List(Rc::new(RefCell::new(list)))
}
